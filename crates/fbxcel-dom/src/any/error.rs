//! Error and result types for `tree::any` module.

use fbxcel_low::FbxVersion;
use fbxcel_tree::any::Error as AnyTreeError;
use thiserror::Error;

use crate::v7400::LoadError;

/// AnyTree load result.
pub type Result<T> = std::result::Result<T, Error>;

/// Error.
#[derive(Debug, Error)]
pub enum Error {
    /// Unknown FBX parser.
    ///
    /// This means that the FBX version may be supported by the backend parser, but the backend
    /// parser used to load the document is unsupported by fbxcel-dom crate.
    #[error("Unsupported version: {0:?}")]
    UnsupportedVersion(FbxVersion),
    /// Tree load error.
    #[error(transparent)]
    Tree(#[from] AnyTreeError),
    /// DOM load error.
    #[error(transparent)]
    Dom(#[from] LoadError),
}
