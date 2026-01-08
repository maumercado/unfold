//! JSON serialization utilities for copy and export operations.
//!
//! Provides functions to convert tree nodes back to JSON strings.

use crate::parser::{JsonTree, JsonValue};

/// Format a node's value for copying to clipboard
/// For primitives: just the value
/// For objects/arrays: JSON representation
pub fn format_node_value_for_copy(tree: &JsonTree, node_index: usize) -> String {
    if let Some(node) = tree.get_node(node_index) {
        match &node.value {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => s.clone(),
            JsonValue::Array | JsonValue::Object => {
                // For containers, rebuild the JSON structure
                node_to_json_string(tree, node_index)
            }
        }
    } else {
        String::new()
    }
}

/// Convert a node and its children to a JSON string (with spaces)
pub fn node_to_json_string(tree: &JsonTree, node_index: usize) -> String {
    node_to_json_string_internal(tree, node_index, false)
}

/// Convert a node and its children to a minified JSON string (no spaces)
pub fn node_to_json_string_minified(tree: &JsonTree, node_index: usize) -> String {
    node_to_json_string_internal(tree, node_index, true)
}

fn node_to_json_string_internal(tree: &JsonTree, node_index: usize, minified: bool) -> String {
    let (sep, kv_sep) = if minified { (",", ":") } else { (", ", ": ") };

    if let Some(node) = tree.get_node(node_index) {
        match &node.value {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", escape_json_string(s)),
            JsonValue::Array => {
                let items: Vec<String> = node.children.iter()
                    .map(|&child_idx| node_to_json_string_internal(tree, child_idx, minified))
                    .collect();
                format!("[{}]", items.join(sep))
            }
            JsonValue::Object => {
                let items: Vec<String> = node.children.iter()
                    .filter_map(|&child_idx| {
                        tree.get_node(child_idx).map(|child| {
                            let key = child.key.as_deref().unwrap_or("");
                            let value = node_to_json_string_internal(tree, child_idx, minified);
                            format!("\"{}\"{}{}", key, kv_sep, value)
                        })
                    })
                    .collect();
                format!("{{{}}}", items.join(sep))
            }
        }
    } else {
        String::new()
    }
}

/// Escape special characters in a JSON string
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::build_tree;
    use serde_json::json;

    #[test]
    fn test_node_to_json_string_primitives() {
        // Test primitive value serialization
        let value = json!({"str": "hello", "num": 42, "bool": true, "null": null});
        let tree = build_tree(&value);

        // Get root node and test children
        if let Some(root) = tree.get_node(tree.root_index()) {
            for &child_idx in &root.children {
                let json_str = node_to_json_string(&tree, child_idx);
                // Each child is a key-value pair, so we can verify it's valid JSON-ish
                assert!(!json_str.is_empty());
            }
        }
    }

    #[test]
    fn test_node_to_json_string_nested() {
        // Test nested object serialization
        let value = json!({"nested": {"key": "value"}});
        let tree = build_tree(&value);

        let json_str = node_to_json_string(&tree, tree.root_index());
        // Should contain the nested structure
        assert!(json_str.contains("nested"));
        assert!(json_str.contains("key"));
        assert!(json_str.contains("value"));
    }

    #[test]
    fn test_node_to_json_string_minified() {
        // Test minified JSON output has no spaces
        let value = json!({"key": "value", "nested": {"a": 1, "b": 2}});
        let tree = build_tree(&value);

        let minified = node_to_json_string_minified(&tree, tree.root_index());

        // Minified should not have spaces after colons or commas
        assert!(!minified.contains(": "), "Minified should not have ': ' (colon-space)");
        assert!(!minified.contains(", "), "Minified should not have ', ' (comma-space)");

        // But should still have colons and commas
        assert!(minified.contains(":"), "Should contain colons");
        assert!(minified.contains(","), "Should contain commas");
    }

    #[test]
    fn test_node_to_json_string_regular_vs_minified() {
        // Test that regular has spaces, minified doesn't
        let value = json!({"a": 1, "b": 2});
        let tree = build_tree(&value);

        let regular = node_to_json_string(&tree, tree.root_index());
        let minified = node_to_json_string_minified(&tree, tree.root_index());

        // Regular should be longer due to spaces
        assert!(regular.len() > minified.len(), "Regular should be longer than minified");

        // Both should produce valid JSON structure
        assert!(regular.starts_with('{'));
        assert!(regular.ends_with('}'));
        assert!(minified.starts_with('{'));
        assert!(minified.ends_with('}'));
    }

    #[test]
    fn test_minified_json_array() {
        // Test minified JSON for arrays
        let value = json!([1, 2, 3, "test"]);
        let tree = build_tree(&value);

        let minified = node_to_json_string_minified(&tree, tree.root_index());

        // Should be compact
        assert!(!minified.contains(", "));
        assert!(minified.starts_with('['));
        assert!(minified.ends_with(']'));
    }

    #[test]
    fn test_minified_json_with_special_chars() {
        // Test that special characters in strings are properly escaped
        let value = json!({"text": "line1\nline2\ttab"});
        let tree = build_tree(&value);

        let minified = node_to_json_string_minified(&tree, tree.root_index());

        // Should have escaped newline and tab
        assert!(minified.contains("\\n"), "Should escape newlines");
        assert!(minified.contains("\\t"), "Should escape tabs");
    }

    #[test]
    fn test_minified_json_with_quotes() {
        // Test that quotes in strings are properly escaped
        let value = json!({"text": "he said \"hello\""});
        let tree = build_tree(&value);

        let minified = node_to_json_string_minified(&tree, tree.root_index());

        // Should have escaped quotes
        assert!(minified.contains("\\\""), "Should escape quotes");
    }

    #[test]
    fn test_json_primitives_minified() {
        // Test primitive values
        let null_val = json!(null);
        let bool_val = json!(true);
        let num_val = json!(42);
        let str_val = json!("hello");

        let null_tree = build_tree(&null_val);
        let bool_tree = build_tree(&bool_val);
        let num_tree = build_tree(&num_val);
        let str_tree = build_tree(&str_val);

        assert_eq!(node_to_json_string_minified(&null_tree, null_tree.root_index()), "null");
        assert_eq!(node_to_json_string_minified(&bool_tree, bool_tree.root_index()), "true");
        assert_eq!(node_to_json_string_minified(&num_tree, num_tree.root_index()), "42");
        assert_eq!(node_to_json_string_minified(&str_tree, str_tree.root_index()), "\"hello\"");
    }

    #[test]
    fn test_format_node_value_for_copy() {
        // Test the copy value formatting
        let value = json!({"str": "hello", "num": 42});
        let tree = build_tree(&value);

        // Find the string child node
        if let Some(root) = tree.get_node(tree.root_index()) {
            for &child_idx in &root.children {
                let copy_value = format_node_value_for_copy(&tree, child_idx);
                // Should produce a non-empty string
                assert!(!copy_value.is_empty());
            }
        }
    }

    #[test]
    fn test_deeply_nested_minified() {
        // Test deeply nested structure
        let value = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": [1, 2, 3]
                    }
                }
            }
        });
        let tree = build_tree(&value);

        let minified = node_to_json_string_minified(&tree, tree.root_index());

        // Should be compact throughout all levels
        assert!(!minified.contains(": "));
        assert!(!minified.contains(", "));

        // Should contain all the nested keys
        assert!(minified.contains("level1"));
        assert!(minified.contains("level2"));
        assert!(minified.contains("level3"));
        assert!(minified.contains("value"));
    }

    #[test]
    fn test_empty_object_and_array() {
        let empty_obj = json!({});
        let empty_arr = json!([]);

        let obj_tree = build_tree(&empty_obj);
        let arr_tree = build_tree(&empty_arr);

        assert_eq!(node_to_json_string_minified(&obj_tree, obj_tree.root_index()), "{}");
        assert_eq!(node_to_json_string_minified(&arr_tree, arr_tree.root_index()), "[]");
    }
}
