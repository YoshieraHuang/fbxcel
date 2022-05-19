//! Data error.
//!
//! This is mainly syntax and low-level structure error.

use fbxcel_low::{v7400::ArrayAttributeEncoding, LowError};
use std::string::FromUtf8Error;
use thiserror::Error;

/// Data error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum DataError {
    #[error("Data with broken compression (codec={0:?}): {1:?}")]
    BrokenCompression(
        Compression,
        #[source] Box<dyn std::error::Error + Send + Sync>,
    ),
    #[error("FBX footer is broken")]
    BrokenFbxFooter,
    #[error(transparent)]
    Low(#[from] LowError),
    #[error("Invalid node name encoding: {0}")]
    InvalidNodeNameEncoding(#[source] FromUtf8Error),
    #[error("Some error occured while reading node attributes")]
    NodeAttributeError,
    #[error("Node ends with unexpected position: expected {0}, got {1:?}")]
    NodeLengthMismatch(u64, Option<u64>),
    #[error("Unexpected attribute value or type: expected {0}, got {1}")]
    UnexpectedAttribute(String, String),
}

/// Compression format or algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compression {
    /// ZLIB compression.
    Zlib,
}

impl From<ArrayAttributeEncoding> for Compression {
    fn from(v: ArrayAttributeEncoding) -> Self {
        match v {
            ArrayAttributeEncoding::Direct => unreachable!(
                "Data with `ArrayAttributeEncoding::Direct` should not cause (de)compression error",
            ),
            ArrayAttributeEncoding::Zlib => Self::Zlib,
        }
    }
}
