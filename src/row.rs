//! Turns a core `FlatRow` into one line of text plus colored ranges.
//!
//! GPUI can draw a whole line with a single `StyledText` and a list of
//! `(byte range, style)` highlights. That is much cheaper than one element per
//! colored piece, so each visible row becomes exactly
//! one text element.
//!
//! This module is pure data: no GPUI types, so it is easy to unit test. The
//! view maps each [`Span`] to a theme color.

use std::ops::Range;
use unfold_core::flat_row::{FlatRow, ValueType};
use unfold_core::search;

/// What a piece of the row is, so the view can pick a color for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    /// Tree lines (`│ ├ └ ─`) and the `: ` separator
    Bracket,
    /// The `⊞` / `⊟` expand indicator
    Indicator,
    Key,
    Value(ValueType),
    /// Part of the key or value that matches the search query
    Match,
}

/// One row, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct RowText {
    pub text: String,
    pub spans: Vec<(Range<usize>, Span)>,
}

/// Search settings needed to highlight matches inside a row.
pub struct Highlight<'a> {
    pub query: &'a str,
    pub case_sensitive: bool,
    pub use_regex: bool,
}

/// Build the text and color spans for a row.
///
/// `highlight` is `Some` only for rows that are search matches, which keeps
/// the regex work off every other row.
pub fn build_row_text(row: &FlatRow, highlight: Option<&Highlight>) -> RowText {
    let mut out = RowText {
        text: String::with_capacity(row.prefix.len() + row.value_display.len() + 32),
        spans: Vec::with_capacity(6),
    };

    out.push(&row.prefix, Span::Bracket);
    if row.is_expandable {
        out.push(if row.is_expanded { "⊟ " } else { "⊞ " }, Span::Indicator);
    } else {
        out.push("─ ", Span::Bracket);
    }

    if let Some(key) = &row.key {
        let key = if key.is_empty() { "\"\"" } else { key.as_str() };
        out.push_highlighted(key, Span::Key, highlight);
        out.push(": ", Span::Bracket);
    }

    // An expanded container shows only its key; its children carry the content.
    if !(row.is_expandable && row.is_expanded) {
        out.push_highlighted(&row.value_display, Span::Value(row.value_type), highlight);
    }

    out
}

impl RowText {
    fn push(&mut self, s: &str, span: Span) {
        if s.is_empty() {
            return;
        }
        let start = self.text.len();
        self.text.push_str(s);
        self.spans.push((start..self.text.len(), span));
    }

    fn push_highlighted(&mut self, s: &str, span: Span, highlight: Option<&Highlight>) {
        let Some(h) = highlight.filter(|h| !h.query.is_empty()) else {
            self.push(s, span);
            return;
        };
        for (segment, is_match) in
            search::highlight_segments(s, h.query, h.case_sensitive, h.use_regex)
        {
            self.push(&segment, if is_match { Span::Match } else { span });
        }
    }
}

/// Rough display width of a row in characters, used to find the widest row
/// for horizontal scrolling. Counting chars (not bytes) keeps `│` at width 1.
pub fn row_char_width(row: &FlatRow) -> usize {
    let key = row.key.as_ref().map(|k| k.chars().count() + 2).unwrap_or(0);
    let value = if row.is_expandable && row.is_expanded {
        0
    } else {
        row.value_display.chars().count()
    };
    row.prefix.chars().count() + 2 + key + value
}

/// Index of the widest row, or `None` when there are no rows.
pub fn widest_row(rows: &[FlatRow]) -> Option<usize> {
    rows.iter()
        .enumerate()
        .max_by_key(|(_, r)| row_char_width(r))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use unfold_core::flat_row::flatten_visible_nodes;
    use unfold_core::parser::build_tree;

    fn rows(value: serde_json::Value) -> Vec<FlatRow> {
        flatten_visible_nodes(&build_tree(&value))
    }

    fn slice(r: &RowText, span: Span) -> Vec<&str> {
        r.spans
            .iter()
            .filter(|(_, s)| *s == span)
            .map(|(range, _)| &r.text[range.clone()])
            .collect()
    }

    #[test]
    fn leaf_row_has_connector_key_and_value() {
        let rows = rows(json!({"name": "Ada"}));
        let r = build_row_text(&rows[0], None);
        assert_eq!(r.text, "└─ name: \"Ada\"");
        assert_eq!(slice(&r, Span::Key), vec!["name"]);
        assert_eq!(slice(&r, Span::Value(ValueType::String)), vec!["\"Ada\""]);
    }

    #[test]
    fn collapsed_container_shows_indicator_and_summary() {
        let rows = rows(json!({"a": [1, 2]}));
        let r = build_row_text(&rows[0], None);
        assert_eq!(r.text, "└⊞ a: [...]");
        assert_eq!(slice(&r, Span::Indicator), vec!["⊞ "]);
    }

    #[test]
    fn empty_key_is_shown_as_quotes() {
        let rows = rows(json!({"": 1}));
        let r = build_row_text(&rows[0], None);
        assert_eq!(slice(&r, Span::Key), vec!["\"\""]);
    }

    #[test]
    fn spans_cover_the_text_without_gaps() {
        let rows = rows(json!({"k": {"x": true}, "z": null}));
        for row in &rows {
            let r = build_row_text(row, None);
            let mut end = 0;
            for (range, _) in &r.spans {
                assert_eq!(range.start, end);
                end = range.end;
            }
            assert_eq!(end, r.text.len());
        }
    }

    #[test]
    fn search_match_is_split_into_its_own_span() {
        let rows = rows(json!({"username": "alpha"}));
        let h = Highlight {
            query: "name",
            case_sensitive: false,
            use_regex: false,
        };
        let r = build_row_text(&rows[0], Some(&h));
        assert_eq!(slice(&r, Span::Match), vec!["name"]);
        assert_eq!(slice(&r, Span::Key), vec!["user"]);
    }

    #[test]
    fn widest_row_counts_chars_not_bytes() {
        let rows = rows(json!({"a": "ééééé", "b": "xxxxxxx"}));
        assert_eq!(widest_row(&rows), Some(1));
        assert_eq!(widest_row(&[]), None);
    }
}
