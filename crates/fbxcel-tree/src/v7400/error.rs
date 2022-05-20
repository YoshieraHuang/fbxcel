//! Error types.

use fbxcel_pull_parser::Error as ParserError;
use thiserror::Error;

/// FBX data tree load error.
#[derive(Debug, Error)]
pub enum LoadError {
    /// Bad parser.
    ///
    /// This error will be mainly caused by user logic error.
    #[error("bad parser")]
    BadParser,
    /// Parser error.
    #[error(transparent)]
    Parser(#[from] ParserError),
}
