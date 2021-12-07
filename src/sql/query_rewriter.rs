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
use protobuf::well_known_types::Option;
use protobuf::ProtobufEnum;
use sqlparser::ast::{
    Expr, FunctionArg, Ident, Query, Select, SelectItem, SetExpr, Statement, TableFactor,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use std::borrow::Cow;
use std::marker::PhantomData;
// QueryRewriter validates the user query and rewrites if neccessary.
pub struct QueryRewriter<T: RuleEngine> {
    // rule engine is responsible for handling all the rules which are enforced by
    // the end user.
    rule_engine: T,
}

impl<T: RuleEngine> QueryRewriter<T> {
    // new will return query rewriter
    pub fn new(rule_engine: T) -> QueryRewriter<T> {
        return QueryRewriter {
            rule_engine: rule_engine,
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
                let cte_state = self.handle_query(&mut cte.query, state)?;
                // cte state are pushed to the underlying table so let's merge allowed columns
                // to table info.
                let table_name = cte.alias.name.value.clone();
                local_state.merge_table_info(table_name, cte_state);
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
                let _left_state = self.handle_set_expr(left, state)?;
                let right_state = self.handle_set_expr(right, state)?;
                // usually left and right should be selection because it's a union call.
                // so let's check the projection left and right have same number of projections
                // so we can hit the client about the kind of error.
                let left_count = match &**left {
                    SetExpr::Select(select) => Some(select.projection.len()),
                    _ => None,
                };
                let right_count = match &**right {
                    SetExpr::Select(select) => Some(select.projection.len()),
                    _ => None,
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

                // we need to find whether this table is with public schema or not.
                // if we have columns directly then means we got the table in a derivied
                // form.
                let (mut table_name, table_columns) = match state.get_columns(&table_name) {
                    Some(columns) => (table_name, columns),
                    _ => {
                        table_name = format!("public.{}", table_name);
                        match state.get_columns(&table_name) {
                            Some(columns) => (table_name, columns),
                            None => {
                                unreachable!("unable to get columns name for the table {:?}", name)
                            }
                        }
                    }
                };

                // TODO: table_name not mutated finding table name protected is
                // wrong.
                // if the given table is protected table then we should throw error.
                if self.rule_engine.is_table_protected(&table_name) {
                    return Err(InspektorSqlError::UnAuthorizedColumn((
                        Some(name.0[0].value.clone()),
                        "".to_string(),
                    )));
                }

                let mut allowed_columns = self
                    .rule_engine
                    .get_allowed_columns(&table_name, table_columns);
                if let Some(alias) = alias {
                    table_name = alias.name.value.clone()
                } else {
                    table_name = join_indents(&name.0);
                }
                let allowed_columns = allowed_columns
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>();
                local_state.insert_allowed_columns(table_name, allowed_columns);
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
                let derived_state = self.handle_query(subquery, &local_state)?;
                local_state
                    .merge_allowed_selections(subquery_alias.name.value.clone(), derived_state);
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
                self.handle_expr(state, expr)?;
                return Ok(vec![SelectItem::UnnamedExpr(expr.clone())]);
            }
            SelectItem::Wildcard => {
                // for wildcard we just rewrite with all the allowed columns.
                return Ok(state.build_allowed_column_expr());
            }
            SelectItem::ExprWithAlias {
                expr,
                alias: _alias,
            } => {
                self.handle_expr(state, expr)?;
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
                    return Err(InspektorSqlError::UnAuthorizedColumn((
                        state.get_default_table(),
                        object_name.value.clone(),
                    )));
                }
            }
            Expr::CompoundIdentifier(identifiers) => {
                let (table_name, column_name) = get_column_from_idents(&identifiers);
                if !state.is_allowed_column(&table_name, &column_name) {
                    return Err(InspektorSqlError::UnAuthorizedColumn((
                        Some(table_name),
                        column_name.clone(),
                    )));
                }
            }
            Expr::Subquery(query) => {
                // we don't need to do any merge of the state.
                // because this comes as select projection.
                self.handle_query(query, state)?;
            }
            Expr::Function(function) => {
                // validate all the args whether it's allowed or not.
                for arg in &mut function.args {
                    match arg {
                        FunctionArg::Unnamed(expr) => return self.handle_expr(state, expr),
                        _ => {
                            unreachable!("unknown fucntion args {} {:?}", arg, arg);
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
                    self.handle_expr(state, condition)?
                }
                for result in results {
                    self.handle_expr(state, result)?
                }
                if let Some(else_result) = else_result {
                    self.handle_expr(state, else_result)?
                }
            }
            Expr::Cast { expr, .. }
            | Expr::TryCast { expr, .. }
            | Expr::Extract { expr, .. }
            | Expr::Collate { expr, .. }
            | Expr::Nested(expr) => {
                self.handle_expr(state, expr)?;
            }
            Expr::Trim { expr, trim_where } => {
                self.handle_expr(state, expr)?;
                if let Some((_, expr)) = trim_where {
                    self.handle_expr(state, expr)?;
                }
            }
            Expr::Substring {
                expr,
                substring_from,
                substring_for,
            } => {
                self.handle_expr(state, expr)?;
                if let Some(from) = substring_from {
                    self.handle_expr(state, from)?;
                }
                if let Some(expr) = substring_for {
                    self.handle_expr(state, expr)?;
                }
            }
            Expr::Wildcard | Expr::QualifiedWildcard(_) => {
                // expr wild card come as parameter to a function.
                // eg: count(*) so we don't need to change anything.
            }
            Expr::Value(_) | Expr::TypedString { .. } => {
                // things that needs no evaluation.
            }
            Expr::BinaryOp {
                left,
                op: _op,
                right,
            } => {
                self.handle_expr(state, left)?;
                self.handle_expr(state, right)?;
            }
            Expr::UnaryOp { op: _op, expr } => self.handle_expr(state, expr)?,
            Expr::IsNull(_)
            | Expr::IsNotNull(_)
            | Expr::IsDistinctFrom(_, _)
            | Expr::IsNotDistinctFrom(_, _)
            | Expr::InList { .. }
            | Expr::InSubquery { .. }
            | Expr::Between { .. } => {
                // these are list of expression used by where cause so we just
                // simply don't do anything.
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
    use std::collections::HashMap;

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

        let rewriter = QueryRewriter::new(rule_engine);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "select * from kids",
            "SELECT kids.id, kids.name, kids.address FROM kids",
        );

        assert_rewriter(
            &rewriter,
            state,
            "select * from public.kids",
            "SELECT public.kids.id, public.kids.name, public.kids.address FROM public.kids",
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

        let rewriter = QueryRewriter::new(rule_engine);
        assert_rewriter(&rewriter, state, "SELECT w.city, w.temp_lo, w.temp_hi,
        w.prcp, w.date, cities.location
        FROM weather as w, cities
        WHERE cities.name = w.city;", "SELECT w.city, w.temp_lo, w.temp_hi, w.prcp, w.date, cities.location FROM weather AS w, cities WHERE cities.name = w.city");
    }

    #[test]
    fn test_cte() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("public.kids"), vec![String::from("phone")])]),
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

        let rewriter = QueryRewriter::new(rule_engine);
        assert_rewriter(
            &rewriter,
            state,
            "WITH DUMMY AS (SELECT * FROM kids LIMIT 1)
            SELECT * FROM DUMMY",
            "WITH DUMMY AS (SELECT kids.id, kids.name, kids.address FROM kids LIMIT 1) SELECT DUMMY.id, DUMMY.name, DUMMY.address FROM DUMMY",
        );
    }

    #[test]
    fn test_subquery() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("public.kids"), vec![String::from("phone")])]),
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

        let rewriter = QueryRewriter::new(rule_engine);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "select * from (select * from public.kids) as nested",
            "SELECT nested.id, nested.name, nested.address FROM (SELECT public.kids.id, public.kids.name, public.kids.address FROM public.kids) AS nested",
        );
        assert_rewriter(
            &rewriter,
            state,
            "select * from (with dummy as (select * from kids) select * from dummy)as nested limit 1;",
            "SELECT nested.id, nested.name, nested.address FROM (WITH dummy AS (SELECT kids.id, kids.name, kids.address FROM kids) SELECT dummy.id, dummy.name, dummy.address FROM dummy) AS nested LIMIT 1",
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
        let rewriter = QueryRewriter::new(rule_engine);
        assert_rewriter(
            &rewriter,
            state.clone(),
            "select * from kids UNION select * from public.kids2",
            "SELECT kids.id, kids.name, kids.address FROM kids UNION SELECT public.kids2.id, public.kids2.name, public.kids2.address FROM public.kids2",
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
        let rewriter = QueryRewriter::new(rule_engine);
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
        let rewriter = QueryRewriter::new(rule_engine);
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

        let rewriter = QueryRewriter::new(rule_engine);
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

        let rewriter = QueryRewriter::new(rule_engine);
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

        assert_error(
            &rewriter,
            state.clone(),
            r#"SELECT phone < address COLLATE "de_DE" FROM kids"#,
            InspektorSqlError::UnAuthorizedColumn((Some("kids".to_string()), "phone".to_string())),
        );

        assert_rewriter(
            &rewriter,
            state.clone(),
            r#"SELECT CAST(name as INTEGER), EXTRACT(month from id) FROM kids"#,
            r#"SELECT CAST(name AS INT), EXTRACT(MONTH FROM id) FROM kids"#,
        );
        assert_error(
            &rewriter,
            state.clone(),
            r#"SELECT CAST(name as INTEGER), EXTRACT(month from phone) FROM kids"#,
            InspektorSqlError::UnAuthorizedColumn((Some("kids".to_string()), "phone".to_string())),
        );
        assert_error(
            &rewriter,
            state.clone(),
            r#"SELECT CAST(phone as INTEGER), EXTRACT(month from id) FROM kids"#,
            InspektorSqlError::UnAuthorizedColumn((Some("kids".to_string()), "phone".to_string())),
        );
    }
}
