use std::collections::HashMap;
use std::borrow::Cow;
#[derive(Debug, Default)]
pub struct RuleEngine<'a>{
    pub (crate) protected_columns: HashMap<Cow<'a, str>, Vec<Cow<'a, str>>> 
}

impl <'a>RuleEngine<'a> {
    pub fn is_table_protected(&self,  table_name: &Cow<'a, str>) ->  bool{
        if let Some(protected_columns) = self.protected_columns.get(table_name){
            return protected_columns.len() == 0
        }
        return false
    }

    pub fn get_allowed_columns(&self, table_name: &Cow<'a, str>, columns: Vec<Cow<'a, str>>) -> Vec<Cow<'a, str>> {
        if let Some(protected_columns) = self.protected_columns.get(table_name){
            return columns.iter().filter(|column| {
                protected_columns.iter().position(|protected_column| protected_column == *column).is_none()
            }).map(|c| c.clone()).collect();
        }
        return columns;
    }
}


