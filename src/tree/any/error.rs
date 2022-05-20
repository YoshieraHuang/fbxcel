//! Error and result types for `tree::any` module.

use crate::tree::v7400::LoadError;
use thiserror::Error;

/// AnyTree load result.
pub type Result<T> = std::result::Result<T, Error>;

/// Error.
#[derive(Debug, Error)]
pub enum Error {
    /// Parser creation error.
    #[error(transparent)]
    ParserCreation(#[from] fbxcel_pull_parser::any::Error),
    /// Parser error.
    #[error(transparent)]
    Parser(#[from] fbxcel_pull_parser::Error),
    /// Tree load error.
    #[error(transparent)]
    Tree(#[from] LoadError),
}
