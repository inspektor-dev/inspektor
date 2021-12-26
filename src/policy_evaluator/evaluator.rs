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
use crate::sql::rule_engine::HardRuleEngine;
use anyhow::{Error, Result};
use burrego::opa::host_callbacks::DEFAULT_HOST_CALLBACKS;
use burrego::opa::wasm::Evaluator;
use futures::AsyncReadExt;
use log::*;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
/// PolicyEvaluator is used to to evaluate policy decision for all the end user
/// action.
pub struct PolicyEvaluator {
    evaluator: Evaluator,
    entrypoints: HashMap<String, i32>,
}

pub struct PolicyResult {
    pub allow: bool,
    pub protected_columns: Vec<String>,
}

impl PolicyResult {
    // to_rule_engine will convert the policy result to sql rule engine.
    pub fn to_rule_engine(self) -> HardRuleEngine {
        let mut inner_protected_column: HashMap<String, Vec<String>> = HashMap::default();
        for column in self.protected_columns {
            // all columns should have 2 dots
            // schema.table.column
            let splits = column.split(".").collect::<Vec<&str>>();
            if splits.len() != 3 {
                continue;
            }
            let table_name = format!("{}.{}", splits[0], splits[1]);
            if let Some(cols) = inner_protected_column.get_mut(&table_name) {
                cols.push(splits[2].to_string());
                continue;
            }
            inner_protected_column.insert(table_name, vec![splits[2].to_string()]);
        }
        HardRuleEngine::from_protected_columns(inner_protected_column)
    }

    pub fn get_protected_tables(&self) -> Vec<(&str, &str)> {
        let mut set: HashSet<(&str, &str)> = HashSet::default();
        for column in &self.protected_columns{
            let splits = column.split(".").collect::<Vec<&str>>();
            if splits.len() != 3 {
                continue;
            }
            set.insert((splits[0], splits[1]));
        }
        set.into_iter().collect::<Vec<(&str, &str)>>()
    }
}

impl PolicyEvaluator {
    // new returns policy evaluator to the caller.
    pub fn new(policy: &Vec<u8>) -> Result<PolicyEvaluator, Error> {
        let mut evaluator = Evaluator::new(
            String::from("inspecktor-policy"),
            policy,
            &DEFAULT_HOST_CALLBACKS,
        )?;
        let mut entrypoints: HashMap<String, i32> = HashMap::default();
        // find all the entrypoint ids. this will used while evaluating policies.
        entrypoints.insert(
            String::from("allow"),
            evaluator.entrypoint_id(&"inspektor/resource/acl/allow")?,
        );
        entrypoints.insert(
            String::from("protected_columns"),
            evaluator.entrypoint_id(&"inspektor/resource/acl/protected_columns")?,
        );
        Ok(PolicyEvaluator {
            evaluator,
            entrypoints,
        })
    }

    // evaluate will evaluate the policy for the given input.
    pub fn evaluate(
        &mut self,
        data_source: &String,
        db_name: &String,
        groups: &Vec<String>,
    ) -> Result<PolicyResult, anyhow::Error> {
        debug!(
            "evaluating policy with data_soruce {:?} db_name {:?} groups {:?}",
            data_source, db_name, groups
        );
        let input = self.get_input_value(data_source, db_name, groups);
        let data = Value::Object(Map::default());

        let allow = self.evaluator.evaluate(
            *self.entrypoints.get(&String::from("allow")).unwrap(),
            &input,
            &data,
        )?;
        let allow = self.get_result(allow);
        let allow = match allow {
            Value::Bool(allow) => allow,
            _ => false,
        };
        if !allow {
            return Ok(PolicyResult {
                allow: false,
                protected_columns: Vec::default(),
            });
        }
        // get protected columns for the user.
        let protected_columns = self.evaluator.evaluate(
            *self
                .entrypoints
                .get(&String::from("protected_columns"))
                .unwrap(),
            &input,
            &data,
        )?;
        let protected_columns = match self.get_result(protected_columns) {
            Value::Array(vals) => vals
                .into_iter()
                .map(|i| match i {
                    Value::String(s) => return s,
                    _ => unreachable!("expected string"),
                })
                .collect::<Vec<String>>(),
            _ => Vec::new(),
        };
        Ok(PolicyResult {
            allow: allow,
            protected_columns: protected_columns,
        })
    }

    // get_result will returns result value from the policy value.
    pub fn get_result(&self, value: serde_json::Value) -> Value {
        if let Value::Array(mut objs) = value {
            let obj = objs.remove(0);
            if let Value::Object(mut obj) = obj {
                return obj.remove("result").unwrap();
            }
            unreachable!("expected object");
        }
        unreachable!("expected array");
    }

    // get_input_value
    fn get_input_value(
        &self,
        data_source: &String,
        db_name: &String,
        groups: &Vec<String>,
    ) -> serde_json::Value {
        let mut object = Map::with_capacity(2);
        object.insert(
            String::from("data_source"),
            Value::String(data_source.clone()),
        );
        object.insert(
            String::from("groups"),
            Value::Array(
                groups
                    .iter()
                    .map(|i| Value::String(i.clone()))
                    .collect::<Vec<Value>>(),
            ),
        );
        object.insert(String::from("db_name"), Value::String(db_name.clone()));
        Value::Object(object)
    }
}

#[cfg(test)]
mod tests {
    use super::PolicyEvaluator;
    use std::env;
    use std::fs;

    #[test]
    fn test_evaluator() {
        let path = env::current_dir().unwrap();
        let policy = fs::read(path.join("src/policy_evaluator/policy.wasm")).unwrap();
        let mut evaluator = PolicyEvaluator::new(&policy).unwrap();
        let result = evaluator
            .evaluate(
                &String::from("postgres_production"),
                &String::from("inspektor"),
                &vec![String::from("dev"), String::from("admin")],
            )
            .unwrap();
        assert_eq!(result.allow, true);
        assert_eq!(
            result.protected_columns,
            vec![String::from("public.data_sources.side_car_token")]
        );
    }
}
