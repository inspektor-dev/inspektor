use sqlparser::ast::{Expr, ObjectName, SelectItem};
use sqlparser::ast::{Ident, Value};
use std::collections::{HashMap, HashSet};

// validation state contains all the required metadata that will be used for
// validating the selections.
#[derive(Debug, Default, Clone)]
pub struct Ctx {
    //table info holds the informaion about the table.
    table_info: HashMap<String, Vec<String>>,
    // protected_columns have the details of tables and their protected columns.
    protected_columns: HashMap<String, Vec<String>>,
    // from contains the table names of the current selection/
    from: HashSet<String>,
}

impl Ctx {
    pub fn new(table_info: HashMap<String, Vec<String>>) -> Ctx {
        let mut state = Ctx::default();
        state.table_info = table_info;
        state
    }

    // is_allowed_column will tell the given column in allowed in the given table or not.
    pub fn is_allowed_column(&self, table_name: &String, column: &String) -> bool {
        if let Some(columns) = self.protected_columns.get(table_name) {
            return columns
                .iter()
                .position(|protectec_columns| *protectec_columns == *column)
                .is_none();
        }
        return true;
    }

    // is_allowed_column_ident will tell the given column is allowed or not.
    pub fn is_allowed_column_ident(
        &self,
        column: &String,
        metrics: &mut HashMap<String, HashSet<String>>,
    ) -> bool {
        let froms = self.from.clone().into_iter().collect::<Vec<String>>();
        for from in &froms {
            if let Some(protected_columns) = self.protected_columns.get(from) {
                match protected_columns.iter().position(|col| *col == *column) {
                    Some(_) => return false,
                    None => {
                        if let Some(properties) = metrics.get_mut(from) {
                            properties.insert(column.clone());
                            continue;
                        }
                        let mut set = HashSet::new();
                        set.insert(column.clone());
                        metrics.insert(from.clone(), set);
                        continue;
                    }
                }
            }
        }
        return true;
    }

    // merge_state will merget the incoming state with the current state.
    pub fn merge_state(&mut self, state: Ctx) {
        for (key, val) in state.protected_columns {
            self.protected_columns.insert(key, val);
        }
        for (key, val) in state.table_info {
            self.table_info.insert(key, val);
        }
        for val in state.from {
            self.from.insert(val);
        }
    }

    // build_allowed_column_expr will returns all the allowed selection for the
    // the current state.
    pub fn build_allowed_column_expr(
        &self,
        metrics: &mut HashMap<String, HashSet<String>>,
    ) -> Vec<SelectItem> {
        let mut selections = vec![];
        let mut wildcard = true;
        let mut froms = self.from.clone().into_iter().collect::<Vec<String>>();
        froms.sort();
        for from in froms {
            let exprs = self.column_expr_for_table(&from, false, metrics);
            if exprs.len() == 0 {
                if let Some(properties) = metrics.get_mut(&from) {
                    properties.insert("*".to_string());
                } else {
                    let mut properties = HashSet::new();
                    properties.insert("*".to_string());
                    metrics.insert(from.clone(), properties);
                }
                selections.push(SelectItem::QualifiedWildcard(ObjectName(vec![Ident::new(
                    from,
                )])));
                continue;
            }
            wildcard = false;
            selections.extend_from_slice(&exprs[..]);
        }
        if wildcard {
            return vec![SelectItem::Wildcard];
        }
        return selections;
    }

    // column_expr_for_table returns accepted columne expression for the given table.
    pub fn column_expr_for_table(
        &self,
        table_name: &String,
        prefix_table_name: bool,
        metrics: &mut HashMap<String, HashSet<String>>,
    ) -> Vec<SelectItem> {
        // should_prefix will determine whether we should prefix
        // table name as column name.
        let mut should_prefix = prefix_table_name;
        let splits = table_name.split(".").collect::<Vec<&str>>();
        if splits.len() > 1 {
            should_prefix = true;
        }
        let mut selections = vec![];
        if let Some(protected_columns) = self.protected_columns.get(table_name) {
            let protected_columns_set = protected_columns.iter().collect::<HashSet<&String>>();
            let table_columns = self.table_info.get(table_name).unwrap();
            for col in table_columns {
                if protected_columns_set.contains(col) || protected_columns.len() == 0 {
                    let column_name = match should_prefix {
                        true => format!("{}.{}", table_name, col),
                        false => format!("{}", col),
                    };
                    selections.push(SelectItem::ExprWithAlias {
                        expr: Expr::Value(Value::Null),
                        alias: Ident {
                            value: column_name,
                            quote_style: Some('"'),
                        },
                    });
                    continue;
                }
                if let Some(properties) = metrics.get_mut(table_name) {
                    properties.insert(col.clone());
                } else {
                    let mut properties = HashSet::new();
                    properties.insert(col.clone());
                    metrics.insert(table_name.clone(), properties);
                }
                if should_prefix {
                    selections.push(SelectItem::UnnamedExpr(Expr::CompoundIdentifier(vec![
                        Ident::new(table_name.to_string()),
                        Ident::new(col.to_string()),
                    ])));
                    continue;
                }
                selections.push(SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(
                    col.to_string(),
                ))));
            }
        }
        return selections;
    }

    // memorize_protected_columns insert protected column to the state.
    pub fn memorize_protected_columns(
        &mut self,
        table_name: String,
        protected_columns: Vec<String>,
    ) {
        self.protected_columns.insert(table_name, protected_columns);
    }

    // get_protected_columns returns protected columns for the given table.
    pub fn get_protected_columns(&self, table_name: &String) -> Option<Vec<String>> {
        if let Some(protected_columns) = self.protected_columns.get(table_name) {
            return Some(
                protected_columns
                    .iter()
                    .map(|c| c.clone())
                    .collect::<Vec<String>>(),
            );
        }
        return None;
    }

    // overwrite_table_info will rewrite the src table name with the given alias name.
    pub fn overwrite_table_info(&mut self, table_name: &String, alias: String) {
        if let Some(columns) = self.table_info.get(table_name) {
            self.table_info.insert(alias, columns.clone());
            return;
        }
        self.table_info.insert(alias, vec![]);
    }

    // add_from_src inserts from table
    pub fn add_from_src(&mut self, table_name: String) {
        self.from.insert(table_name);
    }
}
