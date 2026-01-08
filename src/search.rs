//! Search functionality for the JSON tree.
//!
//! Supports plain text and regex search with case sensitivity options.

use regex::Regex;
use crate::parser::{JsonTree, JsonValue};

/// Search all nodes in the tree for matches
/// Returns (results, error_message) where error_message is Some if regex is invalid
pub fn search_nodes(
    tree: &JsonTree,
    query: &str,
    case_sensitive: bool,
    use_regex: bool,
) -> (Vec<usize>, Option<String>) {
    if query.is_empty() {
        return (Vec::new(), None);
    }

    // Build the matcher based on options
    let regex = if use_regex {
        let pattern = if case_sensitive {
            query.to_string()
        } else {
            format!("(?i){}", query)
        };
        match Regex::new(&pattern) {
            Ok(r) => Some(r),
            Err(e) => return (Vec::new(), Some(format!("Invalid regex: {}", e))),
        }
    } else {
        None
    };

    let mut results = Vec::new();

    // Helper closure for matching
    let matches = |text: &str| -> bool {
        if let Some(ref re) = regex {
            re.is_match(text)
        } else if case_sensitive {
            text.contains(query)
        } else {
            text.to_lowercase().contains(&query.to_lowercase())
        }
    };

    // Iterate through all nodes
    for i in 0..tree.node_count() {
        if let Some(node) = tree.get_node(i) {
            // Check key
            if let Some(key) = &node.key
                && matches(key) {
                    results.push(i);
                    continue;
                }

            // Check value
            let value_matches = match &node.value {
                JsonValue::String(s) => matches(s),
                JsonValue::Number(n) => matches(&n.to_string()),
                JsonValue::Bool(b) => matches(&b.to_string()),
                JsonValue::Null => matches("null"),
                _ => false,
            };

            if value_matches {
                results.push(i);
            }
        }
    }

    (results, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::build_tree;
    use serde_json::json;

    #[test]
    fn test_search_nodes_basic() {
        let value = json!({"name": "Unfold", "version": "1.0"});
        let tree = build_tree(&value);

        // Search for "Unfold"
        let (results, error) = search_nodes(&tree, "Unfold", false, false);
        assert!(error.is_none());
        assert!(!results.is_empty(), "Should find 'Unfold'");

        // Search for non-existent
        let (results, error) = search_nodes(&tree, "nonexistent", false, false);
        assert!(error.is_none());
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_nodes_case_sensitive() {
        let value = json!({"Name": "Test"});
        let tree = build_tree(&value);

        // Case insensitive should find it
        let (results, _) = search_nodes(&tree, "name", false, false);
        assert!(!results.is_empty());

        // Case sensitive should not find lowercase
        let (results, _) = search_nodes(&tree, "name", true, false);
        assert!(results.is_empty());

        // Case sensitive should find exact match
        let (results, _) = search_nodes(&tree, "Name", true, false);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_nodes_regex() {
        let value = json!({"email": "test@example.com"});
        let tree = build_tree(&value);

        // Regex search for email pattern
        let (results, error) = search_nodes(&tree, r".*@.*\.com", false, true);
        assert!(error.is_none());
        assert!(!results.is_empty());

        // Invalid regex should return error
        let (results, error) = search_nodes(&tree, r"[invalid", false, true);
        assert!(error.is_some());
        assert!(results.is_empty());
    }
}
