//! FlatRow and row flattening for virtual scrolling.
//!
//! The tree is flattened once per expand/collapse into a `Vec<FlatRow>`. Each
//! row carries everything a UI needs to draw one line, as plain data. Colors and
//! pixel sizes are the UI's job: a row only says *what kind* of value it holds
//! (see [`ValueType`]).

use crate::parser::{JsonTree, JsonValue};

/// Value type, so each UI can pick a theme-aware color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Bracket,
    Key,
}

/// A flattened row ready for rendering.
/// This pre-computes everything needed to render a single tree row.
#[derive(Debug, Clone)]
pub struct FlatRow {
    /// Index in the original JsonTree (for toggle events)
    pub node_index: usize,
    /// Pre-built prefix string (tree lines: "│  ├─ ")
    pub prefix: String,
    /// The key to display (if any)
    pub key: Option<String>,
    /// The value to display (formatted string)
    pub value_display: String,
    /// Type of value (for theme-aware coloring)
    pub value_type: ValueType,
    /// Is this node expandable (has children)?
    pub is_expandable: bool,
    /// Is this node currently expanded?
    pub is_expanded: bool,
    /// Row index in flattened list (for zebra striping)
    pub row_index: usize,
    /// JSON path to this node (e.g., "users[2].email")
    pub path: String,
}

impl FlatRow {
    /// Create a new FlatRow with all display data pre-computed
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_index: usize,
        prefix: String,
        key: Option<String>,
        value_display: String,
        value_type: ValueType,
        is_expandable: bool,
        is_expanded: bool,
        row_index: usize,
        path: String,
    ) -> Self {
        FlatRow {
            node_index,
            prefix,
            key,
            value_display,
            value_type,
            is_expandable,
            is_expanded,
            row_index,
            path,
        }
    }
}

/// Flatten the tree into a Vec<FlatRow> for virtual scrolling
pub fn flatten_visible_nodes(tree: &JsonTree) -> Vec<FlatRow> {
    let mut rows = Vec::new();

    if let Some(root) = tree.get_node(tree.root_index()) {
        let root_is_array = matches!(root.value, JsonValue::Array);
        let child_count = root.children.len();
        for (i, &child_index) in root.children.iter().enumerate() {
            let is_last = i == child_count - 1;
            let child_path = if root_is_array {
                format!("[{}]", i)
            } else if let Some(child) = tree.get_node(child_index) {
                child.key.as_deref().unwrap_or("").to_string()
            } else {
                String::new()
            };
            flatten_node(
                tree,
                child_index,
                &mut rows,
                "",
                is_last,
                false,
                &child_path,
            );
        }
    }

    rows
}

/// Recursively flatten a single node and its visible children
fn flatten_node(
    tree: &JsonTree,
    index: usize,
    rows: &mut Vec<FlatRow>,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    current_path: &str,
) {
    let Some(node) = tree.get_node(index) else {
        return;
    };

    let (current_prefix, child_prefix) = if is_root {
        (String::new(), String::new())
    } else if node.depth == 1 {
        let connector = if is_last { "└" } else { "├" };
        let child = if is_last {
            "   ".to_string()
        } else {
            "│  ".to_string()
        };
        (connector.to_string(), child)
    } else {
        let connector = if is_last { "└" } else { "├" };
        let current = format!("{}{}", prefix, connector);
        let child = if is_last {
            format!("{}   ", prefix)
        } else {
            format!("{}│  ", prefix)
        };
        (current, child)
    };

    let (value_display, value_type) = match &node.value {
        JsonValue::Null => ("null".to_string(), ValueType::Null),
        JsonValue::Bool(b) => (b.to_string(), ValueType::Bool),
        JsonValue::Number(n) => (n.to_string(), ValueType::Number),
        JsonValue::String(s) => (format!("\"{}\"", s), ValueType::String),
        JsonValue::Array => {
            if node.expanded {
                (":".to_string(), ValueType::Bracket)
            } else if node.children.is_empty() {
                ("[]".to_string(), ValueType::Key)
            } else {
                ("[...]".to_string(), ValueType::Key)
            }
        }
        JsonValue::Object => {
            if node.expanded {
                (":".to_string(), ValueType::Bracket)
            } else if node.children.is_empty() {
                ("{}".to_string(), ValueType::Key)
            } else {
                ("{...}".to_string(), ValueType::Key)
            }
        }
    };

    let row_index = rows.len();

    rows.push(FlatRow::new(
        index,
        current_prefix,
        node.key.as_ref().map(|k| k.to_string()),
        value_display,
        value_type,
        node.is_expandable(),
        node.expanded,
        row_index,
        current_path.to_string(),
    ));

    if node.expanded {
        let is_array = matches!(node.value, JsonValue::Array);
        let child_count = node.children.len();
        for (i, &child_index) in node.children.iter().enumerate() {
            let is_last_child = i == child_count - 1;
            let child_path = if is_array {
                format!("{}[{}]", current_path, i)
            } else if let Some(child) = tree.get_node(child_index) {
                let key = child.key.as_deref().unwrap_or("");
                if current_path.is_empty() {
                    key.to_string()
                } else {
                    format!("{}.{}", current_path, key)
                }
            } else {
                current_path.to_string()
            };
            flatten_node(
                tree,
                child_index,
                rows,
                &child_prefix,
                is_last_child,
                false,
                &child_path,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::build_tree;
    use serde_json::json;

    #[test]
    fn test_flatten_visible_nodes() {
        let value = json!({"a": 1, "b": 2});
        let tree = build_tree(&value);

        let flat_rows = flatten_visible_nodes(&tree);

        assert!(!flat_rows.is_empty());
    }

    #[test]
    fn test_flatten_lists_root_children_with_connectors() {
        let tree = build_tree(&json!({"a": 1, "b": 2}));

        let rows = flatten_visible_nodes(&tree);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].prefix, "├");
        assert_eq!(rows[1].prefix, "└");
        assert_eq!(rows[0].key.as_deref(), Some("a"));
        assert_eq!(rows[0].value_display, "1");
        assert_eq!(rows[0].value_type, ValueType::Number);
        assert_eq!(rows[0].path, "a");
        assert_eq!(rows[1].row_index, 1);
    }

    #[test]
    fn test_flatten_collapsed_containers_show_placeholders() {
        // serde_json keeps object keys sorted, so rows come out in key order
        let tree =
            build_tree(&json!({"arr": [1], "empty_arr": [], "empty_obj": {}, "obj": {"x": 1}}));

        let rows = flatten_visible_nodes(&tree);
        let display: Vec<&str> = rows.iter().map(|r| r.value_display.as_str()).collect();

        assert_eq!(display, vec!["[...]", "[]", "{}", "{...}"]);
        assert!(rows[0].is_expandable);
        assert!(!rows[1].is_expandable);
        assert!(!rows[2].is_expandable);
        assert!(rows[3].is_expandable);
    }

    #[test]
    fn test_flatten_expanded_nodes_build_paths_and_prefixes() {
        // Keys are sorted: "a" comes first, then "users"
        let mut tree = build_tree(&json!({"users": [{"email": "a@b.c"}], "a": null}));
        let root = tree.root_index();
        tree.set_expanded_recursive(root, true);

        let rows = flatten_visible_nodes(&tree);
        let paths: Vec<&str> = rows.iter().map(|r| r.path.as_str()).collect();

        assert_eq!(paths, vec!["a", "users", "users[0]", "users[0].email"]);
        assert_eq!(rows[0].prefix, "├");
        assert_eq!(rows[0].value_type, ValueType::Null);
        assert_eq!(rows[1].value_display, ":");
        assert_eq!(rows[1].value_type, ValueType::Bracket);
        // "users" is the last child, so its descendants get blank space, not a vertical line
        assert_eq!(rows[1].prefix, "└");
        assert_eq!(rows[2].prefix, "   └");
        assert_eq!(rows[3].prefix, "      └");
    }

    #[test]
    fn test_flatten_non_last_parent_keeps_vertical_line() {
        // Keys are sorted: "a" (with a child) comes before "z"
        let mut tree = build_tree(&json!({"a": {"x": 1}, "z": 2}));
        let root = tree.root_index();
        tree.set_expanded_recursive(root, true);

        let rows = flatten_visible_nodes(&tree);

        assert_eq!(rows[0].prefix, "├");
        assert_eq!(rows[1].prefix, "│  └");
        assert_eq!(rows[1].path, "a.x");
        assert_eq!(rows[2].prefix, "└");
    }

    #[test]
    fn test_flatten_array_root_uses_index_paths() {
        let tree = build_tree(&json!(["x", "y"]));

        let rows = flatten_visible_nodes(&tree);

        assert_eq!(rows[0].path, "[0]");
        assert_eq!(rows[1].path, "[1]");
    }

    #[test]
    fn test_flatten_empty_tree_has_no_rows() {
        let tree = JsonTree::new();

        assert!(flatten_visible_nodes(&tree).is_empty());
    }
}
