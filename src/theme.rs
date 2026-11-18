//! Unfold's colors for GPUI.
//!
//! Syntax and row colors for the dark and light themes.
//! Buttons, inputs and dialogs come from gpui-component and follow its own
//! theme, which we switch to light or dark alongside ours (see `apply`).

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, Rgba, Window};
use unfold_core::flat_row::ValueType;

pub use unfold_core::config::AppTheme;

use crate::row::Span;

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Hsla {
    Rgba { r, g, b, a }.into()
}

fn rgb(r: f32, g: f32, b: f32) -> Hsla {
    rgba(r, g, b, 1.0)
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub key: Hsla,
    pub string: Hsla,
    pub number: Hsla,
    pub boolean: Hsla,
    pub null: Hsla,
    pub bracket: Hsla,
    pub indicator: Hsla,
    pub background: Hsla,
    pub toolbar_bg: Hsla,
    pub status_bar_bg: Hsla,
    pub row_odd: Hsla,
    pub search_match: Hsla,
    pub search_current: Hsla,
    pub search_highlight_text: Hsla,
    pub selected: Hsla,
    pub error: Hsla,
    pub error_context: Hsla,
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub link: Hsla,
    pub success: Hsla,
}

impl Colors {
    pub fn for_theme(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => Self::dark(),
            AppTheme::Light => Self::light(),
        }
    }

    pub fn dark() -> Self {
        Colors {
            key: rgb(0.4, 0.7, 0.9),
            string: rgb(0.6, 0.8, 0.5),
            number: rgb(0.9, 0.7, 0.4),
            boolean: rgb(0.8, 0.5, 0.7),
            null: rgb(0.6, 0.6, 0.6),
            bracket: rgb(0.7, 0.7, 0.7),
            indicator: rgb(0.5, 0.5, 0.5),
            background: rgb(0.12, 0.12, 0.12),
            toolbar_bg: rgb(0.12, 0.12, 0.12),
            status_bar_bg: rgb(0.15, 0.15, 0.15),
            row_odd: rgba(1.0, 1.0, 1.0, 0.03),
            search_match: rgba(0.9, 0.7, 0.2, 0.3),
            search_current: rgba(0.9, 0.5, 0.1, 0.5),
            search_highlight_text: rgb(1.0, 0.85, 0.2),
            selected: rgba(0.3, 0.5, 0.8, 0.3),
            error: rgb(0.9, 0.4, 0.4),
            error_context: rgb(0.7, 0.7, 0.5),
            text_primary: rgb(1.0, 1.0, 1.0),
            text_secondary: rgb(0.7, 0.7, 0.7),
            link: rgb(0.3, 0.5, 0.8),
            success: rgb(0.2, 0.7, 0.3),
        }
    }

    pub fn light() -> Self {
        Colors {
            key: rgb(0.0, 0.4, 0.7),
            string: rgb(0.2, 0.5, 0.2),
            number: rgb(0.8, 0.4, 0.0),
            boolean: rgb(0.6, 0.2, 0.6),
            null: rgb(0.5, 0.5, 0.5),
            bracket: rgb(0.3, 0.3, 0.3),
            indicator: rgb(0.6, 0.6, 0.6),
            background: rgb(0.98, 0.98, 0.98),
            toolbar_bg: rgb(0.94, 0.94, 0.94),
            status_bar_bg: rgb(0.90, 0.90, 0.90),
            row_odd: rgba(0.0, 0.0, 0.0, 0.03),
            search_match: rgba(1.0, 0.9, 0.4, 0.5),
            search_current: rgba(1.0, 0.6, 0.2, 0.6),
            search_highlight_text: rgb(0.7, 0.35, 0.0),
            selected: rgba(0.3, 0.5, 0.8, 0.2),
            error: rgb(0.8, 0.2, 0.2),
            error_context: rgb(0.6, 0.5, 0.2),
            text_primary: rgb(0.1, 0.1, 0.1),
            text_secondary: rgb(0.4, 0.4, 0.4),
            link: rgb(0.3, 0.5, 0.8),
            success: rgb(0.2, 0.6, 0.3),
        }
    }

    pub fn span(&self, span: Span) -> Hsla {
        match span {
            Span::Bracket => self.bracket,
            Span::Indicator => self.indicator,
            Span::Key => self.key,
            Span::Match => self.search_highlight_text,
            Span::Value(v) => match v {
                ValueType::Null => self.null,
                ValueType::Bool => self.boolean,
                ValueType::Number => self.number,
                ValueType::String => self.string,
                ValueType::Bracket => self.bracket,
                ValueType::Key => self.key,
            },
        }
    }
}

/// Switch gpui-component's theme (buttons, inputs, dialogs) to match ours.
pub fn apply(theme: AppTheme, window: Option<&mut Window>, cx: &mut App) {
    let mode = match theme {
        AppTheme::Dark => ThemeMode::Dark,
        AppTheme::Light => ThemeMode::Light,
    };
    Theme::change(mode, window, cx);
}

pub fn toggled(theme: AppTheme) -> AppTheme {
    match theme {
        AppTheme::Dark => AppTheme::Light,
        AppTheme::Light => AppTheme::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_differ() {
        assert_ne!(Colors::dark().background, Colors::light().background);
        assert_eq!(toggled(toggled(AppTheme::Dark)), AppTheme::Dark);
    }
}
