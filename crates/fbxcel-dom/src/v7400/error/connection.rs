//! Connection-related error.

use fbxcel_low::v7400::AttributeType;
use fbxcel_tree::v7400::NodeId;
use thiserror::Error;

use crate::v7400::{connection::ConnectionIndex, object::ObjectId, LoadError};

/// Object metadata load error.
#[derive(Debug, Clone, Error)]
pub(crate) enum ConnectionError {
    /// Duplicate object ID.
    #[error(
        "Duplicate connections: source={source_object:?}, dest={dest_object:?}, label={label:?} \
    node1={node1:?}, index1={index1:?}, node2={node2:?}, index2={index2:?}"
    )]
    DuplicateConnection {
        source_object: ObjectId,
        dest_object: ObjectId,
        label: Option<String>,
        node1: NodeId,
        index1: ConnectionIndex,
        node2: NodeId,
        index2: ConnectionIndex,
    },
    /// Node types not found.
    #[error("Connection node types not found: node={0:?}, conn_index={1:?}")]
    MissingNodeTypes(NodeId, ConnectionIndex),
    /// Invalid type value of node types.
    #[error(
        "Invalid node types value type for node={0:?}, conn_index={1:?}: \
    expected string, got {2:?}"
    )]
    InvalidNodeTypesType(NodeId, ConnectionIndex, AttributeType),
    /// Invalid value of node types.
    #[error("Invalid node types value for node={:?}, conn_index={0:?}: got {1:?}")]
    InvalidNodeTypesValue(NodeId, ConnectionIndex, String),
    /// Source object ID not found.
    #[error("Connection source object ID not found: node={0:?}, conn_index={1:?}")]
    MissingSourceId(NodeId, ConnectionIndex),
    /// Invalid source ID value type.
    #[error(
        "Invalid source object ID value type for node={0:?}, conn_index={1:?}: \
    expected `i64`, got {2:?}"
    )]
    InvalidSourceIdType(NodeId, ConnectionIndex, AttributeType),
    /// Destination object ID not found.
    #[error("Connection destination object ID not found: node={0:?}, conn_index={1:?}")]
    MissingDestinationId(NodeId, ConnectionIndex),
    /// Invalid source ID value type.
    #[error(
        "Invalid destination object ID value type for node={0:?}, conn_index={1:?}: \
    expected `i64`, got {2:?}"
    )]
    InvalidDestinationIdType(NodeId, ConnectionIndex, AttributeType),
    /// Invalid connection label value type.
    #[error(
        "Invalid connection label value type for node={0:?}, conn_index={1:?}: \
    expected string, got {2:?}"
    )]
    InvalidLabelType(NodeId, ConnectionIndex, AttributeType),
}

impl From<ConnectionError> for LoadError {
    fn from(e: ConnectionError) -> Self {
        Self::new(e)
    }
}
