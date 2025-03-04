use std::fmt;
use std::sync::Arc;

use uuid::Uuid;
use yrs::block::Prelim;

use crate::{
    iter::{TraversalOrder, TreeIter},
    Result, Tree, TreeError,
};

/// The ID of a node in a tree. Strings can be made into `NodeId`s using the `into()` method,
/// and `NodeId`s can be converted back into strings using the `to_string()` method.
///
/// Note that `Into<NodeId>` for the string `"<ROOT>"` will return `NodeId::Root`,
/// which cannot be used as a node ID as it is reserved for the actual root node of the tree.
#[derive(Clone, Debug, Default, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum NodeId {
    #[default]
    Root,
    Id(String),
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            NodeId::Root => write!(f, "<ROOT>"),
            NodeId::Id(id) => write!(f, "{}", id),
        }
    }
}

impl PartialEq<&str> for NodeId {
    fn eq(&self, other: &&str) -> bool {
        match self {
            NodeId::Root => *other == "<ROOT>",
            NodeId::Id(id) => id == other,
        }
    }
}

impl From<&str> for NodeId {
    fn from(id: &str) -> Self {
        match id {
            "<ROOT>" => NodeId::Root,
            _ => NodeId::Id(id.to_string()),
        }
    }
}

impl From<&String> for NodeId {
    fn from(id: &String) -> Self {
        NodeId::from(id.as_str())
    }
}

impl From<String> for NodeId {
    fn from(id: String) -> Self {
        NodeId::from(id.as_str())
    }
}

/// A trait for objects that can behave like a node in a tree;
/// this is implemented for [`Node`] and [`Tree`]. When these methods
/// are used on a [`Tree`], they behave as if they were called on the root node.
pub trait NodeApi {
    /// Returns the ID of the node.
    fn id(self: &Arc<Self>) -> &NodeId;

    /// Creates a new child node with a generated ID.
    fn create_child(self: &Arc<Self>) -> Result<Arc<Node>>;

    /// Creates a new child node with a generated ID at the given index in the parent's children.
    fn create_child_at(self: &Arc<Self>, index: usize) -> Result<Arc<Node>>;

    /// Creates a new child node with the given ID at the end of the parent's children.
    fn create_child_with_id(self: &Arc<Self>, id: impl Into<NodeId>) -> Result<Arc<Node>>;

    /// Creates a new child node with the given ID at the given index in the parent's children.
    fn create_child_with_id_at(
        self: &Arc<Self>,
        id: impl Into<NodeId>,
        index: usize,
    ) -> Result<Arc<Node>>;

    /// Moves the node to the given parent, placing it in that parent's children at the given index.
    ///
    /// Given:
    ///
    /// ```text
    /// <ROOT>
    /// ├──A
    /// │  ├──C
    /// │  ├──D
    /// │  └──E
    /// └──B
    /// ```
    ///
    /// If we call `B.move_to(&A, Some(1))`, we get:
    ///
    /// ```text
    /// <ROOT>
    /// └──A
    ///    ├──C
    ///    ├──B
    ///    ├──D
    ///    └──E
    /// ```
    ///
    /// Passing `None` as the index moves the node to the end of the parent's children.
    fn move_to(self: &Arc<Self>, parent: &Node, index: Option<usize>) -> Result<()>;

    /// Moves the node before the given node.
    ///
    /// Given:
    ///
    /// ```text
    /// <ROOT>
    /// ├──A
    /// │  ├──C
    /// │  ├──D
    /// │  └──E
    /// └──B
    /// ```
    ///
    /// If we call `B.move_before(&E)`, we get:
    ///
    /// ```text
    /// <ROOT>
    /// └──A
    ///    ├──C
    ///    ├──D
    ///    ├──B
    ///    └──E
    /// ```
    fn move_before(self: &Arc<Self>, other: &Arc<Node>) -> Result<()>;

    /// Moves the node after the given node.
    ///
    /// Given:
    ///
    /// ```text
    /// <ROOT>
    /// ├──A
    /// │  ├──C
    /// │  ├──D
    /// │  └──E
    /// └──B
    /// ```
    ///
    /// If we call `B.move_after(&E)`, we get:
    ///
    /// ```text
    /// <ROOT>
    /// └──A
    ///    ├──C
    ///    ├──D
    ///    ├──E
    ///    └──B
    /// ```
    fn move_after(self: &Arc<Self>, other: &Arc<Node>) -> Result<()>;

    /// Returns the parent of the node.
    fn parent(self: &Arc<Self>) -> Option<Arc<Node>>;

    /// Returns the ancestors of the node, starting with the node's parent and ending
    /// at the root node.
    fn ancestors(self: &Arc<Self>) -> Vec<Arc<Node>>;

    /// Returns the children of the node.
    fn children(self: &Arc<Self>) -> Vec<Arc<Node>>;

    /// Returns the descendants of the node. Equivalent to `self.traverse(order).skip(1).collect()`.
    fn descendants(self: &Arc<Self>, order: TraversalOrder) -> Vec<Arc<Node>>;

    /// Returns the siblings of the node.
    fn siblings(self: &Arc<Self>) -> Vec<Arc<Node>>;

    /// Returns an iterator over the node and its descendants in the given order.
    fn traverse(self: &Arc<Self>, order: TraversalOrder) -> TreeIter;

    /// Returns the depth of the node. The root node has a depth of 0; all other
    /// nodes have a depth of 1 plus the depth of their parent.
    fn depth(self: &Arc<Self>) -> usize;

    /// Deletes the node from the tree.
    ///
    /// `strategy` can be one of:
    ///   * [`DeleteStrategy::Promote`] - assign this Node's children
    ///     to its parent, placing them at the end of the vector.
    ///   * [`DeleteStrategy::Cascade`] - deletes this node and all its children,
    ///     in reverse-depth-first order.
    fn delete(self: &Arc<Self>, strategy: DeleteStrategy) -> Result<()>;
}

/// The strategy to use when deleting a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteStrategy {
    /// Promote this node's children to the node's parent.
    Promote,
    /// Cascade the deletion to this node's children.
    Cascade,
}

/// A node in a tree.
///
/// * See [`Tree`] for methods to create and find nodes in the tree.
/// * See [`NodeApi`] for the operations that can be performed on a node.
pub struct Node {
    id: NodeId,
    tree: Arc<Tree>,
}

impl Node {
    pub(crate) fn new(id: NodeId, tree: Arc<Tree>) -> Arc<Self> {
        Arc::new(Self { id, tree })
    }

    fn do_create_child(
        self: &Arc<Self>,
        id: impl Into<NodeId>,
        index: Option<usize>,
    ) -> Result<Arc<Self>> {
        let id = id.into();

        if id == NodeId::Root {
            return Err(
                TreeError::InvalidId("<ROOT> cannot be used as a node ID".to_string()).into(),
            );
        }

        self.tree.update_node(&id, &self.id, index)?;
        Ok(Self::new(id, self.tree.clone()))
    }

    fn move_relative(self: &Arc<Self>, other: &Arc<Node>, offset: usize) -> Result<()> {
        if other.id == self.id {
            return Err(TreeError::Cycle(self.id.clone(), other.id.clone()).into());
        }

        if other.id == NodeId::Root {
            return Err(TreeError::InvalidTarget(NodeId::Root).into());
        }

        let new_parent = other.parent().unwrap();
        let siblings = other.siblings();
        let cur_idx = siblings
            .iter()
            .position(|sibling| sibling.id == other.id)
            .unwrap();

        let new_index = cur_idx + offset;
        self.tree
            .update_node(&self.id, &new_parent.id, Some(new_index))?;

        Ok(())
    }

    /// Sets a value on the node at the given key.
    ///
    /// See the "Implementors" section of the [`yrs::block::Prelim`] trait for more
    /// information on the values that can be stored.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use yrs_tree::{Node, Tree, NodeApi};
    /// # use yrs::Doc;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let doc = Arc::new(Doc::new());
    /// # let tree = Tree::new(doc, "directory_structure")?;
    /// let node = tree.create_child()?;
    /// node.set("folder", "New Folder")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set<V: Prelim + Into<yrs::Any>>(&self, key: &str, value: V) -> Result<V::Return> {
        self.tree.set_data(&self.id, key, value)
    }

    /// Gets a previously set value on the node at the given key.
    ///
    /// See [`yrs::Out`] for more information on the types of values that can be returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use yrs_tree::{Node, Tree, NodeApi};
    /// # use yrs::{Doc, Transact};
    /// # use yrs::types::GetString;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let doc = Arc::new(Doc::new());
    /// # let tree = Tree::new(doc.clone(), "directory_structure")?;
    /// let node = tree.create_child()?;
    /// node.set("folder", "New Folder")?;
    /// # let txn = doc.transact();
    /// let Some(yrs::Out::Any(yrs::Any::String(folder))) = node.get("folder")? else {
    ///     panic!("folder is not a string");
    /// };
    /// assert_eq!(*folder, *"New Folder");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, key: &str) -> Result<Option<yrs::Out>> {
        self.tree.get_data(&self.id, key)
    }

    /// Gets a previously set value on the node at the given key, cast to a specific type.
    /// If no value was found, [`yrs::Any::Null`] will be substituted for the value and
    /// deserialized into the given type instead (e.g. into an `Option`).
    ///
    /// See [`yrs::types::map::Map::get_as`] for more information.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use yrs_tree::{Node, Tree, NodeApi};
    /// # use yrs::Doc;
    /// # use yrs::types::GetString;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let doc = Arc::new(Doc::new());
    /// # let tree = Tree::new(doc, "directory_structure")?;
    /// let node = tree.create_child()?;
    /// node.set("folder", "New Folder")?;
    /// let folder = node.get_as::<String>("folder")?;
    /// assert_eq!(folder.as_str(), "New Folder");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_as<V: serde::de::DeserializeOwned>(&self, key: &str) -> Result<V> {
        self.tree.get_data_as(&self.id, key)
    }
}

impl NodeApi for Node {
    fn id(self: &Arc<Self>) -> &NodeId {
        &self.id
    }

    fn create_child(self: &Arc<Self>) -> Result<Arc<Self>> {
        let id = Uuid::now_v7().to_string();
        self.create_child_with_id(id)
    }

    fn create_child_at(self: &Arc<Self>, index: usize) -> Result<Arc<Self>> {
        let id = Uuid::now_v7().to_string();
        self.do_create_child(id, Some(index))
    }

    fn create_child_with_id(self: &Arc<Self>, id: impl Into<NodeId>) -> Result<Arc<Self>> {
        self.do_create_child(id, None)
    }

    fn create_child_with_id_at(
        self: &Arc<Self>,
        id: impl Into<NodeId>,
        index: usize,
    ) -> Result<Arc<Self>> {
        self.do_create_child(id, Some(index))
    }

    fn children(self: &Arc<Self>) -> Vec<Arc<Self>> {
        self.tree
            .get_children(&self.id)
            .into_iter()
            .map(|id| Node::new(id.clone(), self.tree.clone()))
            .collect()
    }

    fn descendants(self: &Arc<Self>, order: TraversalOrder) -> Vec<Arc<Self>> {
        // Don't list ourselves as a descendant
        self.traverse(order).skip(1).collect()
    }

    fn parent(self: &Arc<Self>) -> Option<Arc<Self>> {
        self.tree
            .get_parent(&self.id)
            .map(|id| Node::new(id, self.tree.clone()))
    }

    fn ancestors(self: &Arc<Self>) -> Vec<Arc<Self>> {
        let mut ancestors = vec![];
        let mut current = self.parent();

        while let Some(parent) = current {
            ancestors.push(parent.clone());
            current = parent.parent();
        }

        ancestors
    }

    fn siblings(self: &Arc<Self>) -> Vec<Arc<Self>> {
        if let Some(parent) = self.parent() {
            parent.children().clone()
        } else {
            vec![]
        }
    }

    fn traverse(self: &Arc<Self>, order: TraversalOrder) -> TreeIter {
        self.tree.traverse_starting_at(self.id(), order)
    }

    fn depth(self: &Arc<Self>) -> usize {
        if self.id == NodeId::Root {
            return 0;
        }

        let mut depth = 1;
        let mut current = self.tree.get_parent(&self.id);

        while let Some(parent_id) = current {
            if parent_id == NodeId::Root {
                break;
            }
            depth += 1;
            current = self.tree.get_parent(&parent_id);
        }
        depth
    }

    fn move_to(self: &Arc<Self>, parent: &Node, index: Option<usize>) -> Result<()> {
        self.tree.update_node(&self.id, &parent.id, index)
    }

    fn move_before(self: &Arc<Self>, other: &Arc<Node>) -> Result<()> {
        self.move_relative(other, 0)
    }

    fn move_after(self: &Arc<Self>, other: &Arc<Node>) -> Result<()> {
        self.move_relative(other, 1)
    }

    fn delete(self: &Arc<Self>, strategy: DeleteStrategy) -> Result<()> {
        self.tree.delete_node(&self.id, strategy)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.id)
    }
}
