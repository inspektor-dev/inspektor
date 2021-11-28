use anyhow::{Error, Result};
use futures::StreamExt;
use md5::digest::generic_array::typenum::Len;
use protobuf::ProtobufEnum;
use sqlparser::ast::{
    Expr, Ident, Query, Select, SelectItem, SetExpr, Statement, TableFactor, TableWithJoins,
};
use crate::sql::error::InspektorSqlError;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use std::collections::HashMap;

pub struct TableState {
    alias: Option<String>,
    table_name: String,
    allowed_rows: Vec<String>,
}

#[derive(Default)]
pub struct Rule {
    protected_fields: HashMap<String, Vec<String>>,
    limit: Option<i32>,
}

#[derive(Default)]
pub struct QueryValidator {
    table_info: HashMap<String, Vec<String>>,
    rule: Rule,
}

impl QueryValidator {
    pub fn add_table_info(&mut self, table_name: String, columns: Vec<String>) {
        self.table_info.insert(table_name, columns);
    }

    pub fn validate_query(&self,mut allowed_projections: HashMap<String, Vec<String>>, query: &mut Box<Query>) -> Result<HashMap<String, Vec<String>>, InspektorSqlError> {
        // query may have cte, so it's necessary to build allowed projection because upcoming select
        // may come from cte.
        if let Some(with) = &mut query.with{
            
            for cte in &mut with.cte_tables{
                let mut cte_allowed_projections = HashMap::default();
                let mut query = Box::new(cte.query.clone());
                let cte_projections = self.validate_query(cte_allowed_projections, &mut query)?;
                for (_key, value) in cte_projections{
                    allowed_projections.insert(cte.alias.name.value.clone(), value);
                }
                cte.query = *query;    
            }
        }
        let mut result_projections = HashMap::default();
        match &mut query.body {
            SetExpr::Select(select) => {
                // it's select query check whether the table is protected or not.
                result_projections = self.handle_select(allowed_projections, select)?;
            }
            _ => {
                unreachable!("not implemented")
            }
        }
        Ok(result_projections)
    }

    pub fn handle_select(&self,mut allowed_projections: HashMap<String, Vec<String>>, select: &mut Select) -> Result<HashMap<String, Vec<String>>, InspektorSqlError> {
        // check from is protected or not.
        // for now we just support only one select.
        let mut default_table = String::from("");
        // let's build possible selection for the soruce.
        for table in &mut select.from {
            match &table.relation {
                TableFactor::Table {
                    name,
                    alias,
                    args,
                    with_hints,
                } => {
                    // table name can be only one so let's read directly.
                    let mut table_name = &name.0[0].value;
                    // let's figure out all the possible projections for this table.
                    let protected_columns = self.rule.protected_fields.get(table_name);
                    if protected_columns.is_none() {
                        // since there is no protected field for this table name.
                        // let's skip this table.
                        if alias.is_some() {
                            table_name = &alias.as_ref().unwrap().name.value;
                        }
                        default_table = table_name.clone();
                        allowed_projections.insert(table_name.clone(), vec![]);
                        continue;
                    }
                    let protected_columns = protected_columns.unwrap();
                    // we have to determine the projections for this table since
                    // there are protected fileds for this table.
                    let table_columns = self.table_info.get(table_name);
                    if table_columns.is_none() {
                        unreachable!("{:?} doesn't have any table info", table_name);
                    }
                    let table_columns = table_columns.unwrap();
                    // filter out all the allowed columns for this table.
                    let allowed_columns = table_columns
                        .iter()
                        .filter(|column| {
                            protected_columns
                                .iter()
                                .position(|protected_column| protected_column == *column)
                                .is_none()
                        })
                        .map(|column| column.clone())
                        .collect::<Vec<String>>();
                    // replace the table name with alias name if alias exist.
                    if alias.is_some() {
                        table_name = &alias.as_ref().unwrap().name.value;
                    }
                    default_table = table_name.clone();
                    allowed_projections.insert(table_name.clone(), allowed_columns);
                }
                _ => {
                    unreachable!("unknown table relation {:?}", &table.relation)
                }
            }
        }
        // if it's only one table we won't be having combied expr so let's get the default table.
        // let's build all the possible projections for this selection.
        let mut projections = Vec::<SelectItem>::new();
        for projection in &select.projection {
            match projection {
                SelectItem::UnnamedExpr(expr) => {
                    match expr {
                        Expr::Identifier(column_name) => {
                            // typically this'll come only if there is one table so, let's check whether we have only one
                            // table in the allowed projections.
                            assert_eq!(allowed_projections.len(), 1);
                            let allowed_columns = allowed_projections.get(&default_table).unwrap();
                            // check whether this column exist in the allowed column;
                            if allowed_columns
                                .iter()
                                .position(|allowed_column| *allowed_column == *column_name.value)
                                .is_none()
                            {
                                return Err(InspektorSqlError::UnAuthorizedColumn((default_table, column_name.value.clone())));
                            }
                            projections.push(SelectItem::UnnamedExpr(Expr::Identifier(
                                column_name.clone(),
                            )));
                        }
                        Expr::CompoundIdentifier(identifiers) => {
                            // first identifier is alias name.
                            let alias_name = &identifiers[0].value;
                            let column_name = &identifiers[1].value;
                            let allowed_columns = allowed_projections.get(alias_name);
                            if allowed_columns.is_none(){
                                return Err(InspektorSqlError::InvalidReference(alias_name.clone()))
                            }
                            let allowed_columns = allowed_columns.unwrap();
                            // if the allowed column is set to zero then we'll allow all the column
                            // for this table.
                            if allowed_columns.len() !=0 && allowed_columns
                                .iter()
                                .position(|allowed_column| *allowed_column == *column_name)
                                .is_none()
                            {
                                return Err(InspektorSqlError::UnAuthorizedColumn((alias_name.clone(), column_name.clone())));
                            }
                            projections.push(SelectItem::UnnamedExpr(Expr::CompoundIdentifier(
                                identifiers.clone(),
                            )));
                        }
                        _ => {
                            unreachable!("unknown expression {:?}", expr)
                        }
                    }
                }
                SelectItem::Wildcard => {
                    let allowed_columns = allowed_projections.get(&default_table).unwrap();
                    if allowed_columns.len() == 0 {
                        projections.push(SelectItem::UnnamedExpr(Expr::Wildcard));
                        continue;
                    }
                    // if we have some columns to project remove the wild card and push
                    // all the allowed columns.
                    projections.extend(
                        allowed_columns
                            .iter()
                            .map(|column_name| {
                                SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(
                                    column_name.clone(),
                                )))
                            })
                            .collect::<Vec<SelectItem>>(),
                    );
                }
                _ => unreachable!("unknown projection {:?}", projection),
            }
        }
        select.projection = projections;
        // let projections = self.handle_from_table(table_from).unwrap();
        // if projections.is_none() {
        //     unreachable!("protected table")
        // }
        // let projections = projections.unwrap();
        // if projections.len() == 0 {
        //     return;
        // }
        // let projections = projections
        //     .iter()
        //     .map(|column_name| SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(column_name))))
        //     .collect::<Vec<SelectItem>>();
        // select.projection = projections;
        Ok(allowed_projections)
    }

    pub fn handle_from_table(
        &self,
        table: &mut TableWithJoins,
    ) -> Result<Option<Vec<String>>, anyhow::Error> {
        match &table.relation {
            TableFactor::Table {
                name,
                alias,
                args,
                with_hints,
            } => {
                // check table is in protected field.
                let table_name = &name.0[0].value;
                let protected_columns = self.rule.protected_fields.get(table_name);
                if protected_columns.is_none() {
                    return Ok(Some(vec![]));
                }

                let protected_columns = protected_columns.unwrap();
                if protected_columns.len() == 0 {
                    return Ok(Some(vec![]));
                }
                // so we have some protected fields wild card won't work so give all
                // columns for the projections.
                let columns = self.table_info.get(table_name).unwrap();
                // filter out all the protected columns.
                let filtered_columns = columns
                    .iter()
                    .filter(|column| {
                        let position = protected_columns
                            .iter()
                            .position(|protected_columns| protected_columns == *column);
                        position.is_none()
                    })
                    .map(|column| column.clone())
                    .collect::<Vec<String>>();
                return Ok(Some(filtered_columns));
            }
            _ => {
                unreachable!("unreachable table relation")
            }
        }
    }
}

pub fn validate(query: &String, protected_fields: Vec<String>) {
    let dialect = PostgreSqlDialect {};
    let ast = Parser::parse_sql(&dialect, query).unwrap();
    println!("{:#?}", ast[0]);
}

#[cfg(test)]
mod tests {
    use crate::sql::validator;

    use super::*;
    #[test]
    fn test_validate() {
        let query = String::from(
            r#"WITH regional_sales AS (
                SELECT region, SUM(amount) AS total_sales
                FROM orders
                GROUP BY region
             ), top_regions AS (
                SELECT region
                FROM regional_sales
                WHERE total_sales > (SELECT SUM(total_sales)/10 FROM regional_sales)
             )
        SELECT region,
               product,
               SUM(quantity) AS product_units,
               SUM(amount) AS product_sales
        FROM orders
        WHERE region IN (SELECT region FROM top_regions)
        GROUP BY region, product;"#,
        );
        validate(&query, vec![]);
    }

    fn test_query(tests: Vec<(&str, &str)>, validator: QueryValidator) {
        let dialect = PostgreSqlDialect {};
        for (input_query, output_query) in tests {
            let mut statements = Parser::parse_sql(&dialect, &input_query).unwrap();
            let statement = statements.remove(0);
            match statement {
                Statement::Query(mut query) => {
                    let allowed_projections = HashMap::default();
                    validator.validate_query(allowed_projections,&mut query).unwrap();
                    assert_eq!(format!("{}", query), output_query);
                }
                _ => unreachable!("expected query but got different statement"),
            }
        }
    }
    #[test]
    fn test_basic_select() {
        let validator = QueryValidator {
            table_info: HashMap::from([(
                "users".to_string(),
                vec!["id".to_string(), "name".to_string(), "phone".to_string()],
            )]),
            rule: Rule {
                protected_fields: HashMap::from([("users".to_string(), vec!["id".to_string()])]),
                limit: None,
            },
        };
        test_query(
            vec![
                ("SELECT * FROM users", "SELECT name, phone FROM users"),
                ("SELECT * FROM hello", "SELECT * FROM hello"),
            ],
            validator,
        );
    }

    #[test]
    fn test_simple_join() {
        let validator = QueryValidator {
            table_info: HashMap::from([(
                "weathers".to_string(),
                vec![
                    "city".to_string(),
                    "temp_lo".to_string(),
                    "temp_hi".to_string(),
                    "prcp".to_string(),
                    "date".to_string(),
                ],
            ),(
                "cities".to_string(),
                vec![
                    "name".to_string(),
                    "location".to_string(),
                ],
            )]),
            rule: Rule {
                protected_fields: HashMap::from([("users".to_string(), vec!["id".to_string()])]),
                limit: None,
            },
        };
        test_query(
            vec![(
                r#"SELECT w.city, w.temp_lo, w.temp_hi,
                w.prcp, w.date, cities.location
             FROM weather as w, cities
             WHERE cities.name = w.city;"#,
                "SELECT w.city, w.temp_lo, w.temp_hi, w.prcp, w.date, cities.location FROM weather AS w, cities WHERE cities.name = w.city",
            )],
            validator,
        );
    }

    #[test]
    fn test_cte(){
        let validator = QueryValidator {
            table_info: HashMap::from([(
                "kids".to_string(),
                vec![
                    "id".to_string(),
                    "phone".to_string(),
                    "gender".to_string(),
                    "balance".to_string(),
                ],
            )]),
            rule: Rule {
                protected_fields: HashMap::from([("kids".to_string(), vec!["phone".to_string()])]),
                limit: None,
            },
        };
        test_query(
            vec![(
                r#"WITH DUMMY AS (SELECT * FROM kids LIMIT 1)
                SELECT * FROM DUMMY;"#,
                "WITH DUMMY AS (SELECT id, gender, balance FROM kids LIMIT 1) SELECT * FROM DUMMY",
            )],
            validator,
        );
    }
}
