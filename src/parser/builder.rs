use serde_json::Value;
use super::node::{JsonNode, JsonValue};
use super::tree::JsonTree;

/// Build a JsonTree from a serde_json::Value
pub fn build_tree(json: &Value) -> JsonTree {
    let mut tree = JsonTree::new();

    // Recursively build the tree starting from root
    let root_index = build_node(&mut tree, None, json, 0);

    // Set the root to the actual root node
    tree.set_root(root_index);

    tree
}

/// Recursively build a node and its children
/// Returns the index of the created node
fn build_node(
    tree: &mut JsonTree,
    key: Option<String>,
    value: &Value,
    depth: usize,
) -> usize {
    // First, determine the JsonValue and collect children
    let (node_value, child_values) = match value {
        Value::Null => (JsonValue::Null, vec![]),
        Value::Bool(b) => (JsonValue::Bool(*b), vec![]),
        Value::Number(n) => {
            // Convert to f64 (serde_json stores numbers specially)
            let num = n.as_f64().unwrap_or(0.0);
            (JsonValue::Number(num), vec![])
        }
        Value::String(s) => (JsonValue::String(s.clone()), vec![]),
        Value::Array(arr) => {
            // Collect array items with index as key: [0], [1], etc.
            let children: Vec<(Option<String>, &Value)> =
                arr.iter()
                    .enumerate()
                    .map(|(i, v)| (Some(format!("[{}]", i)), v))
                    .collect();
            (JsonValue::Array, children)
        }
        Value::Object(obj) => {
            // Collect object entries as (Some(key), value) pairs
            let children: Vec<(Option<String>, &Value)> =
                obj.iter().map(|(k, v)| (Some(k.clone()), v)).collect();
            (JsonValue::Object, children)
        }
    };

    // Build children first (we need their indices)
    let child_indices: Vec<usize> = child_values
        .into_iter()
        .map(|(child_key, child_value)| {
            build_node(tree, child_key, child_value, depth + 1)
        })
        .collect();

    // All containers start collapsed for better performance with large files
    // Users expand what they need to see

    // Now create this node with the child indices
    let node = JsonNode {
        key,
        value: node_value,
        depth,
        children: child_indices,
        expanded: false,  // Start collapsed - expand on demand
    };

    // Add to tree and return index
    tree.add_node(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_null() {
        let value = json!(null);
        let tree = build_tree(&value);

        assert_eq!(tree.node_count(), 1);

        let root = tree.root().unwrap();
        assert_eq!(root.value, JsonValue::Null);
    }

    #[test]
    fn test_build_simple_object() {
        let value = json!({"name": "Unfold"});
        let tree = build_tree(&value);

        assert_eq!(tree.node_count(), 2);  // Root object + 1 string
    }

    #[test]
    fn test_build_nested_object() {
        let value = json!({
            "app": {
                "name": "Unfold",
                "version": "0.1.0"
            }
        });
        let tree = build_tree(&value);

        // Root object + "app" object + "name" string + "version" string
        assert_eq!(tree.node_count(), 4);
    }

    #[test]
    fn test_build_array() {
        let value = json!(["a", "b", "c"]);
        let tree = build_tree(&value);

        // Root array + 3 strings
        assert_eq!(tree.node_count(), 4);

        let root = tree.root().unwrap();
        assert_eq!(root.value, JsonValue::Array);
        assert_eq!(root.children.len(), 3);
    }
}
