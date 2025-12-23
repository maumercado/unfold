use super::node::{JsonNode, JsonValue};
use std::fmt::Write;

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

    /// Get a mutable reference to a node
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut JsonNode> {
        self.nodes.get_mut(index)
    }

    /// Toggle the expanded state of a node
    pub fn toggle_expanded(&mut self, index: usize) {
        if let Some(node) = self.nodes.get_mut(index) {
            if node.is_expandable() {
                node.expanded = !node.expanded;
            }
        }
    }

    /// Get the root index
    pub fn root_index(&self) -> usize {
        self.root_index
    }

    /// Pretty print the tree structure
    pub fn print_tree(&self) -> String {
        let mut output = String::new();
        if let Some(_root) = self.root() {
            self.print_node(&mut output, self.root_index, 0);
        }
        output
    }

    /// Recursively print a node and its children
    fn print_node(&self, output: &mut String, index: usize, indent: usize) {
        let Some(node) = self.get_node(index) else {
            return;
        };

        // Create indentation
        let prefix = "  ".repeat(indent);

        // Expand/collapse indicator for containers
        let expand_indicator = if node.is_expandable() {
            if node.expanded { "▼ " } else { "▶ " }
        } else {
            "  "
        };

        // Format the node
        let key_str = match &node.key {
            Some(k) => format!("\"{}\": ", k),
            None => String::new(),
        };

        let value_str = match &node.value {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Array => format!("[{} items]", node.children.len()),
            JsonValue::Object => format!("{{{} fields}}", node.children.len()),
        };

        // Write this node
        let _ = writeln!(output, "{}{}{}{}", prefix, expand_indicator, key_str, value_str);

        // Only print children if expanded
        if node.expanded {
            for &child_index in &node.children {
                self.print_node(output, child_index, indent + 1);
            }
        }
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
            expanded: true,
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
            expanded: false,
        };
        let name_index = tree.add_node(name_node);

        // Then add the root with the child's index
        let root_node = JsonNode {
            key: None,
            value: JsonValue::Object,
            depth: 0,
            children: vec![name_index],
            expanded: true,
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

    #[test]
    fn test_toggle_expanded() {
        use crate::parser::builder::build_tree;
        use serde_json::json;

        let value = json!({"name": "Unfold"});
        let mut tree = build_tree(&value);

        let root_idx = tree.root_index();

        // Should start expanded
        assert!(tree.get_node(root_idx).unwrap().expanded);

        // Toggle to collapse
        tree.toggle_expanded(root_idx);
        assert!(!tree.get_node(root_idx).unwrap().expanded);

        // Toggle to expand again
        tree.toggle_expanded(root_idx);
        assert!(tree.get_node(root_idx).unwrap().expanded);
    }

    #[test]
    fn test_print_tree() {
        use crate::parser::builder::build_tree;
        use serde_json::json;

        let value = json!({
            "name": "Unfold",
            "version": "0.1.0"
        });

        let tree = build_tree(&value);
        let output = tree.print_tree();

        println!("{}", output);  // Will show when running with --nocapture

        assert!(output.contains("name"));
        assert!(output.contains("Unfold"));
        assert!(output.contains("version"));
  }
}
