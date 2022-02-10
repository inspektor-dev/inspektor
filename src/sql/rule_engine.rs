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

pub trait RuleEngine {
    fn is_table_protected(&self, table_name: &String) -> bool;
    fn get_allowed_columns(&self, table_name: &String, columns: Vec<String>) -> Vec<String>;
    fn get_protected_columns(&self, table_name: &String) -> Option<Vec<String>>;
    fn is_insert_allowed(&self) -> bool;
    fn is_update_allowed(&self) -> bool;
    fn is_copy_allowed(&self) -> bool;
    fn get_allowed_insert_attributes(&self) -> &HashMap<String, Vec<String>>;
    fn get_allowed_copy_attributes(&self) -> &HashMap<String, Vec<String>>;
    fn get_allowed_update_attributes(&self) -> &HashMap<String, Vec<String>>;
    fn is_protected_column(&self, table_name: &String, column: &String) -> bool;
}

#[derive(Debug, Default, Clone)]

pub struct HardRuleEngine {
    pub protected_columns: HashMap<String, Vec<String>>,
    pub insert_allowed: bool,
    pub update_allowed: bool,
    pub copy_allowed: bool,
    pub copy_allowed_attributes: HashMap<String, Vec<String>>,
    pub insert_allowed_attributes: HashMap<String, Vec<String>>,
    pub update_allowed_attributes: HashMap<String, Vec<String>>
}

impl RuleEngine for HardRuleEngine {
    fn is_table_protected(&self, table_name: &String) -> bool {
        if let Some(protected_columns) = self.protected_columns.get(table_name) {
            return protected_columns.len() == 0;
        }
        return false;
    }

    fn get_allowed_columns(&self, table_name: &String, columns: Vec<String>) -> Vec<String> {
        if let Some(protected_columns) = self.protected_columns.get(table_name) {
            return columns
                .iter()
                .filter(|column| {
                    protected_columns
                        .iter()
                        .position(|protected_column| protected_column == *column)
                        .is_none()
                })
                .map(|c| c.clone())
                .collect();
        }
        return columns;
    }

    fn get_protected_columns(&self, table_name: &String) -> Option<Vec<String>> {
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

    fn is_protected_column(&self, table_name: &String, column: &String) -> bool {
        if let Some(protected_columns) = self.protected_columns.get(table_name) {
            return protected_columns
                .iter()
                .position(|protected_column| *protected_column == *column)
                .is_some();
        }
        return false;
    }

    fn is_insert_allowed(&self) -> bool {
        self.insert_allowed
    }

    fn get_allowed_insert_attributes(&self) -> &HashMap<String, Vec<String>> {
        &self.insert_allowed_attributes
    }

    fn get_allowed_copy_attributes(&self) -> &HashMap<String, Vec<String>> {
        &self.copy_allowed_attributes
    }

    fn get_allowed_update_attributes(&self) -> &HashMap<String, Vec<String>> {
        &self.update_allowed_attributes
    }
    
    fn is_update_allowed(&self) -> bool {
        self.update_allowed
    }

    fn is_copy_allowed(&self) -> bool {
        self.copy_allowed
    }
}

impl HardRuleEngine {
    pub fn from_protected_columns(
        protected_columns: HashMap<String, Vec<String>>,
    ) -> HardRuleEngine {
        HardRuleEngine {
            protected_columns,
            insert_allowed: false,
            update_allowed: false,
            copy_allowed: false,
            copy_allowed_attributes: HashMap::default(),
            update_allowed_attributes: HashMap::default(),
            insert_allowed_attributes: HashMap::default()
        }
    }
}
