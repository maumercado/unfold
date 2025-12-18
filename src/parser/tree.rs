use super::node::{JsonNode, JsonValue};

/// A complete JSON tree stored as a flat array of nodes
#[derive(Debug)]
pub struct JsonTree {
    /// All nodes stored in a flat array
    nodes: Vec<JsonNode>,
    /// Index of the root node (usually 0)
    root_index: usize,
}

impl JsonTree {
      /// Create a new empty tree
    pub fn new() -> Self {
        JsonTree {
            nodes: Vec::new(),
            root_index: 0,
        }
    }

    /// Add a node to the tree and return its index
    pub fn add_node(&mut self, node: JsonNode) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    /// Set the root index
    pub fn set_root(&mut self, index: usize) {
        self.root_index = index;
    }

    /// Get a node by index
    pub fn get_node(&self, index: usize) -> Option<&JsonNode> {
        self.nodes.get(index)
    }

    /// Get the root node
    pub fn root(&self) -> Option<&JsonNode> {
        self.get_node(self.root_index)
    }

    /// Get the total number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_tree() {
        let tree = JsonTree::new();

        assert_eq!(tree.node_count(), 0);
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_add_single_node() {
        let mut tree = JsonTree::new();

        let node = JsonNode {
            key: None,
            value: JsonValue::Object,
            depth: 0,
            children: vec![],
        };

        let index = tree.add_node(node);

        assert_eq!(index, 0);
        assert_eq!(tree.node_count(), 1);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_build_simple_tree() {
        // Build: {"name": "Unfold"}
        let mut tree = JsonTree::new();

        // First, add the child node (we need its index)
        let name_node = JsonNode {
            key: Some(String::from("name")),
            value: JsonValue::String(String::from("Unfold")),
            depth: 1,
            children: vec![],
        };
        let name_index = tree.add_node(name_node);

        // Then add the root with the child's index
        let root_node = JsonNode {
            key: None,
            value: JsonValue::Object,
            depth: 0,
            children: vec![name_index],
        };
        tree.add_node(root_node);

        // Verify structure
        assert_eq!(tree.node_count(), 2);

        // Check root
        let root = tree.get_node(1).unwrap();  // Root is at index 1
        assert_eq!(root.value, JsonValue::Object);
        assert_eq!(root.children.len(), 1);

        // Check child
        let child = tree.get_node(root.children[0]).unwrap();
        assert_eq!(child.key, Some(String::from("name")));
    }
}
