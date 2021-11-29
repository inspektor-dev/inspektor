use sqlparser::parser::ParserError;
use std;
use std::fmt::{Display, Formatter};
use thiserror::Error;
#[derive(Error, Debug)]
pub enum InspektorSqlError {
    PaserError(#[from] ParserError),
    UnAuthorizedColumn((String, String)),
    InvalidReference(String),
    FromNeedAlias
}

impl Display for InspektorSqlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InspektorSqlError::PaserError(e) => write!(f, "{:?}", e),
            InspektorSqlError::UnAuthorizedColumn((table, column)) => write!(
                f,
                "unauthorized column {:?} for the table {:?}",
                column, table
            ),
            InspektorSqlError::InvalidReference(table) => {
                write!(f, "invalid reference to FROM clause entry for table {:?}", table)
            }
            InspektorSqlError::FromNeedAlias => {
                write!(f, "from need alias")
            }
        }
    }
}
