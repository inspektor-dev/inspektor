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

use sqlparser::parser::ParserError;
use std;
use std::fmt::{Display, Formatter};
use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum QueryRewriterError {
    PaserError(#[from] ParserError),
    UnAuthorizedColumn((Option<String>, String)),
    InvalidReference(String),
    FromNeedAlias,
    Error(String),
    RewriteExpr { alias_name: String },
}

impl Display for QueryRewriterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryRewriterError::PaserError(e) => write!(f, "{:?}", e),
            QueryRewriterError::UnAuthorizedColumn((table, column)) => {
                if let Some(table) = table {
                    if column == "" {
                        return write!(f, "unauthorized  table {:?}", table);
                    }
                    return write!(
                        f,
                        "unauthorized column {:?} for the table {:?}",
                        column, table
                    );
                }
                write!(f, "unauthorized column {:?}", column)
            }
            QueryRewriterError::InvalidReference(table) => {
                write!(
                    f,
                    "invalid reference to FROM clause entry for table {:?}",
                    table
                )
            }
            QueryRewriterError::FromNeedAlias => {
                write!(f, "from need alias")
            }
            QueryRewriterError::Error(msg) => {
                write!(f, "{}", msg)
            }
            QueryRewriterError::RewriteExpr { alias_name } => {
                write!(
                    f,
                    "rewrite expression with null value with alias name {}",
                    alias_name
                )
            }
        }
    }
}
