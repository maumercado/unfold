/// Represents a JSON value with its data
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array,   // Children stored in JsonNode.children
    Object,  // Children stored in JsonNode.children
}

/// Represents a single node in our JSON tree
#[derive(Debug, Clone)]
pub struct JsonNode {
    /// The key if this node is in an object (None for array items or root)
    pub key: Option<String>,
    /// The value of this node
    pub value: JsonValue,
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
            value: JsonValue::Null,
            depth: 0,
            children: vec![],
        };

        assert_eq!(node.value, JsonValue::Null);
        assert_eq!(node.depth, 0);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_create_string_node() {
        let node = JsonNode {
            key: Some(String::from("greeting")),
            value: JsonValue::String(String::from("Hello, Rust!")),
            depth: 1,
            children: vec![],
        };

        assert_eq!(node.key, Some(String::from("greeting")));

        // Pattern matching to extract the string value!
        match &node.value {
            JsonValue::String(s) => assert_eq!(s, "Hello, Rust!"),
            _ => panic!("Expected a String variant"),
        }
    }

    #[test]
    fn test_create_number_node() {
        let node = JsonNode {
            key: Some(String::from("count")),
            value: JsonValue::Number(42.0),
            depth: 1,
            children: vec![],
        };

        match node.value {
            JsonValue::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected a Number variant"),
        }
    }

    #[test]
    fn test_create_bool_node() {
        let node = JsonNode {
            key: Some(String::from("active")),
            value: JsonValue::Bool(true),
            depth: 1,
            children: vec![],
        };

        assert_eq!(node.value, JsonValue::Bool(true));
    }

    #[test]
    fn test_create_object_node_with_children() {
        let node = JsonNode {
            key: Some(String::from("user")),
            value: JsonValue::Object,
            depth: 1,
            children: vec![2, 3, 4],  // Indices of child nodes
        };

        assert_eq!(node.value, JsonValue::Object);
        assert_eq!(node.children.len(), 3);
        assert_eq!(node.children[0], 2);
    }

    #[test]
    fn test_create_array_node() {
        let node = JsonNode {
            key: Some(String::from("items")),
            value: JsonValue::Array,
            depth: 1,
            children: vec![5, 6, 7, 8],
        };

        assert_eq!(node.value, JsonValue::Array);
        assert_eq!(node.children.len(), 4);
    }
}
