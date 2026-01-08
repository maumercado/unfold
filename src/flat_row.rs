//! FlatRow and related types for virtual scrolling.
//!
//! Pre-computed row data for efficient rendering of the JSON tree.

use iced::Color;
use crate::theme::ThemeColors;

/// Virtual scrolling constants
pub const ROW_HEIGHT: f32 = 16.0;      // Fixed height per row (tight for connected tree lines)
pub const BUFFER_ROWS: usize = 5;      // Extra rows above/below (reduced for performance)

/// Value type for theme-aware coloring
#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Bracket,
    Key,
}

impl ValueType {
    /// Get the appropriate color for this value type given a theme
    pub fn color(&self, colors: &ThemeColors) -> Color {
        match self {
            ValueType::Null => colors.null,
            ValueType::Bool => colors.boolean,
            ValueType::Number => colors.number,
            ValueType::String => colors.string,
            ValueType::Bracket => colors.bracket,
            ValueType::Key => colors.key,
        }
    }
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
