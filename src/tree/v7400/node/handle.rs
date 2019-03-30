//! Node handle.

use crate::{
    low::v7400::AttributeValue,
    tree::v7400::{NodeData, NodeId, NodeNameSym, Tree},
};

/// Node handle.
#[derive(Debug, Clone, Copy)]
pub struct NodeHandle<'a> {
    /// The tree the node belongs to.
    tree: &'a Tree,
    /// Node ID.
    node_id: NodeId,
}

impl<'a> NodeHandle<'a> {
    /// Creates a new `NodeHandle`.
    ///
    /// # Panics and safety
    ///
    /// This may panic if the given node ID is not used in the given tree.
    ///
    /// Even if `new()` does not panic, subsequent operations through
    /// `NodeHandle` object may panic if the given node ID is not used in the
    /// given tree.
    pub(crate) fn new(tree: &'a Tree, node_id: NodeId) -> Self {
        assert!(
            tree.contains_node(node_id),
            "The node ID is not used in the given tree: node_id={:?}",
            node_id
        );

        Self { tree, node_id }
    }

    /// Returns a reference to the tree.
    pub fn tree(&self) -> &'a Tree {
        self.tree
    }

    /// Returns the node ID.
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Returns the internally managed node data.
    pub(crate) fn node(&self) -> &'a indextree::Node<NodeData> {
        self.tree.node(self.node_id)
    }

    /// Returns the node name symbol.
    pub(crate) fn name_sym(&self) -> NodeNameSym {
        self.node().data.name_sym()
    }

    /// Returns the node name.
    pub fn name(&self) -> &'a str {
        self.tree.resolve_node_name(self.name_sym())
    }

    /// Returns the node attributes.
    pub fn attributes(&self) -> &'a [AttributeValue] {
        self.node().data.attributes()
    }

    /// Returns an iterator of children with the given name.
    pub fn children(&self) -> impl Iterator<Item = NodeHandle<'a>> + 'a {
        let tree = self.tree;
        self.node_id
            .raw()
            .children(&tree.arena)
            .map(move |child_id| NodeId::new(child_id).to_handle(tree))
    }

    /// Returns an iterator of children with the given name.
    pub fn children_by_name(&self, name: &str) -> impl Iterator<Item = NodeHandle<'a>> + 'a {
        // Using `flat_map` for `Option<impl Iterator>`, the iterator can return
        // `None` before without traversing the tree if `target_name` is not
        // registered.
        self.tree
            .node_name_sym(name)
            .map(|sym| self.children().filter(move |child| child.name_sym() == sym))
            .into_iter()
            .flat_map(|iter| iter)
    }
}

macro_rules! impl_related_node_accessor {
    (
        $(
            $(#[$meta:meta])*
            $accessor:ident;
        )*
    ) => {
        impl<'a> NodeHandle<'a> {
            $(
                impl_related_node_accessor! { @single, $(#[$meta])* $accessor; }
            )*
        }
    };
    (@single, $(#[$meta:meta])* $accessor:ident;) => {
        $(#[$meta])*
        pub fn $accessor(&self) -> Option<NodeHandle<'a>> {
            self.node()
                .$accessor()
                .map(|id| NodeId::new(id).to_handle(&self.tree))
        }
    };
}

impl_related_node_accessor! {
    /// Returns parent node handle if available.
    parent;
    /// Returns first child node handle if available.
    first_child;
    /// Returns last child node handle if available.
    last_child;
    /// Returns previous sibling node handle if available.
    previous_sibling;
    /// Returns next sibling node handle if available.
    next_sibling;
}
