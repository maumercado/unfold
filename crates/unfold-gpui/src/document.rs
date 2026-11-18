//! The open JSON document: tree, visible rows and search state.
//!
//! Nothing here touches GPUI. The view (`viewer.rs`) owns a `Document` and asks
//! it to change; the document answers with plain data such as "scroll to row
//! 42". That split keeps the behavior testable without opening a window.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use unfold_core::flat_row::{FlatRow, flatten_visible_nodes};
use unfold_core::parse_error::ParseError;
use unfold_core::parser::{JsonTree, JsonValue, build_tree};
use unfold_core::search;

use crate::row;

/// A successfully loaded file. Built on a background thread, so every field
/// must be `Send` (plain data is).
pub struct Document {
    pub path: PathBuf,
    pub tree: JsonTree,
    pub rows: Vec<FlatRow>,
    /// Row index used to measure the list width for horizontal scrolling
    pub widest_row: Option<usize>,
    pub load_time: Duration,
    pub selected: Option<usize>,
    pub search: SearchState,
}

#[derive(Default)]
pub struct SearchState {
    pub query: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub results: Vec<usize>,
    pub matches: HashSet<usize>,
    pub current: Option<usize>,
    pub regex_error: Option<String>,
}

impl SearchState {
    pub fn current_node(&self) -> Option<usize> {
        self.current.and_then(|i| self.results.get(i).copied())
    }

    /// Text for the toolbar, e.g. "3 / 12" or "No matches"
    pub fn summary(&self) -> String {
        if let Some(err) = &self.regex_error {
            err.clone()
        } else if self.results.is_empty() {
            if self.query.is_empty() {
                String::new()
            } else {
                "No matches".into()
            }
        } else {
            let current = self.current.map(|i| i + 1).unwrap_or(0);
            format!("{} / {}", current, self.results.len())
        }
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into())
}

/// Read, parse and flatten a file. Slow for big files: call it off the UI thread.
pub fn load(path: PathBuf) -> Result<Document, ParseError> {
    let start = Instant::now();
    let filename = file_name(&path);
    let contents = fs::read_to_string(&path).map_err(|e| ParseError {
        message: e.to_string(),
        line: None,
        column: None,
        context_line: None,
        filename: filename.clone(),
    })?;
    let value: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| ParseError::from_serde_error(&e, &contents, &filename))?;
    // Free the file text and the serde Value before building rows.
    drop(contents);
    let tree = build_tree(&value);
    drop(value);
    Ok(Document::new(path, tree, start.elapsed()))
}

impl Document {
    pub fn new(path: PathBuf, tree: JsonTree, load_time: Duration) -> Self {
        let mut doc = Document {
            path,
            tree,
            rows: Vec::new(),
            widest_row: None,
            load_time,
            selected: None,
            search: SearchState::default(),
        };
        doc.reflatten();
        doc
    }

    /// Rebuild the visible rows after the tree changed shape.
    pub fn reflatten(&mut self) {
        self.rows = flatten_visible_nodes(&self.tree);
        self.widest_row = row::widest_row(&self.rows);
    }

    pub fn row_of(&self, node: usize) -> Option<usize> {
        self.rows.iter().position(|r| r.node_index == node)
    }

    pub fn toggle(&mut self, node: usize) {
        self.selected = Some(node);
        self.tree.toggle_expanded(node);
        self.reflatten();
    }

    /// Click on a leaf: select it, or deselect it if it was already selected.
    pub fn toggle_selection(&mut self, node: usize) {
        self.selected = if self.selected == Some(node) {
            None
        } else {
            Some(node)
        };
    }

    pub fn set_expanded_recursive(&mut self, node: usize, expanded: bool) {
        self.tree.set_expanded_recursive(node, expanded);
        self.reflatten();
    }

    /// Move the selection by `delta` rows. Returns the new row to scroll to.
    pub fn move_selection(&mut self, delta: isize) -> Option<usize> {
        if self.rows.is_empty() {
            return None;
        }
        let row = match self.selected.and_then(|n| self.row_of(n)) {
            Some(r) => r.saturating_add_signed(delta).min(self.rows.len() - 1),
            None => 0,
        };
        self.selected = Some(self.rows[row].node_index);
        Some(row)
    }

    /// Expand (`true`) or collapse the selected node. Collapsing a leaf or an
    /// already collapsed node moves the selection to its parent instead.
    /// Returns the row to scroll to.
    pub fn expand_selected(&mut self, expand: bool) -> Option<usize> {
        let node_ix = self.selected?;
        let node = self.tree.get_node(node_ix)?;
        if node.is_expandable() && node.expanded != expand {
            self.tree.set_expanded(node_ix, expand);
            self.reflatten();
        } else if !expand {
            let path = self.tree.get_path_to_node(node_ix);
            if let Some(&parent) = path.iter().rev().nth(1)
                && parent != self.tree.root_index()
            {
                self.selected = Some(parent);
            }
        }
        self.selected.and_then(|n| self.row_of(n))
    }

    /// Path and type of the selected node for the status bar.
    pub fn selection_summary(&self) -> String {
        let Some(node_ix) = self.selected else {
            return String::new();
        };
        let (Some(row), Some(node)) = (
            self.rows.iter().find(|r| r.node_index == node_ix),
            self.tree.get_node(node_ix),
        ) else {
            return String::new();
        };
        let kind = match &node.value {
            JsonValue::Null => "(null)".to_string(),
            JsonValue::Bool(_) => "(bool)".to_string(),
            JsonValue::Number(_) => "(number)".to_string(),
            JsonValue::String(_) => "(string)".to_string(),
            JsonValue::Array => format!("(array, {} items)", node.children.len()),
            JsonValue::Object => format!("(object, {} keys)", node.children.len()),
        };
        format!("{} {}", row.path, kind)
    }

    pub fn path_of(&self, node: usize) -> Option<String> {
        self.rows
            .iter()
            .find(|r| r.node_index == node)
            .map(|r| r.path.clone())
    }

    pub fn has_children(&self, node: usize) -> bool {
        self.tree
            .get_node(node)
            .is_some_and(|n| !n.children.is_empty())
    }

    // ---- Search ----

    /// Run the search with the current query and options. Returns the row of
    /// the first result, if any, so the view can scroll to it.
    pub fn run_search(&mut self) -> Option<usize> {
        let s = &mut self.search;
        if s.query.is_empty() {
            self.clear_search_results();
            return None;
        }
        let (results, error) =
            search::search_nodes(&self.tree, &s.query, s.case_sensitive, s.use_regex);
        s.regex_error = error;
        s.matches = results.iter().copied().collect();
        s.results = results;
        if s.results.is_empty() {
            s.current = None;
            None
        } else {
            self.go_to_result(0)
        }
    }

    pub fn clear_search(&mut self) {
        self.search.query.clear();
        self.clear_search_results();
    }

    fn clear_search_results(&mut self) {
        let s = &mut self.search;
        s.results.clear();
        s.matches.clear();
        s.current = None;
        s.regex_error = None;
    }

    /// Step through results (`forward` = next). Wraps around at both ends.
    pub fn step_result(&mut self, forward: bool) -> Option<usize> {
        let len = self.search.results.len();
        if len == 0 {
            return None;
        }
        let next = match (self.search.current, forward) {
            (Some(i), true) => (i + 1) % len,
            (Some(i), false) => (i + len - 1) % len,
            (None, true) => 0,
            (None, false) => len - 1,
        };
        self.go_to_result(next)
    }

    /// Make result `i` current, expand its ancestors, and return its row.
    fn go_to_result(&mut self, i: usize) -> Option<usize> {
        self.search.current = Some(i);
        let target = self.search.results[i];
        for ancestor in self.tree.get_path_to_node(target) {
            if ancestor != target {
                self.tree.set_expanded(ancestor, true);
            }
        }
        self.reflatten();
        self.row_of(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(value: serde_json::Value) -> Document {
        Document::new("test.json".into(), build_tree(&value), Duration::ZERO)
    }

    #[test]
    fn search_expands_path_and_returns_row() {
        let mut d = doc(json!({"a": {"b": {"needle": 1}}, "c": 2}));
        assert_eq!(d.rows.len(), 2);
        d.search.query = "needle".into();
        let row = d.run_search().expect("found");
        assert_eq!(d.rows[row].key.as_deref(), Some("needle"));
        assert_eq!(d.search.summary(), "1 / 1");
    }

    #[test]
    fn step_result_wraps_both_ways() {
        let mut d = doc(json!({"x1": 1, "x2": 2, "x3": 3}));
        d.search.query = "x".into();
        d.run_search();
        assert_eq!(d.search.current, Some(0));
        d.step_result(false);
        assert_eq!(d.search.current, Some(2));
        d.step_result(true);
        assert_eq!(d.search.current, Some(0));
    }

    #[test]
    fn clear_search_resets_everything() {
        let mut d = doc(json!({"x": 1}));
        d.search.query = "x".into();
        d.run_search();
        d.clear_search();
        assert!(d.search.results.is_empty());
        assert_eq!(d.search.summary(), "");
    }

    #[test]
    fn no_matches_summary() {
        let mut d = doc(json!({"x": 1}));
        d.search.query = "zzz".into();
        assert_eq!(d.run_search(), None);
        assert_eq!(d.search.summary(), "No matches");
    }

    #[test]
    fn keyboard_navigation_moves_expands_and_climbs() {
        let mut d = doc(json!({"a": {"b": 1}, "c": 2}));
        assert_eq!(d.move_selection(1), Some(0)); // nothing selected -> first row
        assert_eq!(d.expand_selected(true), Some(0));
        assert_eq!(d.rows.len(), 3);
        assert_eq!(d.move_selection(1), Some(1)); // "b"
        // collapsing a leaf selects its parent
        assert_eq!(d.expand_selected(false), Some(0));
        assert_eq!(d.move_selection(10), Some(2)); // clamps to the last row
        assert_eq!(d.move_selection(-10), Some(0));
    }

    #[test]
    fn selection_summary_shows_path_and_type() {
        let mut d = doc(json!({"users": [1, 2]}));
        let users = d.rows[0].node_index;
        d.selected = Some(users);
        assert_eq!(d.selection_summary(), "users (array, 2 items)");
    }

    #[test]
    fn load_reports_parse_errors_with_location() {
        let dir = std::env::temp_dir().join("unfold-gpui-test");
        fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.json");
        fs::write(&bad, "{\n  \"a\": ,\n}").unwrap();
        let err = load(bad).err().expect("parse error");
        assert_eq!(err.line, Some(2));
        assert_eq!(err.filename, "bad.json");
    }
}
