// Copyright 2021 poonai
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::sql::ctx::Ctx;
use crate::sql::error::InspektorSqlError;
use crate::sql::rule_engine::{HardRuleEngine, RuleEngine};

use protobuf::ProtobufEnum;
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, Ident, Query, Select, SelectItem, SetExpr, Statement,
    TableFactor, TrimWhereField, Value,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use std::borrow::Cow;
use std::marker::PhantomData;
// QueryRewriter validates the user query and rewrites if neccessary.
pub struct QueryRewriter<T: RuleEngine> {
    // rule engine is responsible for handling all the rules which are enforced by
    // the end user.
    namespaces: Vec<String>,
    rule_engine: T,
}

impl<T: RuleEngine> QueryRewriter<T> {
    // new will return query rewriter
    pub fn new(rule_engine: T, ns: Vec<String>) -> QueryRewriter<T> {
        return QueryRewriter {
            rule_engine: rule_engine,
            namespaces: ns,
        };
    }

    pub fn rewrite(
        &self,
        statements: &mut Vec<Statement>,
        state: Ctx,
    ) -> Result<(), InspektorSqlError> {
        for statement in statements {
            match statement {
                Statement::Query(query) => {
                    self.handle_query(query, &state)?;
                }
                _ => {
                    continue;
                }
            }
        }
        Ok(())
    }

    // handle_query will validate the query with the rule engine. it's not valid
    // it's throw an error or it' try to rewrite to make the query to match
    // the rule.
    pub fn handle_query(&self, query: &mut Query, state: &Ctx) -> Result<Ctx, InspektorSqlError> {
        let mut local_state = state.clone();
        // cte table are user created temp table passed down to the subsequent query.
        // so it's is mandatory to validate cte first and build the state
        // to push down to the subsequent statement.
        if let Some(with) = &mut query.with {
            for cte in &mut with.cte_tables {
                // there is no need to merge state of cte table. because cte tables are already
                // filtered in the query state.
                 self.handle_query(&mut cte.query, state)?;
            }
        }
        // we'll evaulate the body first because that is the data which will be retrived for the
        // subsequent query evaluation.
        self.handle_set_expr(&mut query.body, &local_state)
    }

    // handle_set_expr handles set exprs which are basically query, insert,
    // select.. all the core block of the ANSI SQL.
    fn handle_set_expr(&self, expr: &mut SetExpr, state: &Ctx) -> Result<Ctx, InspektorSqlError> {
        match expr {
            SetExpr::Query(query) => return self.handle_query(query, state),
            SetExpr::Select(select) => return self.handle_select(select, state),
            SetExpr::SetOperation {
                op,
                all: _,
                left,
                right,
            } => {
                // set operation are union or intersect of set_expr
                // eg (select * from premimum users) UNION (select * from users);
                let left_state = self.handle_set_expr(left, state)?;
                let right_state = self.handle_set_expr(right, state)?;
                // usually left and right should be selection because it's a union call.
                // so let's check the projection left and right have same number of projections
                // so we can hit the client about the kind of error.
                let left_count = match &**left {
                    SetExpr::Select(select) => Some(select.projection.len()),
                    _ => return Ok(right_state),
                };
                let right_count = match &**right {
                    SetExpr::Select(select) => Some(select.projection.len()),
                    _ => return Ok(left_state),
                };
                if left_count != right_count {
                    return Err(
                        InspektorSqlError::Error(
                            format!("{} requires same number of column left and right. may be avoid using wildcard `*`", op)
                        )
                    );
                }
                // it's is safe to return one state because both left and right carries same
                // columns.
                Ok(right_state)
            }
            _ => {
                unreachable!("not handled set expr {:?}", expr);
            }
        }
    }

    // handle_select handles select statement. select statement data are the one which are
    // returned to the user.
    fn handle_select(&self, select: &mut Select, state: &Ctx) -> Result<Ctx, InspektorSqlError> {
        let mut local_state = state.clone();
        // select projection are not from a table so we don't need to do anythings here.
        // TODO: I'm not convinced about the fast path.
        if select.from.len() == 0 {
            return Ok(local_state);
        }
        // from selection defines what all fields that are allowed for the from tables.
        for from in &mut select.from {
            let factor_state = self.handle_table_factor(state, &mut from.relation)?;
            local_state.merge_state(factor_state);
            for join in &mut from.joins {
                let factor_state = self.handle_table_factor(state, &mut join.relation)?;
                local_state.merge_state(factor_state);
            }
        }
        let mut projection = Vec::with_capacity(select.projection.len());
        // filter out the the allowed projection if it's wildcard. otherwise,
        // check incoming projection is in allowed list.
        for selection in &mut select.projection {
            projection.extend(self.handle_selection(&local_state, selection)?);
        }
        select.projection = projection;
        Ok(local_state)
    }

    // handle_table_factor handles (FROM table). here all the possible columns allowed for the
    // given table is decided.
    fn handle_table_factor(
        &self,
        state: &Ctx,
        table_factor: &mut TableFactor,
    ) -> Result<Ctx, InspektorSqlError> {
        let mut local_state = state.clone();
        match table_factor {
            TableFactor::Table {
                name,
                alias,
                args: _args,
                with_hints: _with_hints,
            } => {
                let mut table_name = join_indents(&name.0);
                local_state.add_from_src(table_name.clone());
                // before checking the rule engine. we have to check the state becauase this can be cte table
                // or some aliased table so we have to check the state before advancing to the rule engine.
                let mut protected_columns = match local_state.get_protected_columns(&table_name) {
                    Some(cols) => cols,
                    None => {
                        let mut cols = vec![];
                        if let Some(protected_cols) =
                            self.rule_engine.get_protected_columns(&table_name)
                        {
                            cols = protected_cols;
                        } else {
                            for ns in &self.namespaces {
                                let ns_table_name = format!("{}.{}", ns, &table_name);
                                if let Some(columns) =
                                    self.rule_engine.get_protected_columns(&ns_table_name)
                                {
                                    if columns.len() == 0 {
                                        return Err(InspektorSqlError::UnAuthorizedColumn((
                                            Some(table_name),
                                            "".to_string(),
                                        )));
                                    }
                                    cols = columns;
                                    table_name = ns_table_name;
                                    break;
                                }
                            }
                        }
                        cols
                    }
                };
                if protected_columns.len() == 0 {
                    return Ok(local_state)
                }
                if let Some(alias) = alias {
                    let alias_name = alias.name.value.clone();
                    local_state.overwrite_table_info(&table_name, alias_name.clone());
                    table_name = alias_name;
                } else {
                    let from_table_name = join_indents(&name.0);
                    local_state.overwrite_table_info(&table_name, from_table_name.clone());
                    table_name = from_table_name;
                }
                local_state.memorize_protected_columns(table_name.clone(), protected_columns);
            }
            TableFactor::Derived {
                lateral,
                subquery,
                alias,
            } => {
                // derived table are the subquery in the FROM clause.
                // eg: SELECT * from (select * from premimum users limit by 10) as users;
                if alias.is_none() {
                    return Err(InspektorSqlError::FromNeedAlias);
                }
                let subquery_alias = alias.as_ref().unwrap();
                // we have a subquery now.
                self.handle_query(subquery, &local_state)?;
                local_state.add_from_src(subquery_alias.name.value.clone());
            }
            TableFactor::NestedJoin(table) => {
                let factor_state = self.handle_table_factor(state, &mut table.relation)?;
                local_state.merge_state(factor_state);
                for join in &mut table.joins {
                    let factor_state = self.handle_table_factor(state, &mut join.relation)?;
                    local_state.merge_state(factor_state);
                }
            }
            _ => {
                unreachable!("not handled statement {:?}", table_factor);
            }
        }
        Ok(local_state)
    }

    // handle_selection handles the selected field. This is bottom down of the evaluation, here
    // we validate that the selected field is in the allowed columns. if not it'll throw an
    // unauthorized error. But, query rewriter always tries to rewrite it's possible to
    // adhere the rule.
    fn handle_selection(
        &self,
        state: &Ctx,
        selection: &mut SelectItem,
    ) -> Result<Vec<SelectItem>, InspektorSqlError> {
        match selection {
            SelectItem::UnnamedExpr(expr) => {
                if let Err(e) = self.handle_expr(state, expr) {
                    match e {
                        InspektorSqlError::RewriteExpr { alias_name } => {
                            return Ok(vec![SelectItem::ExprWithAlias {
                                expr: Expr::Value(Value::Null),
                                alias: Ident::new(alias_name),
                            }]);
                        }
                        _ => return Err(e),
                    }
                }
                return Ok(vec![SelectItem::UnnamedExpr(expr.clone())]);
            }
            SelectItem::Wildcard => {
                // for wildcard we just rewrite with all the allowed columns.
                return Ok(state.build_allowed_column_expr());
            }
            SelectItem::ExprWithAlias { expr, alias } => {
                if let Err(e) = self.handle_expr(state, expr) {
                    match e {
                        InspektorSqlError::RewriteExpr { .. } => {
                            return Ok(vec![SelectItem::ExprWithAlias {
                                expr: Expr::Value(Value::Null),
                                alias: alias.clone(),
                            }]);
                        }
                        _ => return Err(e),
                    }
                }
                return Ok(vec![selection.clone()]);
            }
            SelectItem::QualifiedWildcard(object_name) => {
                // first ident must be table.
                let table_name = &object_name.0[0].value;
                return Ok(state.column_expr_for_table(&Cow::Borrowed(table_name)));
            }
        }
    }
    // handle_expr will handle all the selection expr. eg:
    // SUM(balance) or balance...
    fn handle_expr(&self, state: &Ctx, expr: &mut Expr) -> Result<(), InspektorSqlError> {
        match expr {
            Expr::Identifier(object_name) => {
                // it's a single expression. if we have one table the we pick that.
                if !state.is_allowed_column_ident(&object_name.value) {
                    // column is not allowed but it's a valid column.
                    // so, rewriting the query to return NULL for the given column.
                    // the main reason to do this is that, folks who uses postgres
                    // with other analytical tools won't find any disturbance.
                    // eg: metabase.
                    return Err(InspektorSqlError::RewriteExpr {
                        alias_name: object_name.value.clone(),
                    });
                }
            }
            Expr::CompoundIdentifier(identifiers) => {
                let (table_name, column_name) = get_column_from_idents(&identifiers);
                if !state.is_allowed_column(&table_name, &column_name) {
                    return Err(InspektorSqlError::RewriteExpr {
                        alias_name: join_indents(&identifiers),
                    });
                }
            }
            Expr::Subquery(query) => {
                // we don't need to do any merge of the state.
                // because this comes as select projection.
                self.handle_query(query, state)?;
            }
            Expr::Function(function) => {
                // validate all the args whether it's allowed or not.
                // for the function args we'll rewrite with NULL value if it's not allowed.
                for arg in &mut function.args {
                    match arg {
                        FunctionArg::Unnamed(arg_expr) => {
                            let expr = match arg_expr {
                                FunctionArgExpr::Expr(expr) => expr,
                                FunctionArgExpr::QualifiedWildcard(_) => return Ok(()),
                                FunctionArgExpr::Wildcard => return Ok(()),
                            };
                            if let Err(_) = self.handle_expr(state, expr) {
                                *expr = Expr::Value(Value::Null)
                            }
                        }
                        FunctionArg::Named { name: _name, arg } => {
                            let expr = match arg {
                                FunctionArgExpr::Expr(expr) => expr,
                                FunctionArgExpr::QualifiedWildcard(_) => return Ok(()),
                                FunctionArgExpr::Wildcard => return Ok(()),
                            };
                            if let Err(_) = self.handle_expr(state, expr) {
                                *expr = Expr::Value(Value::Null)
                            }
                        }
                    };
                }
            }
            Expr::Case {
                operand,
                conditions,
                results,
                else_result,
            } => {
                // example query:
                // SELECT
                //     id,
                //     CASE
                //         WHEN rating~E'^\\d+$' THEN
                //             CAST (rating AS INTEGER)
                //         ELSE
                //             0
                //         END as rating
                // FROM
                //     ratings
                if let Some(operand) = operand {
                    self.handle_expr(state, operand)?;
                }
                for condition in conditions {
                    // if the condition fail then we rewrite with case.
                    if let Err(_) = self.handle_expr(state, condition) {
                        return Err(InspektorSqlError::RewriteExpr {
                            alias_name: String::from("case"),
                        });
                    }
                }
                for result in results {
                    if let Err(_) = self.handle_expr(state, result) {
                        return Err(InspektorSqlError::RewriteExpr {
                            alias_name: String::from("case"),
                        });
                    }
                }
                if let Some(else_result) = else_result {
                    if let Err(_) = self.handle_expr(state, else_result) {
                        return Err(InspektorSqlError::RewriteExpr {
                            alias_name: String::from("case"),
                        });
                    }
                }
            }
            Expr::Cast { expr, .. } => self.handle_expr(state, expr)?,
            Expr::TryCast { expr, .. } => self.handle_expr(state, expr)?,
            Expr::Extract { expr, .. } => {
                if let Err(_) = self.handle_expr(state, expr) {
                    return Err(InspektorSqlError::RewriteExpr {
                        alias_name: String::from("date_part"),
                    });
                }
            } // date_part
            Expr::Collate { expr, .. } => {
                if let Err(_) = self.handle_expr(state, expr) {
                    return Err(InspektorSqlError::RewriteExpr {
                        alias_name: String::from("collate"),
                    });
                }
            }
            Expr::Nested(expr) => {
                self.handle_expr(state, expr)?;
            }
            Expr::Trim { expr, trim_where } => {
                // default_column will tell the default column name is used for
                // the select item rewrite.
                let mut default_column_name = &"btrim";
                if let Some((trim_where_field, expr)) = trim_where {
                    match trim_where_field {
                        TrimWhereField::Leading => {
                            default_column_name = &"ltrim";
                        }
                        TrimWhereField::Trailing => {
                            default_column_name = &"rtrim";
                        }
                        _ => {}
                    }
                    if let Err(_) = self.handle_expr(state, expr) {
                        return Err(InspektorSqlError::RewriteExpr {
                            alias_name: default_column_name.to_string(),
                        });
                    }
                }
                if let Err(_) = self.handle_expr(state, expr) {
                    return Err(InspektorSqlError::RewriteExpr {
                        alias_name: default_column_name.to_string(),
                    });
                };
            }
            Expr::Substring {
                expr,
                substring_from,
                substring_for,
            } => {
                if let Err(_) = self.handle_expr(state, expr) {
                    return Err(InspektorSqlError::RewriteExpr {
                        alias_name: "substring".to_string(),
                    });
                }
                if let Some(from) = substring_from {
                    if let Err(_) = self.handle_expr(state, from) {
                        return Err(InspektorSqlError::RewriteExpr {
                            alias_name: "substring".to_string(),
                        });
                    }
                }
                if let Some(expr) = substring_for {
                    if let Err(_) = self.handle_expr(state, expr) {
                        return Err(InspektorSqlError::RewriteExpr {
                            alias_name: "substring".to_string(),
                        });
                    }
                }
            }
            Expr::Value(_) | Expr::TypedString { .. } => {
                // things that needs no evaluation.
            }
            Expr::BinaryOp {
                left,
                op: _op,
                right,
            } => {
                if let Err(_) = self.handle_expr(state, left) {
                    *left = Box::new(Expr::Value(Value::Null));
                };
                if let Err(_) = self.handle_expr(state, right) {
                    *right = Box::new(Expr::Value(Value::Null));
                }
            }
            Expr::UnaryOp { op: _op, expr } => {
                if let Err(_) = self.handle_expr(state, expr) {
                    *expr = Box::new(Expr::Value(Value::Null));
                }
            }
            Expr::IsNull(_)
            | Expr::IsNotNull(_)
            | Expr::IsDistinctFrom(_, _)
            | Expr::IsNotDistinctFrom(_, _)
            | Expr::InList { .. }
            | Expr::InSubquery { .. }
            | Expr::Between { .. } => {
                // these are list of expression used by where cause so we just
                // simply don't do anything.
                log::warn!("where clause expression executer, please report to author if you find this log. ");
            }
            _ => unreachable!("unknown expression {} {:?}", expr, expr),
        }
        Ok(())
    }
}

/// join_indents will join the indent with dotted operation.
pub fn join_indents(idents: &Vec<Ident>) -> String {
    return idents
        .iter()
        .map(|i| i.value.clone())
        .collect::<Vec<String>>()
        .join(".");
}

// get_column_from_idents returns the column name and table.
pub fn get_column_from_idents(idents: &Vec<Ident>) -> (String, String) {
    assert_eq!(idents.len() >= 2, true);
    (
        idents[..idents.len() - 1]
            .iter()
            .map(|i| i.value.clone())
            .collect::<Vec<String>>()
            .join("."),
        idents[idents.len() - 1].value.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json;
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    macro_rules! cowvec {
        ( $( $x:expr ),* ) => {
            {
                let mut temp_vec = Vec::new();
                $(
                    temp_vec.push(String::from($x));
                )*
                temp_vec
            }
        };
    }

    fn assert_rewriter<T: RuleEngine>(
        rewriter: &QueryRewriter<T>,
        state: Ctx,
        input: &'static str,
        output: &'static str,
    ) {
        let dialect = PostgreSqlDialect {};
        let mut statements = Parser::parse_sql(&dialect, input).unwrap();
        rewriter.rewrite(&mut statements, state).unwrap();
        assert_eq!(output, format!("{}", statements[0]))
    }

    fn assert_error<T: RuleEngine>(
        rewriter: &QueryRewriter<T>,
        state: Ctx,
        input: &'static str,
        err: InspektorSqlError,
    ) {
        let dialect = PostgreSqlDialect {};
        let mut statements = Parser::parse_sql(&dialect, input).unwrap();
        let rewriter_err = rewriter.rewrite(&mut statements, state).unwrap_err();
        assert_eq!(rewriter_err, err)
    }

    #[test]
    fn test_for_output() {
        let dialect = PostgreSqlDialect {};
        let statements =
            Parser::parse_sql(&dialect, r#"select (array['Yes', 'No', 'Maybe']);"#).unwrap();
        println!("{:?}", statements[0])
    }
    #[test]
    fn basic_select() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
        };

        let state = Ctx::new(HashMap::from([(
            String::from("public.kids"),
            vec![
                String::from("phone"),
                String::from("id"),
                String::from("name"),
                String::from("address"),
            ],
        )]));

        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "select * from kids",
            "SELECT NULL AS kids.phone, kids.id, kids.name, kids.address FROM kids",
        );

        assert_rewriter(
            &rewriter,
            state,
            "SELECT * FROM public.kids",
            "SELECT NULL AS public.kids.phone, public.kids.id, public.kids.name, public.kids.address FROM public.kids",
        );
    }

    #[test]
    fn test_simple_join() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
        };

        let state = Ctx::new(HashMap::from([
            (
                String::from("weather"),
                cowvec!("city", "temp_lo", "temp_hi", "prcp", "date"),
            ),
            (String::from("cities"), cowvec!("name", "location")),
        ]));

        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(&rewriter, state, "SELECT w.city, w.temp_lo, w.temp_hi,
        w.prcp, w.date, cities.location
        FROM weather as w, cities
        WHERE cities.name = w.city;", "SELECT w.city, w.temp_lo, w.temp_hi, w.prcp, w.date, cities.location FROM weather AS w, cities WHERE cities.name = w.city");
    }

    #[test]
    fn test_cte() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
        };

        let state = Ctx::new(HashMap::from([(
            String::from("public.kids"),
            vec![
                String::from("phone"),
                String::from("id"),
                String::from("name"),
                String::from("address"),
            ],
        )]));

        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &rewriter,
            state,
            "WITH DUMMY AS (SELECT * FROM kids LIMIT 1)
            SELECT * FROM DUMMY",
            "WITH DUMMY AS (SELECT NULL AS kids.phone, kids.id, kids.name, kids.address FROM kids LIMIT 1) SELECT * FROM DUMMY",
        );
    }

    #[test]
    fn test_subquery() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
        };

        let state = Ctx::new(HashMap::from([(
            String::from("public.kids"),
            vec![
                String::from("phone"),
                String::from("id"),
                String::from("name"),
                String::from("address"),
            ],
        )]));

        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "select * from (select * from public.kids) as nested",
            "SELECT * FROM (SELECT NULL AS public.kids.phone, public.kids.id, public.kids.name, public.kids.address FROM public.kids) AS nested",
        );
        assert_rewriter(
            &rewriter,
            state,
            "select * from (with dummy as (select * from kids) select * from dummy)as nested limit 1;",
            "SELECT * FROM (WITH dummy AS (SELECT NULL AS kids.phone, kids.id, kids.name, kids.address FROM kids) SELECT * FROM dummy) AS nested LIMIT 1",
        );
    }

    #[test]
    fn test_union() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([
                (String::from("public.kids"), vec![String::from("phone")]),
                (String::from("public.kids2"), vec![String::from("phone")]),
            ]),
        };

        let state = Ctx::new(HashMap::from([
            (
                String::from("public.kids"),
                vec![
                    String::from("phone"),
                    String::from("id"),
                    String::from("name"),
                    String::from("address"),
                ],
            ),
            (
                String::from("public.kids2"),
                vec![
                    String::from("phone"),
                    String::from("id"),
                    String::from("name"),
                    String::from("address"),
                ],
            ),
        ]));
        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        // assert_rewriter(
        //     &rewriter,
        //     state.clone(),
        //     "select * from kids UNION select * from public.kids2",
        //     "SELECT kids.id, kids.name, kids.address FROM kids UNION SELECT public.kids2.id, public.kids2.name, public.kids2.address FROM public.kids2",
        // );

        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT 49179 AS oid , 1 AS attnum UNION ALL SELECT 49179, 7;",
            "SELECT 49179 AS oid, 1 AS attnum UNION ALL SELECT 49179, 7",
        );
    }

    #[test]
    fn test_joins() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([
                (String::from("public.kids"), vec![String::from("phone")]),
                (String::from("public.kids2"), vec![String::from("phone")]),
            ]),
        };

        let state = Ctx::new(HashMap::from([
            (
                String::from("public.weather"),
                vec![
                    String::from("city"),
                    String::from("temp_lo"),
                    String::from("temp_hi"),
                    String::from("prcp"),
                ],
            ),
            (
                String::from("public.cities"),
                vec![
                    String::from("name"),
                    String::from("state"),
                    String::from("country"),
                    String::from("location"),
                ],
            ),
        ]));
        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT *
            FROM weather INNER JOIN public.cities ON (weather.city = public.cities.name);",
            "SELECT public.cities.name, public.cities.state, public.cities.country, public.cities.location, weather.city, weather.temp_lo, weather.temp_hi, weather.prcp FROM weather JOIN public.cities ON (weather.city = public.cities.name)",
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT w.city, w.temp_lo, w.temp_hi,
                         w.prcp, cities.location
                      FROM weather as w, cities
                      WHERE cities.name = w.city;",
            "SELECT w.city, w.temp_lo, w.temp_hi, w.prcp, cities.location FROM weather AS w, cities WHERE cities.name = w.city",
        );
    }

    #[test]
    fn test_projection_expr() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([
                (String::from("kids"), vec![String::from("phone")]),
                (String::from("kids2"), vec![String::from("phone")]),
            ]),
        };

        let state = Ctx::new(HashMap::from([
            (
                String::from("weather"),
                vec![
                    String::from("city"),
                    String::from("temp_lo"),
                    String::from("temp_hi"),
                    String::from("prcp"),
                ],
            ),
            (
                String::from("cities"),
                vec![
                    String::from("name"),
                    String::from("state"),
                    String::from("country"),
                    String::from("location"),
                ],
            ),
        ]));
        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT *, (select sum(temp_hi) from weather) as temp_hi
            FROM cities",
            "SELECT cities.name, cities.state, cities.country, cities.location, (SELECT sum(temp_hi) FROM weather) AS temp_hi FROM cities",
        );
    }

    #[test]
    fn test_wildcard_qualified_wildcard() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
        };

        let state = Ctx::new(HashMap::from([(
            String::from("kids"),
            vec![
                String::from("phone"),
                String::from("id"),
                String::from("name"),
                String::from("address"),
            ],
        )]));

        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &rewriter,
            state,
            "select kids.* from kids",
            "SELECT kids.id, kids.name, kids.address FROM kids",
        );
    }

    #[test]
    fn test_expr() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
        };

        let state = Ctx::new(HashMap::from([(
            String::from("kids"),
            vec![
                String::from("phone"),
                String::from("id"),
                String::from("name"),
                String::from("address"),
            ],
        )]));

        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(&rewriter, state.clone(), "SELECT 1", "SELECT 1");

        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT count(*) from kids",
            "SELECT count(*) FROM kids",
        );

        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT CASE when address > 10 then address else 0 end from kids",
            "SELECT CASE WHEN address > 10 THEN address ELSE 0 END FROM kids",
        );

        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT name < address COLLATE "de_DE" FROM kids"#,
            r#"SELECT name < address COLLATE "de_DE" FROM kids"#,
        );

        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT name < address COLLATE "de_DE" FROM kids"#,
            r#"SELECT name < address COLLATE "de_DE" FROM kids"#,
        );

        // TODO: collate test.
        // assert_rewriter(
        //     &rewriter,
        //     state.clone(),
        //     r#"SELECT phone < address COLLATE "de_DE" FROM kids"#,
        //     r#"SELECT NULL < address COLLATE "de_DE" FROM kids"#,
        // );

        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT CAST(name as INTEGER), EXTRACT(month from id) FROM kids"#,
            r#"SELECT CAST(name AS INT), EXTRACT(MONTH FROM id) FROM kids"#,
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT CAST(name as INTEGER), EXTRACT(month from phone) FROM kids"#,
            r#"SELECT CAST(name AS INT), NULL AS date_part FROM kids"#,
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT CAST(phone as INTEGER), EXTRACT(month from id) FROM kids"#,
            r#"SELECT NULL AS phone, EXTRACT(MONTH FROM id) FROM kids"#,
        );
    }

    #[test]
    fn test_rewrite_null() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
        };

        let state = Ctx::new(HashMap::from([(
            String::from("public.kids"),
            vec![
                String::from("phone"),
                String::from("id"),
                String::from("name"),
                String::from("address"),
            ],
        )]));

        let rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT id, phone from kids",
            "SELECT id, NULL AS phone FROM kids",
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT id, phone AS demophone from kids",
            "SELECT id, NULL AS demophone FROM kids",
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            "SELECT SUM(phone) from kids",
            "SELECT SUM(NULL) FROM kids",
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT case phone when 10 then "hello" end from kids"#,
            "SELECT NULL AS phone FROM kids",
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT SUM(case phone when 10 then 1 end) from kids"#,
            "SELECT SUM(NULL) FROM kids",
        );
        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT substring(phone from 10) from kids"#,
            "SELECT NULL AS substring FROM kids",
        );
    }
    #[derive(Deserialize, Debug)]
    struct SchemaInformation {
        c0: String,
        c1: String,
        c2: String,
        c3: String,
    }

    fn get_table_info() -> HashMap<String, Vec<String>> {
        let path = env::current_dir().unwrap();
        let table_data = fs::read(path.join("src/sql/default_columns.json")).unwrap();
        let infos: Vec<SchemaInformation> = serde_json::from_slice(&table_data[..]).unwrap();
        let mut table_info: HashMap<String, Vec<String>> = HashMap::default();
        for info in infos {
            let table_name: String = format!("{}.{}", info.c0, info.c1);
            let column_name: String = info.c2;
            if let Some(columns) = table_info.get_mut(&table_name) {
                columns.push(column_name);
                continue;
            }
            table_info.insert(table_name, vec![column_name]);
        }
        table_info
    }

    #[test]
    fn test_metabase_expr() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
        };

        let state = Ctx::new(get_table_info());

        let rewriter = QueryRewriter::new(
            rule_engine,
            vec!["public".to_string(), "pg_catalog".to_string()],
        );
        assert_rewriter(&rewriter, state.clone(), "SELECT DISTINCT t.typname FROM pg_enum e LEFT JOIN pg_type t ON t.oid = e.enumtypid", "SELECT DISTINCT t.typname FROM pg_enum AS e LEFT JOIN pg_type AS t ON t.oid = e.enumtypid");
    }
}
