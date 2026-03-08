//! Search functionality for the JSON tree.
//!
//! Supports plain text and regex search with case sensitivity options,
//! and a scope filter to search keys only, values only, or both.

use crate::parser::{JsonTree, JsonValue};
use regex::Regex;

/// Controls which part of each node is matched against the query.
///
/// - `All`    – check both key and value (original behaviour)
/// - `Keys`   – check only the node's key string
/// - `Values` – check only the node's scalar value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchScope {
    #[default]
    All,
    Keys,
    Values,
}

impl SearchScope {
    /// Cycle through the three scopes: All → Keys → Values → All
    pub fn next(self) -> Self {
        match self {
            SearchScope::All => SearchScope::Keys,
            SearchScope::Keys => SearchScope::Values,
            SearchScope::Values => SearchScope::All,
        }
    }

    /// Short label shown in the toolbar button.
    pub fn label(self) -> &'static str {
        match self {
            SearchScope::All => "K+V",
            SearchScope::Keys => "K",
            SearchScope::Values => "V",
        }
    }
}

/// Search all nodes in the tree for matches.
///
/// Returns `(results, error_message)` where `error_message` is `Some` if the
/// regex pattern is invalid.
///
/// # Arguments
/// * `tree`           – the parsed JSON tree
/// * `query`          – the search string (or regex pattern)
/// * `case_sensitive` – whether the match is case-sensitive
/// * `use_regex`      – interpret `query` as a regular expression
/// * `scope`          – which part of each node to match against
pub fn search_nodes(
    tree: &JsonTree,
    query: &str,
    case_sensitive: bool,
    use_regex: bool,
    scope: SearchScope,
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

    // Helper closure for matching a single string
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
        let Some(node) = tree.get_node(i) else {
            continue;
        };

        // --- Key check (skipped when scope is Values-only) ---
        if scope != SearchScope::Values
            && let Some(key) = &node.key
            && matches(key)
        {
            results.push(i);
            continue; // key matched – no need to also check the value
        }

        // --- Value check (skipped when scope is Keys-only) ---
        if scope != SearchScope::Keys {
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

/// Split `haystack` into a list of `(substring, is_match)` segments so that
/// matched substrings can be rendered in a highlight colour while the rest
/// keeps its normal colour.
///
/// The original casing of `haystack` is always preserved in the output.
///
/// Returns a single `(haystack.to_string(), false)` when:
/// - `query` is empty, or
/// - `query` does not match anything in `haystack`.
pub fn highlight_segments(
    haystack: &str,
    query: &str,
    case_sensitive: bool,
    use_regex: bool,
) -> Vec<(String, bool)> {
    if query.is_empty() || haystack.is_empty() {
        return vec![(haystack.to_string(), false)];
    }

    // Collect byte-ranges of every match.
    let mut ranges: Vec<(usize, usize)> = Vec::new();

    if use_regex {
        let pattern = if case_sensitive {
            query.to_string()
        } else {
            format!("(?i){}", query)
        };
        let Ok(re) = Regex::new(&pattern) else {
            return vec![(haystack.to_string(), false)];
        };
        for m in re.find_iter(haystack) {
            ranges.push((m.start(), m.end()));
        }
    } else {
        // Plain-text: scan for non-overlapping matches, preserving original casing.
        let (search_in, needle) = if case_sensitive {
            (haystack.to_string(), query.to_string())
        } else {
            (haystack.to_lowercase(), query.to_lowercase())
        };

        let mut start = 0;
        while start < search_in.len() {
            if let Some(pos) = search_in[start..].find(&needle) {
                let abs = start + pos;
                ranges.push((abs, abs + needle.len()));
                start = abs + needle.len().max(1);
            } else {
                break;
            }
        }
    }

    if ranges.is_empty() {
        return vec![(haystack.to_string(), false)];
    }

    // Build segments from the collected ranges.
    let mut segments = Vec::new();
    let mut cursor = 0_usize;

    for (start, end) in ranges {
        if cursor < start {
            segments.push((haystack[cursor..start].to_string(), false));
        }
        segments.push((haystack[start..end].to_string(), true));
        cursor = end;
    }

    if cursor < haystack.len() {
        segments.push((haystack[cursor..].to_string(), false));
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::build_tree;
    use serde_json::json;

    // -------------------------------------------------------------------------
    // Shared helper: build a tree that has a clear split of key vs value matches
    // for the query "tran" (case-insensitive):
    //
    //  crsTransferStatus  : ""                 → key match only  (empty value)
    //  description        : "energy transfer…" → value match only
    //  studentName        : "John Tran"         → value match only
    //  status             : "ACTIVE"            → no match
    //  creditType         : "transfer"          → value match only
    //  count              : 42                  → no match
    //  nested / innerTransfer : "some value"    → key match only
    //  nested / innerField    : "transition…"   → value match only
    //
    // Expected counts:  All=6  Keys=2  Values=4
    // -------------------------------------------------------------------------
    fn tran_tree() -> crate::parser::JsonTree {
        build_tree(&json!({
            "crsTransferStatus": "",
            "description": "energy transfer and expenditure",
            "studentName": "John Tran",
            "status": "ACTIVE",
            "creditType": "transfer",
            "count": 42,
            "nested": {
                "innerTransfer": "some value",
                "innerField": "transition planning"
            }
        }))
    }

    // =========================================================================
    // EXISTING TESTS – updated to pass the new `scope` argument (SearchScope::All
    // preserves the original behaviour exactly).
    // =========================================================================

    #[test]
    fn test_search_nodes_basic() {
        let value = json!({"name": "Unfold", "version": "1.0"});
        let tree = build_tree(&value);

        // Search for "Unfold"
        let (results, error) = search_nodes(&tree, "Unfold", false, false, SearchScope::All);
        assert!(error.is_none());
        assert!(!results.is_empty(), "Should find 'Unfold'");

        // Search for non-existent
        let (results, error) = search_nodes(&tree, "nonexistent", false, false, SearchScope::All);
        assert!(error.is_none());
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_nodes_case_sensitive() {
        let value = json!({"Name": "Test"});
        let tree = build_tree(&value);

        // Case insensitive should find it
        let (results, _) = search_nodes(&tree, "name", false, false, SearchScope::All);
        assert!(!results.is_empty());

        // Case sensitive should not find lowercase
        let (results, _) = search_nodes(&tree, "name", true, false, SearchScope::All);
        assert!(results.is_empty());

        // Case sensitive should find exact match
        let (results, _) = search_nodes(&tree, "Name", true, false, SearchScope::All);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_nodes_regex() {
        let value = json!({"email": "test@example.com"});
        let tree = build_tree(&value);

        // Regex search for email pattern
        let (results, error) = search_nodes(&tree, r".*@.*\.com", false, true, SearchScope::All);
        assert!(error.is_none());
        assert!(!results.is_empty());

        // Invalid regex should return error
        let (results, error) = search_nodes(&tree, r"[invalid", false, true, SearchScope::All);
        assert!(error.is_some());
        assert!(results.is_empty());
    }

    // =========================================================================
    // NEW SCOPE TESTS
    // =========================================================================

    /// 1. Scope::All finds both key and value matches (backward-compat baseline).
    #[test]
    fn test_search_scope_all_matches_keys_and_values() {
        let tree = tran_tree();
        let (results, error) = search_nodes(&tree, "tran", false, false, SearchScope::All);
        assert!(error.is_none());
        assert_eq!(
            results.len(),
            6,
            "All scope should find 6 nodes (2 key + 4 value)"
        );
    }

    /// 2. Scope::Keys finds only nodes whose key contains the query.
    #[test]
    fn test_search_scope_keys_only() {
        let tree = tran_tree();
        let (results, error) = search_nodes(&tree, "tran", false, false, SearchScope::Keys);
        assert!(error.is_none());
        assert_eq!(
            results.len(),
            2,
            "Keys scope should find 2 nodes (crsTransferStatus, innerTransfer)"
        );

        // Verify that every matched node actually has a matching key
        for &idx in &results {
            let node = tree.get_node(idx).unwrap();
            let key = node.key.as_deref().unwrap_or("");
            assert!(
                key.to_lowercase().contains("tran"),
                "Node {} key {:?} should contain 'tran'",
                idx,
                key
            );
        }
    }

    /// 3. Scope::Values finds only nodes whose scalar value contains the query.
    #[test]
    fn test_search_scope_values_only() {
        let tree = tran_tree();
        let (results, error) = search_nodes(&tree, "tran", false, false, SearchScope::Values);
        assert!(error.is_none());
        assert_eq!(
            results.len(),
            4,
            "Values scope should find 4 nodes (description, studentName, creditType, innerField)"
        );
    }

    /// 4. Scope::Values must not return nodes where only the key matches.
    #[test]
    fn test_search_scope_values_only_skips_key_matches() {
        let tree = tran_tree();
        // "crsTransfer" appears only in a key, never in a value
        let (results, error) =
            search_nodes(&tree, "crsTransfer", false, false, SearchScope::Values);
        assert!(error.is_none());
        assert!(
            results.is_empty(),
            "Values scope must not match key-only nodes"
        );
    }

    /// 5. Scope::Keys must not return nodes where only the value matches.
    #[test]
    fn test_search_scope_keys_only_skips_value_matches() {
        let tree = tran_tree();
        // "John" appears only in the value of studentName, not in any key
        let (results, error) = search_nodes(&tree, "John", false, false, SearchScope::Keys);
        assert!(error.is_none());
        assert!(
            results.is_empty(),
            "Keys scope must not match value-only nodes"
        );
    }

    /// 6. Scope::Values + case-sensitive distinguishes "Tran" from "transfer".
    #[test]
    fn test_search_scope_with_case_sensitive() {
        let tree = tran_tree();

        // "Tran" (capital T, case-sensitive) → only "John Tran"
        let (results, _) = search_nodes(&tree, "Tran", true, false, SearchScope::Values);
        assert_eq!(
            results.len(),
            1,
            "Case-sensitive 'Tran' in values should find exactly 1 node"
        );

        // "TRAN" (all caps, case-sensitive) → no match at all
        let (results, _) = search_nodes(&tree, "TRAN", true, false, SearchScope::Values);
        assert!(
            results.is_empty(),
            "Case-sensitive 'TRAN' in values should find nothing"
        );
    }

    /// 7. Scope::Values + regex: pattern matches inside values only.
    #[test]
    fn test_search_scope_with_regex() {
        let tree = tran_tree();
        // Matches "transfer" and "transition" in values (3 nodes: description, creditType, innerField)
        let (results, error) =
            search_nodes(&tree, r"trans(fer|ition)", false, true, SearchScope::Values);
        assert!(error.is_none());
        assert_eq!(
            results.len(),
            3,
            "Regex in Values scope should find 3 nodes"
        );
    }

    /// 8. Empty query returns no results regardless of scope.
    #[test]
    fn test_search_scope_empty_query() {
        let tree = tran_tree();
        for scope in [SearchScope::All, SearchScope::Keys, SearchScope::Values] {
            let (results, error) = search_nodes(&tree, "", false, false, scope);
            assert!(error.is_none());
            assert!(
                results.is_empty(),
                "{:?} scope with empty query should return no results",
                scope
            );
        }
    }

    /// 9. A node where both key AND value match is only returned once per scope.
    #[test]
    fn test_search_scope_both_key_and_value_match_no_duplication() {
        // {"transfer": "transfer credit"} – key="transfer", value="transfer credit"
        let tree = build_tree(&json!({"transfer": "transfer credit"}));

        // All: key matches first → node added once via `continue`, value not re-checked
        let (results, _) = search_nodes(&tree, "transfer", false, false, SearchScope::All);
        assert_eq!(
            results.len(),
            1,
            "All scope: node must appear exactly once even if key and value both match"
        );

        // Keys: finds the node via key
        let (results, _) = search_nodes(&tree, "transfer", false, false, SearchScope::Keys);
        assert_eq!(results.len(), 1, "Keys scope: exactly 1 result");

        // Values: finds the node via value ("transfer credit")
        let (results, _) = search_nodes(&tree, "transfer", false, false, SearchScope::Values);
        assert_eq!(results.len(), 1, "Values scope: exactly 1 result");
    }

    /// 10. Simulates the original user's scenario: many key-only matches swamp the
    ///     results; Scope::Values isolates the meaningful value match.
    #[test]
    fn test_search_scope_simulates_user_scenario() {
        // Build an array of 10 course objects (each has crsTransferStatus key with
        // empty value) plus one course whose description mentions "Transition".
        let mut courses: Vec<serde_json::Value> = (0..10)
            .map(|_| json!({ "crsTransferStatus": "", "name": "Some Course" }))
            .collect();
        courses.push(json!({
            "crsTransferStatus": "",
            "name": "Transition to Teaching",
            "description": "Transition to Teaching"
        }));

        let tree = build_tree(&json!({ "courses": courses }));

        let (all_results, _) = search_nodes(&tree, "tran", false, false, SearchScope::All);
        let (key_results, _) = search_nodes(&tree, "tran", false, false, SearchScope::Keys);
        let (value_results, _) = search_nodes(&tree, "tran", false, false, SearchScope::Values);

        // Keys: only the "crsTransferStatus" key matches "tran" — 11 courses × 1 key = 11.
        // The key "name" does NOT contain "tran"; only its *value* does.
        assert_eq!(
            key_results.len(),
            11,
            "Keys scope: 11 crsTransferStatus keys (one per course)"
        );

        // Values: the name and description values of the last course both contain "Transition"
        assert_eq!(
            value_results.len(),
            2,
            "Values scope: only description + name values in last course"
        );

        // All: 11 key matches (crsTransferStatus × 11) + 2 pure-value matches
        //      (name + description of last course, whose keys don't match "tran")
        assert_eq!(
            all_results.len(),
            13,
            "All scope: 11 key + 2 value-only matches"
        );
    }

    // =========================================================================
    // highlight_segments tests
    // =========================================================================

    /// Empty query → single non-highlighted segment.
    #[test]
    fn test_highlight_segments_empty_query() {
        let segs = highlight_segments("crsTransferStatus", "", false, false);
        assert_eq!(segs, vec![("crsTransferStatus".to_string(), false)]);
    }

    /// Empty haystack → single non-highlighted empty segment.
    #[test]
    fn test_highlight_segments_empty_haystack() {
        let segs = highlight_segments("", "tran", false, false);
        assert_eq!(segs, vec![("".to_string(), false)]);
    }

    /// No match → single non-highlighted segment with the full string.
    #[test]
    fn test_highlight_segments_no_match() {
        let segs = highlight_segments("hello world", "xyz", false, false);
        assert_eq!(segs, vec![("hello world".to_string(), false)]);
    }

    /// Match at the very start of the string.
    #[test]
    fn test_highlight_segments_match_at_start() {
        let segs = highlight_segments("transfer credit", "trans", false, false);
        assert_eq!(segs, vec![
            ("trans".to_string(), true),
            ("fer credit".to_string(), false),
        ]);
    }

    /// Match at the very end of the string.
    #[test]
    fn test_highlight_segments_match_at_end() {
        let segs = highlight_segments("credit transfer", "transfer", false, false);
        assert_eq!(segs, vec![
            ("credit ".to_string(), false),
            ("transfer".to_string(), true),
        ]);
    }

    /// Match in the middle of the string.
    #[test]
    fn test_highlight_segments_match_in_middle() {
        let segs = highlight_segments("crsTransferStatus", "tran", false, false);
        assert_eq!(segs, vec![
            ("crs".to_string(), false),
            ("Tran".to_string(), true),   // original casing preserved
            ("sferStatus".to_string(), false),
        ]);
    }

    /// Multiple non-overlapping matches in one string.
    #[test]
    fn test_highlight_segments_multiple_matches() {
        let segs = highlight_segments("tran and TRAN and tran", "tran", false, false);
        assert_eq!(segs, vec![
            ("tran".to_string(), true),
            (" and ".to_string(), false),
            ("TRAN".to_string(), true),
            (" and ".to_string(), false),
            ("tran".to_string(), true),
        ]);
    }

    /// Case-sensitive mode: only exact case matches.
    #[test]
    fn test_highlight_segments_case_sensitive() {
        // "TRAN" should NOT match "tran" in case-sensitive mode
        let segs = highlight_segments("crsTransferStatus", "TRAN", true, false);
        assert_eq!(segs, vec![("crsTransferStatus".to_string(), false)]);

        // "Tran" should match exactly
        let segs = highlight_segments("crsTransferStatus", "Tran", true, false);
        assert_eq!(segs, vec![
            ("crs".to_string(), false),
            ("Tran".to_string(), true),
            ("sferStatus".to_string(), false),
        ]);
    }

    /// Original casing is always preserved in output, even for case-insensitive matches.
    #[test]
    fn test_highlight_segments_preserves_original_casing() {
        let segs = highlight_segments("John TRAN", "tran", false, false);
        assert_eq!(segs, vec![
            ("John ".to_string(), false),
            ("TRAN".to_string(), true),  // "TRAN" preserved, not lowercased to "tran"
        ]);
    }

    /// Regex mode matches and segments correctly.
    #[test]
    fn test_highlight_segments_regex() {
        let segs = highlight_segments("transition and transfer", r"trans(ition|fer)", false, true);
        assert_eq!(segs, vec![
            ("transition".to_string(), true),
            (" and ".to_string(), false),
            ("transfer".to_string(), true),
        ]);
    }

    /// Invalid regex → single non-highlighted segment (graceful fallback).
    #[test]
    fn test_highlight_segments_invalid_regex() {
        let segs = highlight_segments("some text", r"[invalid", false, true);
        assert_eq!(segs, vec![("some text".to_string(), false)]);
    }

    /// Whole-string match → single highlighted segment, no surrounding empties.
    #[test]
    fn test_highlight_segments_full_string_match() {
        let segs = highlight_segments("TRAN", "tran", false, false);
        assert_eq!(segs, vec![("TRAN".to_string(), true)]);
    }
}
