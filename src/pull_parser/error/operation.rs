//! Invalid operation.

use crate::pull_parser::ParserVersion;
use fbxcel_low::FbxVersion;
use thiserror::Error;

/// Invalid operation.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum OperationError {
    #[error("Attempt to parse more data while the parsing is aborted")]
    AlreadyAborted,
    #[error("Attempt to parse more data while the parsing is (successfully) finished.")]
    AlreadyFinished,
    #[error("Unsupported FBX version: parser={0:?}, fbx={1:?}")]
    UnsupportedFbxVersion(ParserVersion, FbxVersion),
}
