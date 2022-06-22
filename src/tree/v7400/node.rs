//! Node type.

use crate::tree::v7400::{NodeHandle, Tree};

pub(crate) use self::{data::NodeData, name::NodeNameSym};

mod data;
pub(crate) mod handle;
mod name;

/// Node ID in FBX data tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(indextree::NodeId);

impl NodeId {
    /// Creates a new `NodeId`.
    pub(crate) fn new(id: indextree::NodeId) -> Self {
        NodeId(id)
    }

    /// Returns the raw node ID used by internal tree implementation.
    pub(crate) fn raw(self) -> indextree::NodeId {
        self.0
    }

    /// Creates a new `NodeHandle` to make accesible to the node in the tree.
    ///
    /// # Panics and safety
    ///
    /// This may panic if the given node ID is not used in the given tree.
    ///
    /// Even if creation of an invalid node ID does not panic, subsequent
    /// operations through `NodeHandle` object may panic if the given node ID is
    /// not used in the given tree.
    pub fn to_handle(self, tree: &Tree) -> NodeHandle<'_> {
        NodeHandle::new(tree, self)
    }
}
