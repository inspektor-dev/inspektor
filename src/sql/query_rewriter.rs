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
use crate::sql::error::QueryRewriterError;
use crate::sql::rule_engine::RuleEngine;
use std::collections::{HashMap, HashSet};

use anyhow::Result;
use log::*;
use sqlparser::ast::{
    Assignment, Expr, FunctionArg, FunctionArgExpr, Ident, ObjectName, Query, Select, SelectItem,
    SetExpr, Statement, TableFactor, TableWithJoins, TrimWhereField, Value,
};
// QueryRewriter validates the user query and rewrites if neccessary.
pub struct QueryRewriter<T: RuleEngine + Clone> {
    // rule engine is responsible for handling all the rules which are enforced by
    // the end user.
    namespaces: Vec<String>,
    rule_engine: T,
    // metrics store all the tables and it's columns name it has accesssed.
    metrics: HashMap<String, HashSet<String>>,
}

impl<T: RuleEngine + Clone> QueryRewriter<T> {
    // new will return query rewriter
    pub fn new(rule_engine: T, ns: Vec<String>) -> QueryRewriter<T> {
        return QueryRewriter {
            rule_engine: rule_engine,
            namespaces: ns,
            metrics: Default::default(),
        };
    }

    pub fn rewrite(
        &mut self,
        statement: &mut Statement,
        state: &Ctx,
    ) -> Result<HashMap<String, HashSet<String>>, QueryRewriterError> {
        match statement {
            Statement::Query(query) => {
                self.handle_query(query, state)?;
            }
            Statement::Update {
                table, assignments, ..
            } => {
                if !self.rule_engine.is_update_allowed() {
                    return Err(QueryRewriterError::UnAuthorizedUpdate);
                }
                self.handle_update(&table, &assignments)?;
            }
            Statement::Insert {
                columns,
                table_name,
                ..
            } => {
                if !self.rule_engine.is_insert_allowed() {
                    return Err(QueryRewriterError::UnAuthorizedInsert);
                }
                let allowed_attributes = self.rule_engine.get_allowed_insert_attributes();
                if !self.is_operation_allowed(&table_name, &columns, allowed_attributes) {
                    return Err(QueryRewriterError::UnAuthorizedInsert);
                }
            }
            Statement::Copy {
                table_name,
                columns,
                ..
            } => {
                if !self.rule_engine.is_copy_allowed() {
                    return Err(QueryRewriterError::UnAthorizedCopy);
                }
                let allowed_attributes = self.rule_engine.get_allowed_copy_attributes();
                if !self.is_operation_allowed(&table_name, &columns, allowed_attributes) {
                    return Err(QueryRewriterError::UnAthorizedCopy);
                }
            }
            _ => {}
        }
        let metrics = std::mem::replace(&mut self.metrics, HashMap::default());
        Ok(metrics)
    }

    pub fn is_operation_allowed(
        &self,
        table_name: &ObjectName,
        columns: &Vec<Ident>,
        allowed_attributes: &HashMap<String, Vec<String>>,
    ) -> bool {
        if table_name.0.len() > 1 {
            // table name have a schema prefix so let's just
            // validate directly.
            let table_name = join_indents(&table_name.0);
            if self.validate_allowed_attributes(allowed_attributes, table_name, columns) {
                return true;
            }
            // this is unauthorized insert statment.
            return false;
        }
        let table_name = join_indents(&table_name.0);
        // let's add schema as prefix to validate the insert.
        let mut allowed = false;
        for ns in &self.namespaces {
            let ns_table_name = format!("{}.{}", ns, table_name);
            if self.validate_allowed_attributes(allowed_attributes, ns_table_name, columns) {
                allowed = true;
                break;
            }
        }
        if !allowed {
            return false;
        }
        return true;
    }

    pub fn validate_allowed_attributes(
        &self,
        allowed_attributes: &HashMap<String, Vec<String>>,
        table_name: String,
        cols: &Vec<Ident>,
    ) -> bool {
        match allowed_attributes.get(&table_name) {
            Some(allowed_cols) => {
                if allowed_cols.len() == 0 {
                    // since there is no columns to filter it's safe to allow this validation.
                    return true;
                }
                for col in cols {
                    if allowed_cols
                        .iter()
                        .position(|attribute| *attribute == col.value)
                        .is_none()
                    {
                        // incoming columns is not part of allowed column.
                        // so the caller operation is not allowed.
                        return false;
                    }
                }
                return true;
            }
            None => return false,
        }
    }

    /// handle_update validate the given table and assignment are allowed by the user, if not this
    /// throw error.
    pub fn handle_update(
        &self,
        table: &TableWithJoins,
        assignments: &Vec<Assignment>,
    ) -> Result<(), QueryRewriterError> {
        // postgres update will have only one table so it's safe to
        // find the table and validate the whether it's allowed to update.
        let table_name = match &table.relation {
            TableFactor::Table { name, .. } => name.0.clone(),
            _ => {
                warn!(
                    "unexpected releationship for the update statement table {:?}",
                    table
                );
                return Err(QueryRewriterError::UnAuthorizedUpdate);
            }
        };
        // let's get the columns which user wants to update.
        let mut columns: Vec<Ident> = vec![];
        for assignment in assignments {
            for column in &assignment.id {
                columns.push(column.clone());
            }
        }
        if !self.is_operation_allowed(
            &ObjectName(table_name),
            &columns,
            self.rule_engine.get_allowed_update_attributes(),
        ) {
            return Err(QueryRewriterError::UnAuthorizedUpdate);
        }
        Ok(())
    }

    // handle_query will validate the query with the rule engine. it's not valid
    // it's throw an error or it' try to rewrite to make the query to match
    // the rule.
    pub fn handle_query(
        &mut self,
        query: &mut Query,
        state: &Ctx,
    ) -> Result<Ctx, QueryRewriterError> {
        let local_state = state.clone();
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
    fn handle_set_expr(
        &mut self,
        expr: &mut SetExpr,
        state: &Ctx,
    ) -> Result<Ctx, QueryRewriterError> {
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
                self.handle_set_expr(left, state)?;
                let right_state = self.handle_set_expr(right, state)?;
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
    fn handle_select(
        &mut self,
        select: &mut Select,
        state: &Ctx,
    ) -> Result<Ctx, QueryRewriterError> {
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
        &mut self,
        state: &Ctx,
        table_factor: &mut TableFactor,
    ) -> Result<Ctx, QueryRewriterError> {
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
                let protected_columns = match local_state.get_protected_columns(&table_name) {
                    Some(cols) => Some(cols),
                    None => {
                        let mut cols = None;
                        if let Some(protected_cols) =
                            self.rule_engine.get_protected_columns(&table_name)
                        {
                            cols = Some(protected_cols);
                        } else {
                            for ns in &self.namespaces {
                                let ns_table_name = format!("{}.{}", ns, &table_name);
                                if let Some(columns) =
                                    self.rule_engine.get_protected_columns(&ns_table_name)
                                {
                                    if columns.len() == 0 {
                                        return Err(QueryRewriterError::UnAuthorizedColumn((
                                            Some(table_name),
                                            "".to_string(),
                                        )));
                                    }
                                    cols = Some(columns);
                                    table_name = ns_table_name;
                                    break;
                                }
                            }
                        }
                        cols
                    }
                };
                if protected_columns.is_none() {
                    return Ok(local_state);
                }
                let protected_columns = protected_columns.unwrap();
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
                subquery, alias, ..
            } => {
                // derived table are the subquery in the FROM clause.
                // eg: SELECT * from (select * from premimum users limit by 10) as users;
                if alias.is_none() {
                    return Err(QueryRewriterError::FromNeedAlias);
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
        &mut self,
        state: &Ctx,
        selection: &mut SelectItem,
    ) -> Result<Vec<SelectItem>, QueryRewriterError> {
        match selection {
            SelectItem::UnnamedExpr(expr) => {
                if let Err(e) = self.handle_expr(state, expr) {
                    match e {
                        QueryRewriterError::RewriteExpr { alias_name } => {
                            return Ok(vec![SelectItem::ExprWithAlias {
                                expr: Expr::Value(Value::Null),
                                alias: Ident {
                                    value: alias_name,
                                    quote_style: Some('"'),
                                },
                            }]);
                        }
                        _ => return Err(e),
                    }
                }
                return Ok(vec![SelectItem::UnnamedExpr(expr.clone())]);
            }
            SelectItem::Wildcard => {
                // for wildcard we just rewrite with all the allowed columns.
                return Ok(state.build_allowed_column_expr(&mut self.metrics));
            }
            SelectItem::ExprWithAlias { expr, alias } => {
                if let Err(e) = self.handle_expr(state, expr) {
                    match e {
                        QueryRewriterError::RewriteExpr { .. } => {
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
                let selections = state.column_expr_for_table(table_name, true, &mut self.metrics);
                if selections.len() != 0 {
                    return Ok(selections);
                }
                if let Some(properties)  = self.metrics.get_mut(table_name) {
                    properties.insert(format!("{}.*", table_name));
                } else { 
                    let mut properties = HashSet::new();
                    properties.insert(format!("{}.*", table_name));
                    self.metrics.insert(table_name.clone(), properties);
                }
                return Ok(vec![SelectItem::QualifiedWildcard(object_name.clone())]);
            }
        }
    }
    // handle_expr will handle all the selection expr. eg:
    // SUM(balance) or balance...
    fn handle_expr(&mut self, state: &Ctx, expr: &mut Expr) -> Result<(), QueryRewriterError> {
        match expr {
            Expr::Identifier(object_name) => {
                // it's a single expression. if we have one table the we pick that.
                if !state.is_allowed_column_ident(&object_name.value, &mut self.metrics) {
                    // column is not allowed but it's a valid column.
                    // so, rewriting the query to return NULL for the given column.
                    // the main reason to do this is that, folks who uses postgres
                    // with other analytical tools won't find any disturbance.
                    // eg: metabase.
                    return Err(QueryRewriterError::RewriteExpr {
                        alias_name: object_name.value.clone(),
                    });
                }
            }
            Expr::CompoundIdentifier(identifiers) => {
                let (table_name, column_name) = get_column_from_idents(&identifiers);
                if !state.is_allowed_column(&table_name, &column_name) {
                    return Err(QueryRewriterError::RewriteExpr {
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
                        return Err(QueryRewriterError::RewriteExpr {
                            alias_name: String::from("case"),
                        });
                    }
                }
                for result in results {
                    if let Err(_) = self.handle_expr(state, result) {
                        return Err(QueryRewriterError::RewriteExpr {
                            alias_name: String::from("case"),
                        });
                    }
                }
                if let Some(else_result) = else_result {
                    if let Err(_) = self.handle_expr(state, else_result) {
                        return Err(QueryRewriterError::RewriteExpr {
                            alias_name: String::from("case"),
                        });
                    }
                }
            }
            Expr::Cast { expr, .. } => self.handle_expr(state, expr)?,
            Expr::TryCast { expr, .. } => self.handle_expr(state, expr)?,
            Expr::Extract { expr, .. } => {
                if let Err(_) = self.handle_expr(state, expr) {
                    return Err(QueryRewriterError::RewriteExpr {
                        alias_name: String::from("date_part"),
                    });
                }
            } // date_part
            Expr::Collate { expr, .. } => {
                if let Err(_) = self.handle_expr(state, expr) {
                    return Err(QueryRewriterError::RewriteExpr {
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
                        return Err(QueryRewriterError::RewriteExpr {
                            alias_name: default_column_name.to_string(),
                        });
                    }
                }
                if let Err(_) = self.handle_expr(state, expr) {
                    return Err(QueryRewriterError::RewriteExpr {
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
                    return Err(QueryRewriterError::RewriteExpr {
                        alias_name: "substring".to_string(),
                    });
                }
                if let Some(from) = substring_from {
                    if let Err(_) = self.handle_expr(state, from) {
                        return Err(QueryRewriterError::RewriteExpr {
                            alias_name: "substring".to_string(),
                        });
                    }
                }
                if let Some(expr) = substring_for {
                    if let Err(_) = self.handle_expr(state, expr) {
                        return Err(QueryRewriterError::RewriteExpr {
                            alias_name: "substring".to_string(),
                        });
                    }
                }
            }
            Expr::Value(_) | Expr::TypedString { .. } => {
                // things that needs no evaluation.
            }
            Expr::BinaryOp { left, right, .. } => {
                if let Err(_) = self.handle_expr(state, left) {
                    *left = Box::new(Expr::Value(Value::Null));
                };
                if let Err(_) = self.handle_expr(state, right) {
                    *right = Box::new(Expr::Value(Value::Null));
                }
            }
            Expr::UnaryOp { expr, .. } => {
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
            Expr::MapAccess { .. } => {
                // map access needs to be handled.
            }
            Expr::TableColumnAccess { .. } => {
                // table coulumn needs to be handled.
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
    use crate::sql::rule_engine::HardRuleEngine;
    use serde::Deserialize;
    use serde_json;
    use sqlparser::dialect::PostgreSqlDialect;
    use sqlparser::parser::Parser;
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

    fn assert_rewriter<T: RuleEngine + Clone>(
        rewriter: &mut QueryRewriter<T>,
        state: Ctx,
        input: &'static str,
        output: &'static str,
    ) {
        let dialect = PostgreSqlDialect {};
        let mut statements = Parser::parse_sql(&dialect, input).unwrap();
        for statement in &mut statements {
            rewriter.rewrite(statement, &state).unwrap();
        }
        assert_eq!(output, format!("{}", statements[0]))
    }

    fn assert_error<T: RuleEngine + Clone>(
        rewriter: &mut QueryRewriter<T>,
        state: Ctx,
        input: &'static str,
        err: QueryRewriterError,
    ) {
        let dialect = PostgreSqlDialect {};
        let mut statements = Parser::parse_sql(&dialect, input).unwrap();
        for statement in &mut statements {
            let rewriter_err = rewriter.rewrite(statement, &state).unwrap_err();
            assert_eq!(rewriter_err, err)
        }
    }

    #[test]
    fn test_for_output() {
        let dialect = PostgreSqlDialect {};
        let statements = Parser::parse_sql(
            &dialect,
            r#"UPDATE accounts SET (contact_last_name, contact_first_name) =
            (SELECT last_name, first_name FROM salesmen
             WHERE salesmen.id = accounts.sales_id);"#,
        )
        .unwrap();
        println!("{:?}", statements[0])
    }
    #[test]
    fn basic_select() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "select * from kids",
            "SELECT NULL AS \"phone\", id, name, address FROM kids",
        );

        assert_rewriter(
            &mut rewriter,
            state,
            "SELECT * FROM public.kids",
            "SELECT NULL AS \"public.kids.phone\", public.kids.id, public.kids.name, public.kids.address FROM public.kids",
        );
    }

    #[test]
    fn test_simple_join() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
        };

        let state = Ctx::new(HashMap::from([
            (
                String::from("weather"),
                cowvec!("city", "temp_lo", "temp_hi", "prcp", "date"),
            ),
            (String::from("cities"), cowvec!("name", "location")),
        ]));

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(&mut rewriter, state, "SELECT w.city, w.temp_lo, w.temp_hi,
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
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state,
            "WITH DUMMY AS (SELECT * FROM kids LIMIT 1)
            SELECT * FROM DUMMY",
            "WITH DUMMY AS (SELECT NULL AS \"phone\", id, name, address FROM kids LIMIT 1) SELECT * FROM DUMMY",
        );
    }

    #[test]
    fn test_subquery() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "select * from (select * from public.kids) as nested",
            "SELECT * FROM (SELECT NULL AS \"public.kids.phone\", public.kids.id, public.kids.name, public.kids.address FROM public.kids) AS nested",
        );
        assert_rewriter(
            &mut rewriter,
            state,
            "select * from (with dummy as (select * from kids) select * from dummy)as nested limit 1;",
            "SELECT * FROM (WITH dummy AS (SELECT NULL AS \"phone\", id, name, address FROM kids) SELECT * FROM dummy) AS nested LIMIT 1",
        );
    }

    #[test]
    fn test_union() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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
        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "select * from kids UNION select * from public.kids2",
            "SELECT NULL AS \"phone\", id, name, address FROM kids UNION SELECT * FROM public.kids2",
        );

        assert_rewriter(
            &mut rewriter,
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
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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
            (
                String::from("public.kids"),
                vec![
                    String::from("phone"),
                    String::from("id"),
                    String::from("name"),
                    String::from("address"),
                ],
            ),
        ]));
        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT *
            FROM weather INNER JOIN public.cities ON (weather.city = public.cities.name);",
            "SELECT * FROM weather JOIN public.cities ON (weather.city = public.cities.name)",
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT w.city, w.temp_lo, w.temp_hi,
                         w.prcp, cities.location
                      FROM weather as w, cities
                      WHERE cities.name = w.city;",
            "SELECT w.city, w.temp_lo, w.temp_hi, w.prcp, cities.location FROM weather AS w, cities WHERE cities.name = w.city",
        );

        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "select * from transactions join kids on transactions.kid_id = kids.id limit 10;",
            r#"SELECT NULL AS "phone", id, name, address, transactions.* FROM transactions JOIN kids ON transactions.kid_id = kids.id LIMIT 10"#,
        );
    }

    #[test]
    fn test_projection_expr() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([
                (String::from("kids"), vec![String::from("phone")]),
                (String::from("kids2"), vec![String::from("phone")]),
            ]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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
        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT *, (select sum(temp_hi) from weather) as temp_hi
            FROM cities",
            "SELECT *, (SELECT sum(temp_hi) FROM weather) AS temp_hi FROM cities",
        );
    }

    #[test]
    fn test_wildcard_qualified_wildcard() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state,
            "select kids.* from kids",
            "SELECT NULL AS \"kids.phone\", kids.id, kids.name, kids.address FROM kids",
        );
    }

    #[test]
    fn test_expr() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(&mut rewriter, state.clone(), "SELECT 1", "SELECT 1");

        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT count(*) from kids",
            "SELECT count(*) FROM kids",
        );

        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT CASE when address > 10 then address else 0 end from kids",
            "SELECT CASE WHEN address > 10 THEN address ELSE 0 END FROM kids",
        );

        assert_rewriter(
            &mut rewriter,
            state.clone(),
            r#"SELECT name < address COLLATE "de_DE" FROM kids"#,
            r#"SELECT name < address COLLATE "de_DE" FROM kids"#,
        );

        assert_rewriter(
            &mut rewriter,
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
            &mut rewriter,
            state.clone(),
            r#"SELECT CAST(name as INTEGER), EXTRACT(month from id) FROM kids"#,
            r#"SELECT CAST(name AS INT), EXTRACT(MONTH FROM id) FROM kids"#,
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            r#"SELECT CAST(name as INTEGER), EXTRACT(month from phone) FROM kids"#,
            r#"SELECT CAST(name AS INT), NULL AS "date_part" FROM kids"#,
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            r#"SELECT CAST(phone as INTEGER), EXTRACT(month from id) FROM kids"#,
            r#"SELECT NULL AS "phone", EXTRACT(MONTH FROM id) FROM kids"#,
        );
    }

    #[test]
    fn test_rewrite_null() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT id, phone from kids",
            "SELECT id, NULL AS \"phone\" FROM kids",
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT id, phone AS demophone from kids",
            "SELECT id, NULL AS demophone FROM kids",
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "SELECT SUM(phone) from kids",
            "SELECT SUM(NULL) FROM kids",
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            r#"SELECT case phone when 10 then "hello" end from kids"#,
            "SELECT NULL AS \"phone\" FROM kids",
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            r#"SELECT SUM(case phone when 10 then 1 end) from kids"#,
            "SELECT SUM(NULL) FROM kids",
        );
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            r#"SELECT substring(phone from 10) from kids"#,
            "SELECT NULL AS \"substring\" FROM kids",
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
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
        };

        let state = Ctx::new(get_table_info());

        let mut rewriter = QueryRewriter::new(
            rule_engine,
            vec!["public".to_string(), "pg_catalog".to_string()],
        );
        assert_rewriter(&mut rewriter, state.clone(), "SELECT DISTINCT t.typname FROM pg_enum e LEFT JOIN pg_type t ON t.oid = e.enumtypid", "SELECT DISTINCT t.typname FROM pg_enum AS e LEFT JOIN pg_type AS t ON t.oid = e.enumtypid");
    }

    #[test]
    fn test_insert() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("id")])]),
            insert_allowed: true,
            insert_allowed_attributes: HashMap::from([(
                String::from("kids"),
                vec![String::from("id")],
            )]),
            update_allowed: false,
            ..Default::default()
        };
        let mut rewriter = QueryRewriter::new(rule_engine, vec![]);
        let state = Ctx::new(get_table_info());
        assert_error(
            &mut rewriter,
            state,
            "INSERT INTO KIDS(phone) values('9843421696')",
            QueryRewriterError::UnAuthorizedInsert,
        );
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("phone")])]),
            insert_allowed: true,
            update_allowed: false,
            insert_allowed_attributes: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
            ..Default::default()
        };
        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        let state = Ctx::new(get_table_info());
        assert_rewriter(
            &mut rewriter,
            state,
            "INSERT INTO kids(phone) values('9843421696')",
            "INSERT INTO kids (phone) VALUES ('9843421696')",
        )
    }

    #[test]
    fn test_update() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("id")])]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
        };
        let mut rewriter = QueryRewriter::new(rule_engine, vec![]);
        let state = Ctx::new(get_table_info());
        assert_error(
            &mut rewriter,
            state,
            "UPDATE kids SET id = 'Dramatic' WHERE phone = '9843421696';",
            QueryRewriterError::UnAuthorizedUpdate,
        );

        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("id")])]),
            insert_allowed: false,
            update_allowed: true,
            ..Default::default()
        };
        let mut rewriter = QueryRewriter::new(rule_engine, vec![]);
        let state = Ctx::new(get_table_info());
        assert_error(
            &mut rewriter,
            state,
            "UPDATE kids SET id = 'Dramatic' WHERE phone = '9843421696';",
            QueryRewriterError::UnAuthorizedUpdate,
        );

        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![String::from("id")])]),
            insert_allowed: false,
            update_allowed: true,
            update_allowed_attributes: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("mobile_number")],
            )]),
            ..Default::default()
        };
        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        let state = Ctx::new(get_table_info());
        assert_rewriter(
            &mut rewriter,
            state,
            "UPDATE public.kids SET mobile_number = 'Dramatic' WHERE mobile_number = '9843421696';",
            "UPDATE public.kids SET mobile_number = 'Dramatic' WHERE mobile_number = '9843421696'",
        );
    }

    #[test]
    fn test_protected_table() {
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(String::from("kids"), vec![])]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
        };

        let state = Ctx::new(HashMap::from([
            (
                String::from("kids"),
                vec![
                    String::from("phone"),
                    String::from("id"),
                    String::from("name"),
                    String::from("address"),
                ],
            ),
            (
                String::from("transactions"),
                vec![
                    String::from("merchant_name"),
                    String::from("kid_id"),
                    String::from("id"),
                    String::from("amount"),
                ],
            ),
        ]));

        let mut rewriter = QueryRewriter::new(rule_engine.clone(), vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state.clone(),
            "select * from kids",
            "SELECT NULL AS \"phone\", NULL AS \"id\", NULL AS \"name\", NULL AS \"address\" FROM kids",
        );

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        assert_rewriter(
            &mut rewriter,
            state,
            "select * from kids join transactions on transactions.kid_id = kids.id",
            "SELECT NULL AS \"phone\", NULL AS \"id\", NULL AS \"name\", NULL AS \"address\", transactions.* FROM kids JOIN transactions ON transactions.kid_id = kids.id",
        );
    }

    #[test]
    fn test_metrics(){
        let rule_engine = HardRuleEngine {
            protected_columns: HashMap::from([(
                String::from("public.kids"),
                vec![String::from("phone")],
            )]),
            insert_allowed: false,
            update_allowed: false,
            ..Default::default()
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

        let mut rewriter = QueryRewriter::new(rule_engine, vec!["public".to_string()]);
        let dialect = PostgreSqlDialect {};
        let mut statements = Parser::parse_sql(&dialect, "SELECT id, phone from kids").unwrap();
        let metrics = rewriter.rewrite(&mut statements[0], &state).unwrap();
        assert!(metrics.len() == 1);
        let mut set = HashSet::new();
        set.insert("id".to_string());
        assert_eq!(metrics.get("kids").unwrap(), &set);
    }
}
