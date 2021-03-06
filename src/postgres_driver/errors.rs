use crate::sql::error::QueryRewriterError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecoderError {
    #[error("inspektor only support postgres version 3")]
    UnsupporedVersion,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("io error")]
    IoErr(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum ProtocolHandlerError {
    #[error("policy rejected the connection")]
    PolicyRejected,
    #[error("unauthorized insert")]
    UnathorizedInsert,
    #[error("unable to parse the query")]
    ErrParsingQuery,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("{0}")]
    RewriterError(#[from] QueryRewriterError),
}
