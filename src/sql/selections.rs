use std::borrow::Cow;
use std::collections::HashMap;

// validation state contains all the required metadata that will be used for
// validating the selections.
#[derive(Debug, Default, Clone)]
pub struct ValidationState<'a> {
    // global_allowed_selections holds all the allowed column name for the entire query statement.
    // eg: cte and processed sub query.
    global_allowed_selections: HashMap<Cow<'a, str>, Vec<Cow<'a, str>>>,
    // allowed_selection hold the selections that is allowed only for the current execution block.
    allowed_selections: HashMap<Cow<'a, str>, Vec<Cow<'a, str>>>,
    //table info holds the informaion about the table.
    table_info: HashMap<Cow<'a, str>, Vec<Cow<'a, str>>>,
}

impl<'a> ValidationState<'a> {
    pub fn new(table_info: HashMap<Cow<'a, str>, Vec<Cow<'a, str>>>) -> ValidationState {
        let mut state = ValidationState::default();
        state.table_info = table_info;
        state
    }
    pub fn insert_allowed_columns(
        &mut self,
        table_name: Cow<'a, str>,
        mut columns: Vec<Cow<'a, str>>,
    ) {
        self.allowed_selections.insert(table_name, columns);
    }

    pub fn get_columns(&self, table_name: &Cow<'a, str>) -> Option<Vec<Cow<'a, str>>> {
        if let Some(columns) = self.table_info.get(table_name) {
            return Some(columns.clone());
        }
        None
    }

    pub fn get_default_table(&self) -> Option<&Cow<'a, str>> {
        if self.allowed_selections.len() == 1 {
            for (key, _) in &self.allowed_selections {
                return Some(key);
            }
        }
        None
    }

    pub fn is_allowed_column(&self, table_name: &Cow<'a, str>, column: &String) -> bool {
        if let Some(columns) = self.allowed_selections.get(table_name) {
            return !columns
                .iter()
                .position(|allowed_column| *allowed_column == *column)
                .is_none();
        }
        return false;
    }

    pub fn get_allowed_columns(&self, table_name: &Cow<'a, str>) -> Option<Vec<String>> {
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

    pub fn merge_table_info(&mut self, table_name: Cow<'a, str>, state: ValidationState<'a>) {
        for (_, val) in state.allowed_selections {
            self.table_info.insert(table_name.clone(), val);
        }
    }

    pub fn merge_allowed_selections(&mut self, table_name: Cow<'a, str>, state: ValidationState<'a>) {
        for (_, val) in state.allowed_selections {
            self.allowed_selections.insert(table_name.clone(), val);
        }
    }

    pub fn merge_state(&mut self, state: ValidationState<'a>) {
        for (key, val) in state.allowed_selections{
            self.allowed_selections.insert(key, val);
        }
        for (key, val) in state.table_info{
            self.table_info.insert(key, val);
        }
    }
}
