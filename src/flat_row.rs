//! Iced-specific parts of row rendering.
//!
//! The row data itself (`FlatRow`, `ValueType`, flattening) lives in `unfold-core`.
//! This module holds what only makes sense for the Iced UI: pixel sizes and colors.

use crate::theme::ThemeColors;
use iced::Color;
use unfold_core::flat_row::ValueType;

/// Virtual scrolling constants
pub const ROW_HEIGHT: f32 = 16.0; // Fixed height per row (tight for connected tree lines)
pub const BUFFER_ROWS: usize = 5; // Extra rows above/below (reduced for performance)

/// Adds a theme-aware color lookup to the core `ValueType`.
///
/// `ValueType` is defined in another crate, so Rust does not let us add a method to
/// it directly (the "orphan rule"). A trait defined here lets us call
/// `value_type.color(&colors)` anyway, as long as the trait is imported.
pub trait ValueTypeColor {
    /// Get the appropriate color for this value type given a theme
    fn color(&self, colors: &ThemeColors) -> Color;
}

impl ValueTypeColor for ValueType {
    fn color(&self, colors: &ThemeColors) -> Color {
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
