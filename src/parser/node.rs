/// Represents the type of a JSON node
#[derive(Debug, Clone, PartialEq)]
pub enum JsonType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

/// Represents a single node in our JSON tree
#[derive(Debug, Clone)]
pub struct JsonNode {
    /// The key if this node is in an object (None for array items or root)
    pub key: Option<String>,
    /// The type of this node
    pub node_type: JsonType,
    /// Depth in the tree (root = 0)
    pub depth: usize,
    /// Indices of child nodes (for arrays and objects)
    pub children: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_null_node() {
        let node = JsonNode {
            key: None,
            node_type: JsonType::Null,
            depth: 0,
            children: vec![],
        };

        assert_eq!(node.node_type, JsonType::Null);
        assert_eq!(node.depth, 0);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_create_object_node_with_key() {
        let node = JsonNode {
            key: Some(String::from("user")),
            node_type: JsonType::Object,
            depth: 1,
            children: vec![2, 3, 4],  // Indices of child nodes
        };

        assert_eq!(node.key, Some(String::from("user")));
        assert_eq!(node.node_type, JsonType::Object);
        assert_eq!(node.children.len(), 3);
    }
}
