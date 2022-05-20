//! Object-related error.

use fbxcel_low::v7400::AttributeType;
use fbxcel_tree::v7400::NodeId;
use thiserror::Error;

use crate::v7400::{object::ObjectId, LoadError};

/// Object metadata load error.
#[derive(Debug, Clone, Error)]
pub(crate) enum ObjectMetaError {
    /// Duplicate object ID.
    #[error("Duplicate object ID: object={0:?}, node1={1:?}, node2={2:?}")]
    DuplicateObjectId(ObjectId, NodeId, NodeId),
    /// Object ID not found.
    #[error("Object ID not found: node={0:?}")]
    MissingId(NodeId),
    /// Invalid ID value type.
    #[error("Invalid object ID value type for node={0:?}: expected `i64`, got {1:?}")]
    InvalidIdType(NodeId, AttributeType),
    /// Name and class not found.
    #[error("Object name and class not found: node={0:?}, object={1:?}")]
    MissingNameClass(NodeId, ObjectId),
    /// Invalid name and class value type.
    #[error(
        "Invalid object name and class value type for node={0:?}, obj={1:?}: \
    expected string, got {2:?}"
    )]
    InvalidNameClassType(NodeId, ObjectId, AttributeType),
    /// Subclass not found.
    #[error("Object subclass not found: node={0:?}, object={1:?}")]
    MissingSubclass(NodeId, ObjectId),
    /// Invalid subclass value type.
    #[error(
        "Invalid object subclass value type for node={0:?}, obj={1:?}: \
    expected string, got {2:?}"
    )]
    InvalidSubclassType(NodeId, ObjectId, AttributeType),
}

impl From<ObjectMetaError> for LoadError {
    fn from(e: ObjectMetaError) -> Self {
        Self::new(e)
    }
}
