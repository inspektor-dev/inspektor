// Copyright 2021 Balaji (rbalajis25@gmail.com)
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


