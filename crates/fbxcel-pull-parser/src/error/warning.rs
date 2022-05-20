//! Invalid operation.

use thiserror::Error;

/// Warning.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Warning {
    #[error("Node name is empty")]
    EmptyNodeName,
    #[error("Extra (unexpected) node end marker found.")]
    ExtraNodeEndMarker,
    #[error("Incorrect boolean representation.")]
    IncorrectBooleanRepresentation,
    #[error("Invalid footer padding length: expected {0} bytes, got {1} ytes")]
    InvalidFooterPaddingLength(usize, usize),
    #[error("Missing a node end marker where the marker is expected.")]
    MissingNodeEndMarker,
    #[error("Unexpected value for footer fields (mainly for unknown fields).")]
    UnexpectedFooterFieldValue,
}
