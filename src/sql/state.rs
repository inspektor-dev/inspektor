use sqlparser::ast::Ident;
use sqlparser::ast::{Expr, SelectItem};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

// validation state contains all the required metadata that will be used for
// validating the selections.
#[derive(Debug, Default, Clone)]
pub struct ValidationState{
    // global_allowed_selections holds all the allowed column name for the entire query statement.
    // eg: cte and processed sub query.
    global_allowed_selections: HashMap<String, Vec<String>>,
    // allowed_selection hold the selections that is allowed only for the current execution block.
    allowed_selections: BTreeMap<String, Vec<String>>,
    //table info holds the informaion about the table.
    table_info: HashMap<String, Vec<String>>,
}

impl ValidationState {
    pub fn new(table_info: HashMap<String ,Vec<String>>) -> ValidationState {
        let mut state = ValidationState::default();
        state.table_info = table_info;
        state
    }
    pub fn insert_allowed_columns(
        &mut self,
        table_name: String,
        mut columns: Vec<String>,
    ) {
        self.allowed_selections.insert(table_name, columns);
    }

    pub fn  get_columns(&self, table_name: &String) -> Option<Vec<String>> {
        if let Some(columns) = self.table_info.get(table_name) {
            return Some(columns.clone());
        }
        None
    }

    pub fn get_default_table(&self) -> Option<String> {
        if self.allowed_selections.len() == 1 {
            for (key, _) in &self.allowed_selections {
                return Some(key.to_string());
            }
        }
        None
    }

    pub fn is_allowed_column(&self, table_name: &String, column: &String) -> bool {
        if let Some(columns) = self.allowed_selections.get(table_name) {
            return !columns
                .iter()
                .position(|allowed_column| *allowed_column == *column)
                .is_none();
        }
        return false;
    }

    pub fn is_allowed_column_ident(&self, column: &String) -> bool {
        for (_, columns) in &self.allowed_selections {
            match columns
                .iter()
                .position(|allowed_column| *allowed_column == *column)
            {
                Some(_) => return true,
                None => continue,
            };
        }
        return false;
    }

    pub fn get_allowed_columns(&self, table_name: &String) -> Option<Vec<String>> {
        if let Some(columns) = self.allowed_selections.get(table_name) {
            return Some(
                columns
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>(),
            );
        }
        None
    }

    pub fn merge_table_info(&mut self, table_name: String, state: ValidationState) {
        for (_, val) in state.allowed_selections {
            self.table_info.insert(table_name.clone(), val);
        }
    }

    pub fn merge_allowed_selections(
        &mut self,
        table_name: String,
        state: ValidationState,
    ) {
        for (_, val) in state.allowed_selections {
            self.allowed_selections.insert(table_name.clone(), val);
        }
    }

    pub fn merge_state(&mut self, state: ValidationState) {
        for (key, val) in state.allowed_selections {
            self.allowed_selections.insert(key, val);
        }
        for (key, val) in state.table_info {
            self.table_info.insert(key, val);
        }
    }

    pub fn build_allowed_column_expr(&self) -> Vec<SelectItem> {
        let mut selections = Vec::new();
        for (table_name, columns) in &self.allowed_selections {
            if columns.len() == 0 {
                continue;
            }
            for column in columns {
                selections.push(SelectItem::UnnamedExpr(Expr::CompoundIdentifier(vec![
                    Ident::new(table_name.to_string()),
                    Ident::new(column.to_string()),
                ])));
            }
        }
        return selections;
    }

    pub fn column_expr_for_table(&self, table_name: &String) -> Vec<SelectItem> {
        let columns = match self.allowed_selections.get(table_name) {
            Some(columns) => columns,
            None => return Vec::new(),
        };
        let mut result = Vec::with_capacity(columns.len());
        for column in columns {
            result.push(SelectItem::UnnamedExpr(Expr::CompoundIdentifier(vec![
                Ident::new(table_name.to_string()),
                Ident::new(column.to_string()),
            ])))
        }
        result
    }
}
