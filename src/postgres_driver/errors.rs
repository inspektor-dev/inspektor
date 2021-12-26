use std::io;
use thiserror::Error;
use crate::sql::error::QueryRewriterError;

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
    #[error("query rewriter error")]
    RewriterError(#[from] QueryRewriterError),
}