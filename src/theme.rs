//! Theme system for the Unfold application.
//!
//! Provides dark and light color schemes with consistent styling across all UI components.

use iced::widget::button;
use iced::{Border, Color, Shadow};
use iced::border::Radius;
use iced::widget::button::Status as ButtonStatus;

/// Application theme selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Dark,
    Light,
}

/// All theme-dependent colors in one place
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    // Syntax highlighting
    pub key: Color,
    pub string: Color,
    pub number: Color,
    pub boolean: Color,
    pub null: Color,
    pub bracket: Color,
    pub indicator: Color,
    // UI colors
    pub background: Color,
    pub toolbar_bg: Color,
    pub status_bar_bg: Color,
    pub row_odd: Color,
    pub search_match: Color,
    pub search_current: Color,
    pub selected: Color,
    pub error: Color,
    pub error_context: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    // Button colors
    pub btn_bg: Color,
    pub btn_bg_hover: Color,
    pub btn_border_top: Color,
    pub btn_border_bottom: Color,
    pub btn_disabled: Color,
    pub btn_active_bg: Color,
    pub btn_active_border: Color,
}

impl ThemeColors {
    pub fn dark() -> Self {
        ThemeColors {
            // Syntax highlighting (bright on dark)
            key: Color::from_rgb(0.4, 0.7, 0.9),
            string: Color::from_rgb(0.6, 0.8, 0.5),
            number: Color::from_rgb(0.9, 0.7, 0.4),
            boolean: Color::from_rgb(0.8, 0.5, 0.7),
            null: Color::from_rgb(0.6, 0.6, 0.6),
            bracket: Color::from_rgb(0.7, 0.7, 0.7),
            indicator: Color::from_rgb(0.5, 0.5, 0.5),
            // UI colors
            background: Color::from_rgb(0.12, 0.12, 0.12),
            toolbar_bg: Color::from_rgb(0.12, 0.12, 0.12),
            status_bar_bg: Color::from_rgb(0.15, 0.15, 0.15),
            row_odd: Color::from_rgba(1.0, 1.0, 1.0, 0.03),
            search_match: Color::from_rgba(0.9, 0.7, 0.2, 0.3),
            search_current: Color::from_rgba(0.9, 0.5, 0.1, 0.5),
            selected: Color::from_rgba(0.3, 0.5, 0.8, 0.3),
            error: Color::from_rgb(0.9, 0.4, 0.4),
            error_context: Color::from_rgb(0.7, 0.7, 0.5),
            text_primary: Color::WHITE,
            text_secondary: Color::from_rgb(0.7, 0.7, 0.7),
            // Button colors
            btn_bg: Color::from_rgb(0.28, 0.28, 0.30),
            btn_bg_hover: Color::from_rgb(0.32, 0.32, 0.35),
            btn_border_top: Color::from_rgb(0.45, 0.45, 0.48),
            btn_border_bottom: Color::from_rgb(0.15, 0.15, 0.17),
            btn_disabled: Color::from_rgb(0.22, 0.22, 0.24),
            btn_active_bg: Color::from_rgb(0.3, 0.5, 0.7),
            btn_active_border: Color::from_rgb(0.4, 0.6, 0.8),
        }
    }

    pub fn light() -> Self {
        ThemeColors {
            // Syntax highlighting (darker for light background)
            key: Color::from_rgb(0.0, 0.4, 0.7),
            string: Color::from_rgb(0.2, 0.5, 0.2),
            number: Color::from_rgb(0.8, 0.4, 0.0),
            boolean: Color::from_rgb(0.6, 0.2, 0.6),
            null: Color::from_rgb(0.5, 0.5, 0.5),
            bracket: Color::from_rgb(0.3, 0.3, 0.3),
            indicator: Color::from_rgb(0.6, 0.6, 0.6),
            // UI colors
            background: Color::from_rgb(0.98, 0.98, 0.98),
            toolbar_bg: Color::from_rgb(0.94, 0.94, 0.94),
            status_bar_bg: Color::from_rgb(0.90, 0.90, 0.90),
            row_odd: Color::from_rgba(0.0, 0.0, 0.0, 0.03),
            search_match: Color::from_rgba(1.0, 0.9, 0.4, 0.5),
            search_current: Color::from_rgba(1.0, 0.6, 0.2, 0.6),
            selected: Color::from_rgba(0.3, 0.5, 0.8, 0.2),
            error: Color::from_rgb(0.8, 0.2, 0.2),
            error_context: Color::from_rgb(0.6, 0.5, 0.2),
            text_primary: Color::from_rgb(0.1, 0.1, 0.1),
            text_secondary: Color::from_rgb(0.4, 0.4, 0.4),
            // Button colors (lighter)
            btn_bg: Color::from_rgb(0.88, 0.88, 0.90),
            btn_bg_hover: Color::from_rgb(0.82, 0.82, 0.85),
            btn_border_top: Color::from_rgb(0.95, 0.95, 0.98),
            btn_border_bottom: Color::from_rgb(0.70, 0.70, 0.72),
            btn_disabled: Color::from_rgb(0.92, 0.92, 0.94),
            btn_active_bg: Color::from_rgb(0.4, 0.6, 0.85),
            btn_active_border: Color::from_rgb(0.3, 0.5, 0.75),
        }
    }
}

/// Get theme colors for the given theme
pub fn get_theme_colors(theme: AppTheme) -> ThemeColors {
    match theme {
        AppTheme::Dark => ThemeColors::dark(),
        AppTheme::Light => ThemeColors::light(),
    }
}

/// Custom 3D button style with raised appearance (theme-aware)
pub fn button_3d_style_themed(colors: ThemeColors) -> impl Fn(&iced::Theme, ButtonStatus) -> button::Style {
    move |_theme: &iced::Theme, status: ButtonStatus| {
        let (bg_color, text_color, border_color) = match status {
            ButtonStatus::Active => (colors.btn_bg, colors.text_primary, colors.btn_border_top),
            ButtonStatus::Hovered => (colors.btn_bg_hover, colors.text_primary, colors.btn_border_top),
            ButtonStatus::Pressed => (colors.btn_border_bottom, colors.text_secondary, colors.btn_border_bottom),
            ButtonStatus::Disabled => (colors.btn_disabled, colors.text_secondary, colors.btn_disabled),
        };

        button::Style {
            background: Some(bg_color.into()),
            text_color,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 3.0,
            },
            snap: true,
        }
    }
}

/// Toggle button style - highlighted when active (theme-aware)
pub fn button_toggle_style_themed(is_active: bool, colors: ThemeColors) -> impl Fn(&iced::Theme, ButtonStatus) -> button::Style {
    move |_theme: &iced::Theme, status: ButtonStatus| {
        let (bg_color, text_color, border_color) = match (is_active, status) {
            (true, ButtonStatus::Active) => (colors.btn_active_bg, colors.text_primary, colors.btn_active_border),
            (true, ButtonStatus::Hovered) => (colors.btn_active_bg, colors.text_primary, colors.btn_active_border),
            (true, ButtonStatus::Pressed) => (colors.btn_active_bg, colors.text_primary, colors.btn_active_border),
            (false, ButtonStatus::Active) => (colors.btn_bg, colors.text_secondary, colors.btn_border_top),
            (false, ButtonStatus::Hovered) => (colors.btn_bg_hover, colors.text_primary, colors.btn_border_top),
            (false, ButtonStatus::Pressed) => (colors.btn_border_bottom, colors.text_secondary, colors.btn_border_bottom),
            (_, ButtonStatus::Disabled) => (colors.btn_disabled, colors.text_secondary, colors.btn_disabled),
        };

        button::Style {
            background: Some(bg_color.into()),
            text_color,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            snap: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        // Test that theme colors are valid
        let dark = get_theme_colors(AppTheme::Dark);
        let light = get_theme_colors(AppTheme::Light);

        // Colors should be different between themes
        assert_ne!(dark.background, light.background);
        assert_ne!(dark.text_primary, light.text_primary);
    }
}
