//! Search functionality for the JSON tree.
//!
//! Supports plain text and regex search with case sensitivity options.
//! Search always checks both keys and values simultaneously.

use crate::parser::{JsonTree, JsonValue};
use regex::Regex;

/// Search all nodes in the tree for matches against both keys and values.
///
/// Returns `(results, error_message)` where `error_message` is `Some` if the
/// regex pattern is invalid.
///
/// # Arguments
/// * `tree`           – the parsed JSON tree
/// * `query`          – the search string (or regex pattern)
/// * `case_sensitive` – whether the match is case-sensitive
/// * `use_regex`      – interpret `query` as a regular expression
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

        // --- Key check ---
        if let Some(key) = &node.key
            && matches(key)
        {
            results.push(i);
            continue; // key matched – no need to also check the value
        }

        // --- Value check ---
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
    // Shared helper: build a tree with both key and value matches for "tran":
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
    // Expected total matches: 6
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
    // search_nodes tests
    // =========================================================================

    #[test]
    fn test_search_nodes_basic() {
        let value = json!({"name": "Unfold", "version": "1.0"});
        let tree = build_tree(&value);

        // Search for "Unfold" (appears in value)
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

        // Case insensitive should find key "Name" via "name"
        let (results, _) = search_nodes(&tree, "name", false, false);
        assert!(!results.is_empty());

        // Case sensitive should not find lowercase "name" when key is "Name"
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

    /// Search always checks both keys and values.
    #[test]
    fn test_search_matches_keys_and_values() {
        let tree = tran_tree();
        let (results, error) = search_nodes(&tree, "tran", false, false);
        assert!(error.is_none());
        assert_eq!(
            results.len(),
            6,
            "Should find 6 nodes (2 key matches + 4 value matches)"
        );
    }

    /// A node where both key AND value match is only returned once (no duplicates).
    #[test]
    fn test_search_no_duplicate_when_key_and_value_both_match() {
        // {"transfer": "transfer credit"} – key="transfer", value contains "transfer"
        let tree = build_tree(&json!({"transfer": "transfer credit"}));
        let (results, _) = search_nodes(&tree, "transfer", false, false);
        assert_eq!(
            results.len(),
            1,
            "Node must appear exactly once even if key and value both match"
        );
    }

    /// Empty query returns no results.
    #[test]
    fn test_search_empty_query() {
        let tree = tran_tree();
        let (results, error) = search_nodes(&tree, "", false, false);
        assert!(error.is_none());
        assert!(results.is_empty(), "Empty query should return no results");
    }

    /// Case-sensitive search distinguishes casing in both keys and values.
    #[test]
    fn test_search_case_sensitive_keys_and_values() {
        let tree = tran_tree();

        // "Tran" (capital T, case-sensitive) matches:
        //   - key "crsTransferStatus"  (contains "Transfer" → "Tran")
        //   - key "innerTransfer"      (contains "Transfer" → "Tran")
        //   - value "John Tran"        (exact "Tran")
        let (results, _) = search_nodes(&tree, "Tran", true, false);
        assert_eq!(
            results.len(),
            3,
            "Case-sensitive 'Tran' should find 2 key matches + 1 value match"
        );

        // "TRAN" (all caps, case-sensitive) → no match anywhere
        let (results, _) = search_nodes(&tree, "TRAN", true, false);
        assert!(results.is_empty(), "Case-sensitive 'TRAN' should find nothing");
    }

    /// Regex search works against both keys and values.
    #[test]
    fn test_search_regex_keys_and_values() {
        let tree = tran_tree();
        // "transfer" in keys: crsTransferStatus, innerTransfer (2)
        // "transfer" or "transition" in values: description, creditType, innerField (3)
        // studentName "John Tran" doesn't match trans(fer|ition)
        let (results, error) = search_nodes(&tree, r"trans(fer|ition)", false, true);
        assert!(error.is_none());
        assert_eq!(
            results.len(),
            5,
            "Regex should find 2 key matches + 3 value matches"
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
