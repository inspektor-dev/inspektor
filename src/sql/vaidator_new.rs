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

use crate::sql::error::InspektorSqlError;
use crate::sql::rule_engine::RuleEngine;
use crate::sql::selections::ValidationState;
use futures::StreamExt;
use protobuf::ProtobufEnum;
use sqlparser::ast::{Expr, Ident, Query, Select, SelectItem, SetExpr, Statement, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use std::borrow::Cow;
pub struct QueryRewriter<'a> {
    rule_engine: RuleEngine<'a>,
}

impl<'a> QueryRewriter<'a> {
    fn new<'s>(rule_engine: RuleEngine<'s>) -> Result<QueryRewriter<'s>, InspektorSqlError> {
        Ok(QueryRewriter {
            rule_engine: rule_engine,
        })
    }

    fn validate(
        &self,
        statements: &mut Vec<Statement>,
        state: ValidationState<'a>,
    ) -> Result<(), InspektorSqlError> {
        for statement in statements {
            match statement {
                Statement::Query(query) => {
                    self.handle_query(query, &state)?;
                }
                _ => {
                    unreachable!("unknown statement {:?}", statement);
                }
            }
        }
        Ok(())
    }

    fn handle_query<'s>(
        &self,
        query: &mut Query,
        state: &ValidationState<'s>,
    ) -> Result<ValidationState<'s>, InspektorSqlError> {
        let mut local_state = state.clone();
        if let Some(with) = &mut query.with {
            for cte in &mut with.cte_tables {
                let cte_state = self.handle_query(&mut cte.query, state)?;
                // cte state are pushed to the underlying table so let's merge allowed columns
                // to table info.
                let table_name = cte.alias.name.value.clone();
                local_state.merge_table_info(Cow::from(table_name), cte_state);
            }
        }
        // we'll evaulate the body first because that is the data which will be retrived for the
        // subsequent query evaluation.
        match &mut query.body {
            SetExpr::Query(query) => return self.handle_query(query, &local_state),
            SetExpr::Select(select) => return self.handle_select(select, &local_state),
            _ => {
                unreachable!("not handled statement {:?}", query.body);
            }
        }
        Ok(local_state)
    }

    fn handle_select<'s>(
        &self,
        select: &mut Select,
        state: &ValidationState<'s>,
    ) -> Result<ValidationState<'s>, InspektorSqlError> {
        let mut local_state = state.clone();
        for from in &mut select.from {
            match &mut from.relation {
                TableFactor::Table {
                    name,
                    alias,
                    args,
                    with_hints,
                } => {
                    let mut table_name = Cow::Owned(name.0[0].value.clone());
                    // if the given table is protected table then we should throw error.
                    if self.rule_engine.is_table_protected(&table_name) {
                        return Err(InspektorSqlError::UnAuthorizedColumn((
                            name.0[0].value.clone(),
                            "".to_string(),
                        )));
                    }
                    let table_columns = match state.get_columns(&table_name) {
                        Some(columns) => columns,
                        _ => unreachable!("unable to get columns name for the table {:?}", name),
                    };
                    let mut allowed_columns = self
                        .rule_engine
                        .get_allowed_columns(&table_name, table_columns);
                    if let Some(alias) = alias {
                        table_name = Cow::Owned(alias.name.value.clone())
                    }
                    let allowed_columns = allowed_columns
                        .iter()
                        .map(|c| Cow::Owned(c.to_string()))
                        .collect::<Vec<Cow<'_, str>>>();
                    local_state.insert_allowed_columns(table_name, allowed_columns);
                }
                TableFactor::Derived {
                    lateral,
                    subquery,
                    alias,
                } => {
                    // we have a subquery now.
                    let derived_state = self.handle_query(subquery, &local_state)?;
                    // TODO: check whether it's needs to be merged with local state.
                }
                _ => {
                    unreachable!("not handled statement {:?}", select.from);
                }
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

    fn handle_selection(
        &self,
        state: &ValidationState<'a>,
        selection: &mut SelectItem,
    ) -> Result<Vec<SelectItem>, InspektorSqlError> {
        match selection {
            SelectItem::UnnamedExpr(expr) => {
                self.handle_expr(state, expr)?;
                return Ok(vec![SelectItem::UnnamedExpr(expr.clone())]);
            }
            SelectItem::Wildcard => {
                let table = state.get_default_table().unwrap();
                return Ok(state
                    .get_allowed_columns(table)
                    .unwrap()
                    .iter()
                    .map(|c| SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(c))))
                    .collect());
            }
            _ => unreachable!("unknown expr {} {:?}", selection, selection),
        }
    }

    fn handle_expr(
        &self,
        state: &ValidationState<'a>,
        expr: &mut Expr,
    ) -> Result<(), InspektorSqlError> {
        match expr {
            Expr::Identifier(object_name) => {
                // it's a single expression. if we have one table the we pick that.
                let table = match state.get_default_table() {
                    Some(table) => table,
                    _ => unreachable!("didn't get default table"),
                };
                if !state.is_allowed_column(table, &object_name.value) {
                    return Err(InspektorSqlError::UnAuthorizedColumn((
                        table.to_string(),
                        object_name.value.clone(),
                    )));
                }
            }
            Expr::CompoundIdentifier(identifiers) => {
                let alias_name = &identifiers[0].value;
                let column_name = &identifiers[1].value;
                if !state.is_allowed_column(&Cow::Borrowed(alias_name), column_name) {
                    return Err(InspektorSqlError::UnAuthorizedColumn((
                        alias_name.to_string(),
                        column_name.clone(),
                    )));
                }
            }
            _ => unreachable!("unknown expression {} {:?}", expr, expr),
        }
        Ok(())
    }
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
                    temp_vec.push(Cow::from($x));
                )*
                temp_vec
            }
        };
    }

    fn assert_rewriter(
        rewriter: &QueryRewriter,
        state: ValidationState,
        input: &'static str,
        output: &'static str,
    ) {
        let dialect = PostgreSqlDialect {};
        let mut statements = Parser::parse_sql(&dialect, input).unwrap();
        rewriter.validate(&mut statements, state).unwrap();
        assert_eq!(output, format!("{}", statements[0]))
    }
    #[test]
    fn basic_select() {
        let rule_engine = RuleEngine {
            protected_columns: HashMap::from([(Cow::from("kids"), vec![Cow::from("phone")])]),
        };

        let state = ValidationState::new(HashMap::from([(
            Cow::from("kids"),
            vec![
                Cow::from("phone"),
                Cow::from("id"),
                Cow::from("name"),
                Cow::from("address"),
            ],
        )]));

        let rewriter = QueryRewriter::new(rule_engine).unwrap();
        assert_rewriter(
            &rewriter,
            state,
            "select * from kids",
            "SELECT id, name, address FROM kids",
        );
    }

    #[test]
    fn test_simple_join() {
        let rule_engine = RuleEngine {
            protected_columns: HashMap::from([(Cow::from("kids"), vec![Cow::from("phone")])]),
        };

        let state = ValidationState::new(HashMap::from([
            (
                Cow::from("weather"),
                cowvec!("city", "temp_lo", "temp_hi", "prcp", "date"),
            ),
            (Cow::from("cities"), cowvec!("name", "location")),
        ]));

        let rewriter = QueryRewriter::new(rule_engine).unwrap();
        assert_rewriter(&rewriter, state, "SELECT w.city, w.temp_lo, w.temp_hi,
        w.prcp, w.date, cities.location
        FROM weather as w, cities
        WHERE cities.name = w.city;", "SELECT w.city, w.temp_lo, w.temp_hi, w.prcp, w.date, cities.location FROM weather AS w, cities WHERE cities.name = w.city");
    }
}
