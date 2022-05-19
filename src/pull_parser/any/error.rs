//! Error and result types for `pull_parser::any` module.

use fbxcel_low::{FbxVersion, HeaderError};
use thiserror::Error;

/// AnyTree load result.
pub type Result<T> = std::result::Result<T, Error>;

/// Error.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error(transparent)]
    Header(#[from] HeaderError),
    #[error("Unsupported version: {0:?}")]
    UnsupportedVersion(FbxVersion),
}
