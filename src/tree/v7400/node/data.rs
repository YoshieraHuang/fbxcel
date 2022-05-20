//! Node-local data.

use fbxcel_low::v7400::AttributeValue;

use crate::tree::v7400::node::NodeNameSym;

/// Node-local data in FBX data tree.
///
/// This does not manages relations among nodes (including parent-child
/// relatinos).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodeData {
    /// Node name.
    name_sym: NodeNameSym,
    /// Node attributes.
    attributes: Vec<AttributeValue>,
}

impl NodeData {
    /// Returns the node name symbol.
    pub(crate) fn name_sym(&self) -> NodeNameSym {
        self.name_sym
    }

    /// Returns the reference to the attributes.
    pub(crate) fn attributes(&self) -> &[AttributeValue] {
        &self.attributes
    }

    /// Appends the given value to the attributes.
    pub(crate) fn append_attribute(&mut self, v: AttributeValue) {
        self.attributes.push(v)
    }

    /// Creates a new `NodeData`.
    pub(crate) fn new(name_sym: NodeNameSym, attributes: Vec<AttributeValue>) -> Self {
        Self {
            name_sym,
            attributes,
        }
    }
}
