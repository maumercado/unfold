mod parser;

use iced::widget::{button, column, container, mouse_area, row, scrollable, stack, text, text_input, Space};
use iced::widget::scrollable::Viewport;
use iced::{Element, Font, Length, Center, Fill, Color, Size, Task, window, Border, Shadow, Subscription, clipboard, Theme, event, Event};
use iced::border::Radius;
use iced::advanced::widget::{Id as WidgetId, operate};
use iced::advanced::widget::operation::scrollable::{scroll_to, AbsoluteOffset};
use iced::advanced::widget::operation::focusable;
use iced::keyboard::{self, Key, Modifiers, key::Named};
use std::collections::HashSet;
use regex::Regex;
use serde::Deserialize;
use semver::Version;

// Native menu support
use muda::{
    Menu, Submenu, MenuItem, PredefinedMenuItem, MenuEvent,
    accelerator::{Accelerator, Modifiers as MudaModifiers, Code},
    AboutMetadata,
};

/// Menu item identifiers for handling events
mod menu_ids {
    // Menu bar items
    pub const CHECK_UPDATES: &str = "check_updates";
    pub const OPEN_FILE: &str = "open_file";
    pub const OPEN_NEW_WINDOW: &str = "open_new_window";
    pub const OPEN_EXTERNAL: &str = "open_external";
    pub const COPY_VALUE: &str = "copy_value";
    pub const COPY_KEY: &str = "copy_key";
    pub const COPY_PATH: &str = "copy_path";
    pub const TOGGLE_THEME: &str = "toggle_theme";
    pub const KEYBOARD_SHORTCUTS: &str = "keyboard_shortcuts";
    // Context menu items
    pub const EXPORT_JSON: &str = "export_json";
    pub const EXPAND_ALL: &str = "expand_all";
    pub const COLLAPSE_ALL: &str = "collapse_all";
}

/// State for the update check dialog
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateCheckState {
    /// Not checking, dialog not shown
    None,
    /// Currently fetching from GitHub
    Checking,
    /// Update available with version string and release URL
    UpdateAvailable { version: String, release_url: String },
    /// Already on latest version
    UpToDate,
    /// Error occurred during check
    Error(String),
}

/// GitHub release API response (partial)
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

/// Create the native application menu bar
fn create_app_menu() -> Menu {
    let menu = Menu::new();

    // ===== App Menu (macOS) =====
    #[cfg(target_os = "macos")]
    {
        let app_menu = Submenu::new("Unfold", true);
        let _ = app_menu.append_items(&[
            &PredefinedMenuItem::about(None, Some(AboutMetadata {
                name: Some("Unfold".into()),
                version: Some(env!("CARGO_PKG_VERSION").into()),
                copyright: Some("Copyright 2025 Mauricio Mercado".into()),
                ..Default::default()
            })),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(
                menu_ids::CHECK_UPDATES,
                "Check for Updates...",
                true,
                None::<Accelerator>,
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ]);
        let _ = menu.append(&app_menu);
    }

    // ===== File Menu =====
    let file_menu = Submenu::new("File", true);
    let _ = file_menu.append_items(&[
        &MenuItem::with_id(
            menu_ids::OPEN_FILE,
            "Open...",
            true,
            Some(Accelerator::new(Some(MudaModifiers::SUPER), Code::KeyO)),
        ),
        &MenuItem::with_id(
            menu_ids::OPEN_NEW_WINDOW,
            "Open in New Window...",
            true,
            Some(Accelerator::new(Some(MudaModifiers::SUPER), Code::KeyN)),
        ),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(
            menu_ids::OPEN_EXTERNAL,
            "Open in External Editor",
            true,
            Some(Accelerator::new(Some(MudaModifiers::SUPER | MudaModifiers::SHIFT), Code::KeyE)),
        ),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::close_window(None),
    ]);
    let _ = menu.append(&file_menu);

    // ===== Edit Menu =====
    let edit_menu = Submenu::new("Edit", true);
    let _ = edit_menu.append_items(&[
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(
            menu_ids::COPY_VALUE,
            "Copy Value",
            true,
            Some(Accelerator::new(Some(MudaModifiers::SUPER), Code::KeyC)),
        ),
        &MenuItem::with_id(
            menu_ids::COPY_KEY,
            "Copy Key",
            true,
            Some(Accelerator::new(
                Some(MudaModifiers::SUPER | MudaModifiers::SHIFT),
                Code::KeyC,
            )),
        ),
        &MenuItem::with_id(
            menu_ids::COPY_PATH,
            "Copy Path",
            true,
            Some(Accelerator::new(
                Some(MudaModifiers::SUPER | MudaModifiers::ALT),
                Code::KeyC,
            )),
        ),
    ]);
    let _ = menu.append(&edit_menu);

    // ===== View Menu =====
    let view_menu = Submenu::new("View", true);
    let _ = view_menu.append_items(&[
        &MenuItem::with_id(
            menu_ids::TOGGLE_THEME,
            "Toggle Theme",
            true,
            Some(Accelerator::new(Some(MudaModifiers::SUPER), Code::KeyT)),
        ),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::fullscreen(None),
    ]);
    let _ = menu.append(&view_menu);

    // ===== Window Menu (macOS) =====
    #[cfg(target_os = "macos")]
    {
        let window_menu = Submenu::new("Window", true);
        let _ = window_menu.append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::bring_all_to_front(None),
        ]);
        let _ = menu.append(&window_menu);
    }

    // ===== Help Menu =====
    let help_menu = Submenu::new("Help", true);
    let _ = help_menu.append_items(&[
        &MenuItem::with_id(
            menu_ids::KEYBOARD_SHORTCUTS,
            "Keyboard Shortcuts",
            true,
            Some(Accelerator::new(Some(MudaModifiers::SUPER), Code::Slash)),
        ),
    ]);
    let _ = menu.append(&help_menu);

    menu
}

/// Create context menu for right-click on nodes
fn create_context_menu() -> Menu {
    let menu = Menu::new();
    let _ = menu.append_items(&[
        &MenuItem::with_id(
            menu_ids::COPY_VALUE,
            "Copy Value",
            true,
            Some(Accelerator::new(Some(MudaModifiers::SUPER), Code::KeyC)),
        ),
        &MenuItem::with_id(
            menu_ids::COPY_KEY,
            "Copy Key",
            true,
            Some(Accelerator::new(
                Some(MudaModifiers::SUPER | MudaModifiers::SHIFT),
                Code::KeyC,
            )),
        ),
        &MenuItem::with_id(
            menu_ids::COPY_PATH,
            "Copy Path",
            true,
            Some(Accelerator::new(
                Some(MudaModifiers::SUPER | MudaModifiers::ALT),
                Code::KeyC,
            )),
        ),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(menu_ids::EXPORT_JSON, "Export JSON...", true, None::<Accelerator>),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(menu_ids::EXPAND_ALL, "Expand All Children", true, None::<Accelerator>),
        &MenuItem::with_id(menu_ids::COLLAPSE_ALL, "Collapse All Children", true, None::<Accelerator>),
    ]);
    menu
}

// Theme system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTheme {
    Dark,
    Light,
}

// All theme-dependent colors in one place
#[derive(Debug, Clone, Copy)]
struct ThemeColors {
    // Syntax highlighting
    key: Color,
    string: Color,
    number: Color,
    boolean: Color,
    null: Color,
    bracket: Color,
    indicator: Color,
    // UI colors
    background: Color,
    toolbar_bg: Color,
    status_bar_bg: Color,
    row_odd: Color,
    search_match: Color,
    search_current: Color,
    selected: Color,
    error: Color,
    error_context: Color,
    text_primary: Color,
    text_secondary: Color,
    // Button colors
    btn_bg: Color,
    btn_bg_hover: Color,
    btn_border_top: Color,
    btn_border_bottom: Color,
    btn_disabled: Color,
    btn_active_bg: Color,
    btn_active_border: Color,
}

impl ThemeColors {
    fn dark() -> Self {
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

    fn light() -> Self {
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

fn get_theme_colors(theme: AppTheme) -> ThemeColors {
    match theme {
        AppTheme::Dark => ThemeColors::dark(),
        AppTheme::Light => ThemeColors::light(),
    }
}


// Structured parse error for better error display
#[derive(Debug, Clone)]
struct ParseError {
    message: String,
    line: Option<usize>,
    column: Option<usize>,
    context_line: Option<String>,  // The actual line from the file
    filename: String,
}

impl ParseError {
    fn from_serde_error(e: &serde_json::Error, contents: &str, filename: &str) -> Self {
        let line = e.line();
        let column = e.column();

        // Extract the problematic line from the file contents
        let context_line = contents
            .lines()
            .nth(line.saturating_sub(1))
            .map(|s| s.to_string());

        // Classify the error for a friendlier message
        let message = match e.classify() {
            serde_json::error::Category::Io => format!("I/O error: {}", e),
            serde_json::error::Category::Syntax => {
                // Extract just the syntax error description
                let full = e.to_string();
                // serde_json format: "message at line X column Y"
                if let Some(idx) = full.find(" at line ") {
                    full[..idx].to_string()
                } else {
                    full
                }
            }
            serde_json::error::Category::Data => format!("Data error: {}", e),
            serde_json::error::Category::Eof => "Unexpected end of file".to_string(),
        };

        ParseError {
            message,
            line: Some(line),
            column: Some(column),
            context_line,
            filename: filename.to_string(),
        }
    }
}

// Virtual scrolling constants
const ROW_HEIGHT: f32 = 16.0;      // Fixed height per row (tight for connected tree lines)
const BUFFER_ROWS: usize = 5;      // Extra rows above/below (reduced for performance)

use iced::widget::button::Status as ButtonStatus;

/// Custom 3D button style with raised appearance (theme-aware)
fn button_3d_style_themed(colors: ThemeColors) -> impl Fn(&iced::Theme, ButtonStatus) -> button::Style {
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
fn button_toggle_style_themed(is_active: bool, colors: ThemeColors) -> impl Fn(&iced::Theme, ButtonStatus) -> button::Style {
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

use parser::{JsonTree, JsonValue};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::env;
use std::process::Command;

/// A flattened row ready for rendering
/// Value type for theme-aware coloring
#[derive(Debug, Clone, Copy)]
enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Bracket,
    Key,
}

impl ValueType {
    /// Get the appropriate color for this value type given a theme
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

/// This pre-computes everything needed to render a single tree row
#[derive(Debug, Clone)]
struct FlatRow {
    /// Index in the original JsonTree (for toggle events)
    node_index: usize,
    /// Pre-built prefix string (tree lines: "│  ├─ ")
    prefix: String,
    /// The key to display (if any)
    key: Option<String>,
    /// The value to display (formatted string)
    value_display: String,
    /// Type of value (for theme-aware coloring)
    value_type: ValueType,
    /// Is this node expandable (has children)?
    is_expandable: bool,
    /// Is this node currently expanded?
    is_expanded: bool,
    /// Row index in flattened list (for zebra striping)
    row_index: usize,
    /// JSON path to this node (e.g., "users[2].email")
    path: String,
}

// Global menu storage - must persist for app lifetime
// Using thread_local since Menu is not Send/Sync
thread_local! {
    static APP_MENU: std::cell::RefCell<Option<Menu>> = std::cell::RefCell::new(None);
    static CONTEXT_MENU: std::cell::RefCell<Option<Menu>> = std::cell::RefCell::new(None);
    static MENU_INIT_COUNTER: std::cell::Cell<u32> = std::cell::Cell::new(0);
}

/// Initialize the native menu bar (called after a delay to ensure NSApp exists)
/// Returns true if menu was just initialized
fn try_initialize_menu() -> bool {
    MENU_INIT_COUNTER.with(|counter| {
        let count = counter.get();

        // Skip first few ticks to let Iced/winit fully initialize
        // Then initialize once on tick 3
        if count < 3 {
            counter.set(count + 1);
            false
        } else if count == 3 {
            counter.set(count + 1);

            let menu = create_app_menu();
            #[cfg(target_os = "macos")]
            menu.init_for_nsapp();

            APP_MENU.with(|m| *m.borrow_mut() = Some(menu));

            let context_menu = create_context_menu();
            CONTEXT_MENU.with(|m| *m.borrow_mut() = Some(context_menu));

            true
        } else {
            false
        }
    })
}

pub fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .window_size((900.0, 700.0))
        .resizable(true)
        .title(|app: &App| {
            match &app.current_file {
                Some(path) => {
                    let filename = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    format!("{} - Unfold", filename)
                }
                None => String::from("Unfold - JSON Viewer")
            }
        })
        // Subscriptions let us listen for external events like keyboard input
        .subscription(App::subscription)
        .run()
}

// The application state (Model)
struct App {
    tree: Option<JsonTree>,
    status: String,
    current_file: Option<PathBuf>,
    #[allow(dead_code)]
    preferences: Preferences,
    // Current theme (dark/light)
    theme: AppTheme,
    // Time taken to load and parse the file
    load_time: Option<Duration>,
    // Flattened rows for virtual scrolling (rebuilt when tree changes)
    flat_rows: Vec<FlatRow>,
    // Viewport height in pixels (updated on resize)
    viewport_height: f32,
    // Current scroll offset in pixels (for virtual scrolling)
    scroll_offset: f32,
    // Search state
    search_query: String,
    search_results: Vec<usize>,
    search_result_index: Option<usize>,
    search_matches: HashSet<usize>,
    search_case_sensitive: bool,
    search_use_regex: bool,
    search_regex_error: Option<String>,
    // Scrollable ID for programmatic scrolling
    tree_scrollable_id: WidgetId,
    // Search input ID for programmatic focus
    search_input_id: WidgetId,
    // Track current keyboard modifiers (for Shift+Enter in search input)
    current_modifiers: Modifiers,
    // Currently selected node (for copy, path display, etc.)
    selected_node: Option<usize>,
    // Parse error details (for better error display)
    parse_error: Option<ParseError>,
    // Show help overlay with keyboard shortcuts
    show_help: bool,
    // Context menu state: Some((node_index, x, y)) when visible
    context_menu_state: Option<(usize, f32, f32)>,
    // Which submenu is currently open
    context_submenu: ContextSubmenu,
    // Update check state for the dialog
    update_check_state: UpdateCheckState,
}

// User-configurable display preferences (for future use)
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Preferences {
    indent_size: usize,
    show_tree_lines: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            indent_size: 2,        // Default to 2 spaces like the reference
            show_tree_lines: true, // Show tree connector lines
        }
    }
}


// Messages that can be sent to update the app
// In Rust, enums can carry data - this is called "algebraic data types"
// Each variant can have different associated data
#[derive(Debug, Clone)]
enum Message {
    OpenFileDialog,
    FileSelected(Option<PathBuf>),
    FileDropped(PathBuf),
    ToggleNode(usize),
    Scrolled(Viewport),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    ToggleCaseSensitive,
    ToggleRegex,
    // Keyboard events - Key and Modifiers tell us what was pressed
    KeyPressed(Key, Modifiers),
    ModifiersChanged(Modifiers),
    ClearSearch,
    FocusSearch,
    // Search submit from text input (checks current_modifiers for Shift)
    SearchSubmit,
    // Open file dialog, then open selected file in new window
    OpenFileInNewWindow,
    // File was selected for opening in new window
    FileSelectedForNewWindow(Option<PathBuf>),
    // Select a node (for copy, path display)
    SelectNode(usize),
    // Copy selected node's value to clipboard
    CopySelectedValue,
    // Copy selected node's key/name to clipboard
    CopySelectedName,
    // Copy selected node's path to clipboard
    CopySelectedPath,
    // Toggle between dark and light theme
    ToggleTheme,
    // Show help overlay with shortcuts
    ToggleHelp,
    // No operation (used for menu event polling when no event)
    NoOp,
    // Check for updates from GitHub
    CheckForUpdates,
    // Result of update check
    UpdateCheckResult(UpdateCheckState),
    // Dismiss update dialog
    DismissUpdateDialog,
    // Open release URL in browser
    OpenReleaseUrl(String),
    // Export selected node to JSON file
    ExportJson,
    // Expand all children of selected node
    ExpandAllChildren,
    // Collapse all children of selected node
    CollapseAllChildren,
    // Open current file in external editor
    OpenInExternalEditor,
    // Show context menu at position (node_index, x, y)
    ShowContextMenu(usize, f32, f32),
    // Hide context menu
    HideContextMenu,
    // Open submenu
    OpenSubmenu(ContextSubmenu),
    // Copy value in specific format
    CopyValueMinified,
    CopyValueFormatted,
    // Export value in specific format
    ExportAsJson,
    ExportAsMinifiedJson,
    ExportAsFormattedJson,
}

/// Which submenu is currently open in context menu
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextSubmenu {
    None,
    CopyValueAs,
    ExportValueAs,
}

impl App {
    // Initialize the application (called once at startup)
    // Checks for CLI arguments: `unfold myfile.json`
    fn boot() -> (Self, Task<Message>) {
        // Note: Menu is initialized in main() before Iced starts
        let app = App {
            tree: None,
            status: String::from("No file loaded"),
            current_file: None,
            preferences: Preferences::default(),
            theme: AppTheme::Dark,
            load_time: None,
            flat_rows: Vec::new(),
            viewport_height: 600.0,
            scroll_offset: 0.0,
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_index: None,
            search_matches: HashSet::new(),
            search_case_sensitive: false,
            search_use_regex: false,
            search_regex_error: None,
            tree_scrollable_id: WidgetId::unique(),
            search_input_id: WidgetId::unique(),
            current_modifiers: Modifiers::default(),
            selected_node: None,
            parse_error: None,
            show_help: false,
            context_menu_state: None,
            context_submenu: ContextSubmenu::None,
            update_check_state: UpdateCheckState::None,
        };

        // Check if a file path was passed as CLI argument
        // std::env::args() returns an iterator over command-line arguments
        // First argument is the program name, second would be the file path
        let args: Vec<String> = env::args().collect();

        if args.len() > 1 {
            // User passed a file path - load it automatically
            let file_path = PathBuf::from(&args[1]);

            // Return a Task that sends FileSelected message
            // Task::done() creates an immediate task with the given message
            (app, Task::done(Message::FileSelected(Some(file_path))))
        } else {
            // No file argument - show welcome screen
            (app, Task::none())
        }
    }

    // Subscription: Listen for keyboard and menu events
    fn subscription(&self) -> Subscription<Message> {
        // Try to initialize menu (waits a few ticks for NSApp to be ready)
        try_initialize_menu();

        // Combine keyboard, window, and menu event subscriptions
        Subscription::batch([
            // Keyboard events subscription
            keyboard::listen().filter_map(|event| {
                match event {
                    keyboard::Event::KeyPressed { key, modifiers, .. } => {
                        Some(Message::KeyPressed(key, modifiers))
                    }
                    keyboard::Event::ModifiersChanged(modifiers) => {
                        Some(Message::ModifiersChanged(modifiers))
                    }
                    _ => None
                }
            }),
            // Window events subscription - listen for file drops
            event::listen().filter_map(|event| {
                if let Event::Window(window::Event::FileDropped(path)) = event {
                    Some(Message::FileDropped(path))
                } else {
                    None
                }
            }),
            // Menu events subscription - poll for native menu events every 50ms
            iced::time::every(std::time::Duration::from_millis(50)).map(|_| {
                // Try to receive menu event (non-blocking)
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    match event.id().as_ref() {
                        id if id == menu_ids::OPEN_FILE => Message::OpenFileDialog,
                        id if id == menu_ids::OPEN_NEW_WINDOW => Message::OpenFileInNewWindow,
                        id if id == menu_ids::COPY_VALUE => Message::CopySelectedValue,
                        id if id == menu_ids::COPY_KEY => Message::CopySelectedName,
                        id if id == menu_ids::COPY_PATH => Message::CopySelectedPath,
                        id if id == menu_ids::TOGGLE_THEME => Message::ToggleTheme,
                        id if id == menu_ids::KEYBOARD_SHORTCUTS => Message::ToggleHelp,
                        id if id == menu_ids::CHECK_UPDATES => Message::CheckForUpdates,
                        id if id == menu_ids::EXPORT_JSON => Message::ExportJson,
                        id if id == menu_ids::EXPAND_ALL => Message::ExpandAllChildren,
                        id if id == menu_ids::COLLAPSE_ALL => Message::CollapseAllChildren,
                        id if id == menu_ids::OPEN_EXTERNAL => Message::OpenInExternalEditor,
                        _ => Message::NoOp, // PredefinedMenuItems handled by OS
                    }
                } else {
                    Message::NoOp
                }
            }),
        ])
    }

    /// Flatten the tree into a Vec<FlatRow> for virtual scrolling
    /// This walks only expanded nodes, pre-computing all display data
    /// Note: This is a static method to avoid borrow checker issues
    fn flatten_visible_nodes(tree: &JsonTree) -> Vec<FlatRow> {
        let mut rows = Vec::new();

        // Start from root's children (skip root node like collect_nodes does)
        if let Some(root) = tree.get_node(tree.root_index()) {
            // Determine if root is array or object for path building
            let root_is_array = matches!(root.value, JsonValue::Array);
            let child_count = root.children.len();
            for (i, &child_index) in root.children.iter().enumerate() {
                let is_last = i == child_count - 1;
                // Build initial path segment based on root type
                let child_path = if root_is_array {
                    format!("[{}]", i)
                } else if let Some(child) = tree.get_node(child_index) {
                    child.key.as_deref().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                Self::flatten_node(tree, child_index, &mut rows, "", is_last, false, &child_path);
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

        // Build prefix - ends at branch point (├ or └), not including the dash
        // The dash or expand icon is added during rendering for proper alignment
        let (current_prefix, child_prefix) = if is_root {
            (String::new(), String::new())
        } else if node.depth == 1 {
            let connector = if is_last { "└" } else { "├" };
            let child = if is_last { "   ".to_string() } else { "│  ".to_string() };
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

        // Format value (same logic as collect_nodes)
        let (value_display, value_type) = match &node.value {
            JsonValue::Null => ("null".to_string(), ValueType::Null),
            JsonValue::Bool(b) => (b.to_string(), ValueType::Bool),
            JsonValue::Number(n) => (n.to_string(), ValueType::Number),
            JsonValue::String(s) => (format!("\"{}\"", s), ValueType::String),
            JsonValue::Array => {
                if node.expanded {
                    (":".to_string(), ValueType::Bracket)
                } else {
                    ("[...]".to_string(), ValueType::Key)
                }
            }
            JsonValue::Object => {
                if node.expanded {
                    (":".to_string(), ValueType::Bracket)
                } else {
                    ("{...}".to_string(), ValueType::Key)
                }
            }
        };

        // Get current row index before pushing
        let row_index = rows.len();

        // Create the FlatRow
        rows.push(FlatRow {
            node_index: index,
            prefix: current_prefix,
            key: node.key.as_ref().map(|k| k.to_string()),
            value_display,
            value_type,
            is_expandable: node.is_expandable(),
            is_expanded: node.expanded,
            row_index,
            path: current_path.to_string(),
        });

        // Recurse into children if expanded
        if node.expanded {
            let is_array = matches!(node.value, JsonValue::Array);
            let child_count = node.children.len();
            for (i, &child_index) in node.children.iter().enumerate() {
                let is_last_child = i == child_count - 1;
                // Build child path
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
                Self::flatten_node(tree, child_index, rows, &child_prefix, is_last_child, false, &child_path);
            }
        }
    }

    /// Render a single FlatRow into an Element
    fn render_flat_row<'a>(&self, flat_row: &FlatRow) -> Element<'a, Message> {
        // Get theme colors
        let colors = get_theme_colors(self.theme);
        let value_color = flat_row.value_type.color(&colors);

        // Build the row element
        let node_row: Element<'a, Message> = if flat_row.is_expandable {
            // Expandable node - make it clickable
            // Icon replaces the "─" part of the connector for alignment
            let indicator = if flat_row.is_expanded { "⊟ " } else { "⊞ " };

            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(colors.bracket).into(),
                text(indicator).font(Font::MONOSPACE).size(13).color(colors.indicator).into(),
            ];

            // Show key if it exists (empty keys shown as "" for visibility)
            if let Some(k) = &flat_row.key {
                let display_key = if k.is_empty() { "\"\"".to_string() } else { k.clone() };
                row_elements.push(
                    text(display_key)
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(colors.key)
                        .into()
                );
                row_elements.push(
                    text(": ")
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(colors.bracket)
                        .into()
                );
            }

            // Show value preview for collapsed containers ({...} or [...])
            if !flat_row.is_expanded {
                row_elements.push(
                    text(flat_row.value_display.clone())
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(value_color)
                        .into()
                );
            }

            button(row(row_elements).spacing(0))
                .on_press(Message::ToggleNode(flat_row.node_index))
                .padding(0)
                .style(button::text)
                .into()
        } else {
            // Leaf node - clickable to select
            // Add "─ " to complete the connector (same width as icon + space)
            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(colors.bracket).into(),
                text("─ ").font(Font::MONOSPACE).size(13).color(colors.bracket).into(),
            ];

            // Show key if it exists (empty keys shown as "" for visibility)
            if let Some(k) = &flat_row.key {
                let display_key = if k.is_empty() { "\"\"".to_string() } else { k.clone() };
                row_elements.push(
                    text(display_key)
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(colors.key)
                        .into()
                );
                row_elements.push(
                    text(": ")
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(colors.bracket)
                        .into()
                );
            }

            row_elements.push(
                text(flat_row.value_display.clone())
                    .font(Font::MONOSPACE)
                    .size(13)
                    .color(value_color)
                    .into()
            );

            // Wrap in button to make clickable for selection
            button(row(row_elements).spacing(0))
                .on_press(Message::SelectNode(flat_row.node_index))
                .padding(0)
                .style(button::text)
                .into()
        };

        // Determine background color based on selection, search state and zebra striping
        let is_selected = self.selected_node == Some(flat_row.node_index);
        let is_match = self.search_matches.contains(&flat_row.node_index);
        let is_current_result = self.search_result_index
            .map(|i| self.search_results.get(i) == Some(&flat_row.node_index))
            .unwrap_or(false);

        // Priority: current search result > selected > search match > zebra stripe
        let background_color = if is_current_result {
            Some(colors.search_current)
        } else if is_selected {
            Some(colors.selected)
        } else if is_match {
            Some(colors.search_match)
        } else if flat_row.row_index % 2 == 1 {
            Some(colors.row_odd)
        } else {
            None
        };

        // Wrap in container with appropriate background
        let row_container = match background_color {
            Some(color) => {
                container(node_row)
                    .width(Length::Fixed(5000.0))
                    .height(Length::Fixed(ROW_HEIGHT))
                    .style(move |_theme| container::Style {
                        background: Some(color.into()),
                        ..Default::default()
                    })
            }
            None => {
                container(node_row)
                    .width(Length::Fixed(5000.0))
                    .height(Length::Fixed(ROW_HEIGHT))
            }
        };

        // Wrap in mouse_area to detect right-click for context menu
        // Calculate approximate Y position based on row index and scroll
        let node_index = flat_row.node_index;
        let row_index = flat_row.row_index;
        let toolbar_height = 60.0;  // Approximate toolbar height
        let y_pos = toolbar_height + (row_index as f32 * ROW_HEIGHT) - self.scroll_offset + ROW_HEIGHT;
        // Estimate depth from prefix length (each level adds ~4 chars of indent)
        let estimated_depth = flat_row.prefix.len() / 4;
        let x_pos = 50.0 + (estimated_depth as f32 * 15.0);

        mouse_area(row_container)
            .on_right_press(Message::ShowContextMenu(node_index, x_pos, y_pos))
            .into()
    }

    // Calculate the maximum display width needed for the tree
    // Returns estimated pixel width based on character count
    fn calculate_max_width(&self) -> f32 {
        let Some(tree) = &self.tree else {
            return 400.0;  // Default width if no tree
        };

        let max_chars = self.max_line_chars(tree, tree.root_index(), 0);

        // Estimate: ~8.5 pixels per character in monospace size 14, plus padding
        let char_width = 8.5;
        let padding = 80.0;  // Window chrome + margins
        let min_width = 500.0;
        let max_width = 1400.0;  // Don't go too wide

        (max_chars as f32 * char_width + padding).clamp(min_width, max_width)
    }

    // Recursively find the maximum line length in characters
    fn max_line_chars(&self, tree: &JsonTree, index: usize, depth: usize) -> usize {
        let Some(node) = tree.get_node(index) else {
            return 0;
        };

        // Estimate characters for this line:
        // prefix (3 chars per depth) + indicator (2) + key + ": " + value
        let prefix_len = depth * 3;
        let indicator_len = 2;
        let key_len = node.key.as_ref().map(|k| k.len() + 3).unwrap_or(0);  // key + ": "
        let value_len = match &node.value {
            JsonValue::Null => 4,
            JsonValue::Bool(b) => b.to_string().len(),
            JsonValue::Number(n) => n.to_string().len(),
            JsonValue::String(s) => s.len() + 2,  // quotes
            JsonValue::Array | JsonValue::Object => 1,  // just ":"
        };

        let this_line = prefix_len + indicator_len + key_len + value_len;

        // Check children if expanded
        let max_child = if node.expanded {
            node.children.iter()
                .map(|&child_idx| self.max_line_chars(tree, child_idx, depth + 1))
                .max()
                .unwrap_or(0)
        } else {
            0
        };

        this_line.max(max_child)
    }

    // Handle messages and update state
    // Returns a Task for async operations (like file dialogs)
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFileDialog => {
                // Return a Task that opens the file dialog asynchronously
                Task::perform(
                    async {
                        // rfd::AsyncFileDialog works with async-std (which rfd uses by default)
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("JSON", &["json"])
                            .add_filter("All Files", &["*"])
                            .set_title("Open JSON File")
                            .pick_file()
                            .await;

                        // Convert FileHandle to PathBuf
                        file.map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelected,  // This message will be sent with the result
                )
            }
            Message::FileSelected(path_option) => {
                // File dialog returned - either a path or None (cancelled)
                match path_option {
                    Some(path) => {
                        // Try to load the file, measuring time
                        let start = Instant::now();
                        match fs::read_to_string(&path) {
                            Ok(contents) => {
                                match serde_json::from_str::<serde_json::Value>(&contents) {
                                    Ok(json_value) => {
                                        let tree = parser::build_tree(&json_value);
                                        let elapsed = start.elapsed();
                                        let filename = path.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "unknown".to_string());
                                        self.status = format!("✓ {} ({} nodes)", filename, tree.node_count());
                                        self.tree = Some(tree);
                                        self.current_file = Some(path);
                                        self.load_time = Some(elapsed);
                                        self.parse_error = None;  // Clear any previous error

                                        // Rebuild flat_rows for virtual scrolling
                                        self.flat_rows = Self::flatten_visible_nodes(self.tree.as_ref().unwrap());

                                        // Auto-resize window (title updates via title closure)
                                        let new_width = self.calculate_max_width();
                                        return window::latest()
                                            .and_then(move |window_id| {
                                                window::resize(window_id, Size::new(new_width, 700.0))
                                            });
                                    }
                                    Err(e) => {
                                        let filename = path.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "unknown".to_string());
                                        self.parse_error = Some(ParseError::from_serde_error(&e, &contents, &filename));
                                        self.status = format!("✗ Parse error in {}", filename);
                                        self.tree = None;
                                        self.current_file = None;
                                    }
                                }
                            }
                            Err(e) => {
                                let filename = path.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "unknown".to_string());
                                self.parse_error = Some(ParseError {
                                    message: e.to_string(),
                                    line: None,
                                    column: None,
                                    context_line: None,
                                    filename,
                                });
                                self.status = format!("✗ File error: {}", e);
                                self.tree = None;
                                self.current_file = None;
                            }
                        }
                        Task::none()  // No follow-up task needed
                    }
                    None => {
                        // User cancelled the dialog - do nothing
                        Task::none()
                    }
                }
            }
            Message::FileDropped(path) => {
                // File was dropped onto the window - check if it's a JSON file
                let is_json = path.extension()
                    .map(|ext| ext.to_string_lossy().to_lowercase())
                    .map(|ext| ext == "json")
                    .unwrap_or(false);

                if is_json {
                    // Reuse the FileSelected logic
                    self.update(Message::FileSelected(Some(path)))
                } else {
                    // Not a JSON file - show error
                    let filename = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    self.status = format!("✗ Not a JSON file: {}", filename);
                    Task::none()
                }
            }
            Message::ToggleNode(index) => {
                // Also select the node when toggling
                self.selected_node = Some(index);
                if let Some(tree) = &mut self.tree {
                    tree.toggle_expanded(index);
                    // Rebuild flat_rows after toggle
                    self.flat_rows = Self::flatten_visible_nodes(tree);
                }
                Task::none()  // No async work needed
            }
            Message::Scrolled(viewport) => {
                // Update scroll offset and viewport height for virtual scrolling
                self.scroll_offset = viewport.absolute_offset().y;
                self.viewport_height = viewport.bounds().height;
                Task::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.run_search()
            }
            Message::ToggleCaseSensitive => {
                self.search_case_sensitive = !self.search_case_sensitive;
                self.run_search()
            }
            Message::ToggleRegex => {
                self.search_use_regex = !self.search_use_regex;
                self.run_search()
            }
            Message::SearchNext => {
                if !self.search_results.is_empty() {
                    let new_index = match self.search_result_index {
                        Some(i) => (i + 1) % self.search_results.len(),
                        None => 0,
                    };
                    self.search_result_index = Some(new_index);

                    // Expand path to result
                    let node_index = self.search_results[new_index];
                    self.expand_to_node(node_index);

                    // Rebuild flat_rows BEFORE scrolling
                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }

                    // Return scroll task
                    self.scroll_to_node(node_index)
                } else {
                    Task::none()
                }
            }
            Message::SearchPrev => {
                if !self.search_results.is_empty() {
                    let new_index = match self.search_result_index {
                        Some(i) => {
                            if i == 0 {
                                self.search_results.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => self.search_results.len() - 1,
                    };
                    self.search_result_index = Some(new_index);

                    // Expand path to result
                    let node_index = self.search_results[new_index];
                    self.expand_to_node(node_index);

                    // Rebuild flat_rows BEFORE scrolling
                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }

                    // Return scroll task
                    self.scroll_to_node(node_index)
                } else {
                    Task::none()
                }
            }
            Message::ClearSearch => {
                // Clear search state
                self.search_query.clear();
                self.search_results.clear();
                self.search_result_index = None;
                self.search_matches.clear();
                self.search_regex_error = None;
                Task::none()
            }
            Message::FocusSearch => {
                // Focus the search input using a widget operation
                // operate() takes an Operation and returns a Task that executes it
                operate(focusable::focus(self.search_input_id.clone()))
            }
            Message::ModifiersChanged(modifiers) => {
                // Track modifier state so we can check Shift when search submits
                self.current_modifiers = modifiers;
                Task::none()
            }
            Message::SearchSubmit => {
                // Called when Enter is pressed in search input
                // Check if Shift is held to determine direction
                if self.current_modifiers.shift() {
                    self.update(Message::SearchPrev)
                } else {
                    self.update(Message::SearchNext)
                }
            }
            Message::KeyPressed(key, modifiers) => {
                // Handle keyboard shortcuts
                // Key Rust concept: Pattern matching on enums with associated data
                // We match on the key type and check modifiers

                // Check for Cmd on macOS, Ctrl on other platforms
                let cmd_or_ctrl = modifiers.command() || modifiers.control();

                match key {
                    // Escape: Close overlays or clear search
                    Key::Named(Named::Escape) => {
                        if self.show_help {
                            self.update(Message::ToggleHelp)
                        } else if self.context_menu_state.is_some() {
                            self.update(Message::HideContextMenu)
                        } else {
                            self.update(Message::ClearSearch)
                        }
                    }
                    // Enter: Navigate search results
                    Key::Named(Named::Enter) => {
                        if modifiers.shift() {
                            self.update(Message::SearchPrev)
                        } else {
                            self.update(Message::SearchNext)
                        }
                    }
                    // Cmd/Ctrl+O: Open file
                    Key::Character(c) if c.as_str() == "o" && cmd_or_ctrl => {
                        self.update(Message::OpenFileDialog)
                    }
                    // Cmd/Ctrl+G: Navigate search (alternative)
                    Key::Character(c) if c.as_str() == "g" && cmd_or_ctrl => {
                        if modifiers.shift() {
                            self.update(Message::SearchPrev)
                        } else {
                            self.update(Message::SearchNext)
                        }
                    }
                    // Cmd/Ctrl+F: Focus search input
                    Key::Character(c) if c.as_str() == "f" && cmd_or_ctrl => {
                        self.update(Message::FocusSearch)
                    }
                    // Cmd/Ctrl+N: New window (opens file dialog, then opens in new window)
                    Key::Character(c) if c.as_str() == "n" && cmd_or_ctrl => {
                        self.update(Message::OpenFileInNewWindow)
                    }
                    // Cmd/Ctrl+C: Copy selected node value to clipboard
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && !modifiers.shift() && !modifiers.alt() => {
                        self.update(Message::CopySelectedValue)
                    }
                    // Cmd/Ctrl+Shift+C: Copy selected node key/name to clipboard
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && modifiers.shift() && !modifiers.alt() => {
                        self.update(Message::CopySelectedName)
                    }
                    // Cmd/Ctrl+Option+C: Copy selected node path to clipboard
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && modifiers.alt() => {
                        self.update(Message::CopySelectedPath)
                    }
                    // Cmd/Ctrl+T: Toggle theme (dark/light)
                    Key::Character(c) if c.as_str() == "t" && cmd_or_ctrl => {
                        self.update(Message::ToggleTheme)
                    }
                    // Cmd/Ctrl+? or Cmd/Ctrl+/: Toggle help overlay
                    Key::Character(c) if (c.as_str() == "/" || c.as_str() == "?") && cmd_or_ctrl => {
                        self.update(Message::ToggleHelp)
                    }
                    _ => Task::none()
                }
            }
            Message::OpenFileInNewWindow => {
                // Open file dialog, then spawn new window with selected file
                Task::perform(
                    async {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("JSON", &["json"])
                            .add_filter("All Files", &["*"])
                            .set_title("Open JSON File in New Window")
                            .pick_file()
                            .await;
                        file.map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelectedForNewWindow,
                )
            }
            Message::FileSelectedForNewWindow(path_option) => {
                // File was selected - spawn new window with it
                if let Some(file_path) = path_option {
                    Self::spawn_new_window(Some(file_path));
                }
                Task::none()
            }
            Message::SelectNode(node_index) => {
                // Select the node (toggle if already selected)
                if self.selected_node == Some(node_index) {
                    self.selected_node = None;
                } else {
                    self.selected_node = Some(node_index);
                }
                Task::none()
            }
            Message::CopySelectedValue => {
                // Copy the selected node's value to clipboard
                self.context_menu_state = None;  // Hide context menu
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    if tree.get_node(node_index).is_some() {
                        // Format the value for clipboard
                        let value_string = Self::format_node_value_for_copy(tree, node_index);
                        // Use Iced's clipboard API to write
                        return clipboard::write(value_string);
                    }
                }
                Task::none()
            }
            Message::CopySelectedPath => {
                // Copy the selected node's path to clipboard
                self.context_menu_state = None;  // Hide context menu
                if let Some(node_index) = self.selected_node {
                    // Find the path from flat_rows
                    if let Some(flat_row) = self.flat_rows.iter().find(|r| r.node_index == node_index) {
                        return clipboard::write(flat_row.path.clone());
                    }
                }
                Task::none()
            }
            Message::CopySelectedName => {
                // Copy the selected node's key/name to clipboard
                self.context_menu_state = None;  // Hide context menu
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    if let Some(node) = tree.get_node(node_index) {
                        if let Some(key) = &node.key {
                            return clipboard::write(key.clone());
                        }
                    }
                }
                Task::none()
            }
            Message::ToggleTheme => {
                // Toggle between dark and light theme
                self.theme = match self.theme {
                    AppTheme::Dark => AppTheme::Light,
                    AppTheme::Light => AppTheme::Dark,
                };
                Task::none()
            }
            Message::ToggleHelp => {
                // Toggle help overlay visibility
                self.show_help = !self.show_help;
                Task::none()
            }
            Message::NoOp => {
                // No operation - used for menu polling when no event
                Task::none()
            }
            Message::CheckForUpdates => {
                // Show checking state immediately
                self.update_check_state = UpdateCheckState::Checking;

                // Perform async request to GitHub API
                Task::perform(
                    async {
                        Self::fetch_latest_release().await
                    },
                    Message::UpdateCheckResult,
                )
            }
            Message::UpdateCheckResult(state) => {
                self.update_check_state = state;
                Task::none()
            }
            Message::DismissUpdateDialog => {
                self.update_check_state = UpdateCheckState::None;
                Task::none()
            }
            Message::OpenReleaseUrl(url) => {
                // Open URL in default browser
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg(&url)
                        .spawn();
                }
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(["/c", "start", &url])
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&url)
                        .spawn();
                }
                self.update_check_state = UpdateCheckState::None;
                Task::none()
            }
            Message::ExportJson => {
                // Export selected node to JSON file
                self.context_menu_state = None;  // Hide context menu
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    let json_string = Self::node_to_json_string(tree, node_index);
                    Task::perform(
                        async move {
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("export.json")
                                .save_file()
                                .await;
                            if let Some(handle) = file_handle {
                                let _ = std::fs::write(handle.path(), json_string);
                            }
                        },
                        |_| Message::NoOp
                    )
                } else {
                    Task::none()
                }
            }
            Message::ExpandAllChildren => {
                // Expand all children of selected node recursively
                self.context_menu_state = None;  // Hide context menu
                if let Some(node_index) = self.selected_node {
                    if let Some(tree) = &mut self.tree {
                        Self::set_expanded_recursive(tree, node_index, true);
                        // Rebuild flat_rows after expanding
                    }
                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }
                }
                Task::none()
            }
            Message::CollapseAllChildren => {
                // Collapse all children of selected node recursively
                self.context_menu_state = None;  // Hide context menu
                if let Some(node_index) = self.selected_node {
                    if let Some(tree) = &mut self.tree {
                        Self::set_expanded_recursive(tree, node_index, false);
                        // Rebuild flat_rows after collapsing
                    }
                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }
                }
                Task::none()
            }
            Message::OpenInExternalEditor => {
                // Open current file in system's default editor
                if let Some(path) = &self.current_file {
                    #[cfg(target_os = "macos")]
                    {
                        let _ = Command::new("open")
                            .arg("-t")  // Open in default text editor
                            .arg(path)
                            .spawn();
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let _ = Command::new("cmd")
                            .args(["/C", "start", "", path.to_str().unwrap_or("")])
                            .spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = Command::new("xdg-open")
                            .arg(path)
                            .spawn();
                    }
                }
                Task::none()
            }
            Message::ShowContextMenu(node_index, x, y) => {
                // Select the node and show context menu at position
                self.selected_node = Some(node_index);
                self.context_menu_state = Some((node_index, x, y));
                Task::none()
            }
            Message::HideContextMenu => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                Task::none()
            }
            Message::OpenSubmenu(submenu) => {
                self.context_submenu = submenu;
                Task::none()
            }
            Message::CopyValueMinified => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    // Use direct minified output - no spaces after colons or commas
                    let minified = Self::node_to_json_string_minified(tree, node_index);
                    return clipboard::write(minified);
                }
                Task::none()
            }
            Message::CopyValueFormatted => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    let json = Self::node_to_json_string(tree, node_index);
                    // Parse and re-serialize with pretty printing (indentation + newlines)
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                        if let Ok(formatted) = serde_json::to_string_pretty(&value) {
                            return clipboard::write(formatted);
                        }
                    }
                    // Fallback to regular output if parsing fails
                    return clipboard::write(json);
                }
                Task::none()
            }
            Message::ExportAsJson => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    let json_string = Self::node_to_json_string(tree, node_index);
                    Task::perform(
                        async move {
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("export.json")
                                .save_file()
                                .await;
                            if let Some(handle) = file_handle {
                                let _ = std::fs::write(handle.path(), json_string);
                            }
                        },
                        |_| Message::NoOp
                    )
                } else {
                    Task::none()
                }
            }
            Message::ExportAsMinifiedJson => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    // Use direct minified output
                    let minified = Self::node_to_json_string_minified(tree, node_index);
                    Task::perform(
                        async move {
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("export.min.json")
                                .save_file()
                                .await;
                            if let Some(handle) = file_handle {
                                let _ = std::fs::write(handle.path(), minified);
                            }
                        },
                        |_| Message::NoOp
                    )
                } else {
                    Task::none()
                }
            }
            Message::ExportAsFormattedJson => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    let json = Self::node_to_json_string(tree, node_index);
                    let formatted = if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                        serde_json::to_string_pretty(&value).unwrap_or(json)
                    } else {
                        json
                    };
                    Task::perform(
                        async move {
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("export.json")
                                .save_file()
                                .await;
                            if let Some(handle) = file_handle {
                                let _ = std::fs::write(handle.path(), formatted);
                            }
                        },
                        |_| Message::NoOp
                    )
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Format a node's value for copying to clipboard
    /// For primitives: just the value
    /// For objects/arrays: JSON representation
    fn format_node_value_for_copy(tree: &JsonTree, node_index: usize) -> String {
        if let Some(node) = tree.get_node(node_index) {
            match &node.value {
                JsonValue::Null => "null".to_string(),
                JsonValue::Bool(b) => b.to_string(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::String(s) => s.clone(),
                JsonValue::Array | JsonValue::Object => {
                    // For containers, rebuild the JSON structure
                    Self::node_to_json_string(tree, node_index)
                }
            }
        } else {
            String::new()
        }
    }

    /// Convert a node and its children to a JSON string
    fn node_to_json_string(tree: &JsonTree, node_index: usize) -> String {
        Self::node_to_json_string_internal(tree, node_index, false)
    }

    fn node_to_json_string_minified(tree: &JsonTree, node_index: usize) -> String {
        Self::node_to_json_string_internal(tree, node_index, true)
    }

    fn node_to_json_string_internal(tree: &JsonTree, node_index: usize, minified: bool) -> String {
        let (sep, kv_sep) = if minified { (",", ":") } else { (", ", ": ") };

        if let Some(node) = tree.get_node(node_index) {
            match &node.value {
                JsonValue::Null => "null".to_string(),
                JsonValue::Bool(b) => b.to_string(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")),
                JsonValue::Array => {
                    let items: Vec<String> = node.children.iter()
                        .map(|&child_idx| Self::node_to_json_string_internal(tree, child_idx, minified))
                        .collect();
                    format!("[{}]", items.join(sep))
                }
                JsonValue::Object => {
                    let items: Vec<String> = node.children.iter()
                        .filter_map(|&child_idx| {
                            tree.get_node(child_idx).map(|child| {
                                let key = child.key.as_deref().unwrap_or("");
                                let value = Self::node_to_json_string_internal(tree, child_idx, minified);
                                format!("\"{}\"{}{}", key, kv_sep, value)
                            })
                        })
                        .collect();
                    format!("{{{}}}", items.join(sep))
                }
            }
        } else {
            String::new()
        }
    }

    /// Spawn a new instance of the application, optionally with a file
    fn spawn_new_window(file_path: Option<PathBuf>) {
        if let Ok(exe_path) = env::current_exe() {
            let mut cmd = Command::new(exe_path);
            if let Some(path) = file_path {
                cmd.arg(path);
            }
            let _ = cmd.spawn();
        }
    }

    /// Fetch the latest release from GitHub API and compare with current version
    async fn fetch_latest_release() -> UpdateCheckState {
        const GITHUB_API_URL: &str = "https://api.github.com/repos/maumercado/unfold/releases/latest";
        const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

        // Build HTTP client with User-Agent (required by GitHub API)
        let client = match reqwest::Client::builder()
            .user_agent(format!("Unfold/{}", CURRENT_VERSION))
            .build()
        {
            Ok(c) => c,
            Err(e) => return UpdateCheckState::Error(format!("Failed to create HTTP client: {}", e)),
        };

        // Fetch latest release
        let response = match client.get(GITHUB_API_URL).send().await {
            Ok(r) => r,
            Err(e) => return UpdateCheckState::Error(format!("Network error: {}", e)),
        };

        // Check for HTTP errors
        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return UpdateCheckState::Error("No releases found on GitHub".to_string());
            }
            return UpdateCheckState::Error(format!("GitHub API error: {}", response.status()));
        }

        // Parse JSON response
        let release: GitHubRelease = match response.json().await {
            Ok(r) => r,
            Err(e) => return UpdateCheckState::Error(format!("Failed to parse response: {}", e)),
        };

        // Parse versions (remove leading 'v' if present)
        let latest_version_str = release.tag_name.trim_start_matches('v');
        let current_version = match Version::parse(CURRENT_VERSION) {
            Ok(v) => v,
            Err(e) => return UpdateCheckState::Error(format!("Invalid current version: {}", e)),
        };
        let latest_version = match Version::parse(latest_version_str) {
            Ok(v) => v,
            Err(e) => return UpdateCheckState::Error(format!("Invalid release version '{}': {}", latest_version_str, e)),
        };

        // Compare versions
        if latest_version > current_version {
            UpdateCheckState::UpdateAvailable {
                version: release.tag_name,
                release_url: release.html_url,
            }
        } else {
            UpdateCheckState::UpToDate
        }
    }

    /// Search all nodes in the tree for matches
    /// Returns (results, error_message) where error_message is Some if regex is invalid
    fn search_nodes(
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
                if let Some(key) = &node.key {
                    if matches(key) {
                        results.push(i);
                        continue;
                    }
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

    /// Run search with current query and options, update results
    fn run_search(&mut self) -> Task<Message> {
        if self.search_query.is_empty() {
            self.search_results.clear();
            self.search_result_index = None;
            self.search_matches.clear();
            self.search_regex_error = None;
            return Task::none();
        }

        if let Some(tree) = &self.tree {
            let (results, error) = Self::search_nodes(
                tree,
                &self.search_query,
                self.search_case_sensitive,
                self.search_use_regex,
            );

            self.search_regex_error = error;
            self.search_results = results;
            self.search_matches = self.search_results.iter().cloned().collect();

            if !self.search_results.is_empty() {
                self.search_result_index = Some(0);
                let target = self.search_results[0];
                self.expand_to_node(target);
                self.flat_rows = Self::flatten_visible_nodes(self.tree.as_ref().unwrap());
                return self.scroll_to_node(target);
            } else {
                self.search_result_index = None;
                self.flat_rows = Self::flatten_visible_nodes(tree);
            }
        }

        Task::none()
    }

    /// Expand all ancestors of a node to make it visible
    fn expand_to_node(&mut self, target_index: usize) {
        if let Some(tree) = &mut self.tree {
            // Get the path from root to target
            let path = tree.get_path_to_node(target_index);

            // Expand all nodes along the path (except the target itself)
            for &node_index in &path {
                if node_index != target_index {
                    tree.set_expanded(node_index, true);
                }
            }
        }
    }

    /// Recursively set expanded state for a node and all its children
    fn set_expanded_recursive(tree: &mut JsonTree, node_index: usize, expanded: bool) {
        // Set expanded state for this node
        tree.set_expanded(node_index, expanded);

        // Get children and recurse
        if let Some(node) = tree.get_node(node_index) {
            let children = node.children.clone();
            for child_index in children {
                Self::set_expanded_recursive(tree, child_index, expanded);
            }
        }
    }

    /// Calculate the scroll offset to make a node visible and return a scroll Task
    fn scroll_to_node(&self, target_index: usize) -> Task<Message> {
        // Find the row index of this node in flat_rows
        if let Some(row_pos) = self.flat_rows.iter().position(|r| r.node_index == target_index) {
            // Calculate scroll offset to center the result
            let target_offset = row_pos as f32 * ROW_HEIGHT;
            let center_offset = self.viewport_height / 2.0;
            let scroll_y = (target_offset - center_offset).max(0.0);

            // Return a task to scroll to this position using the widget operation
            let id = self.tree_scrollable_id.clone();
            let offset = AbsoluteOffset { x: Some(0.0), y: Some(scroll_y) };
            operate(scroll_to(id, offset))
        } else {
            Task::none()
        }
    }

    // Render the UI
    fn view(&self) -> Element<'_, Message> {
        // Get theme colors
        let colors = get_theme_colors(self.theme);

        // Tree display section
        let tree_view: Element<'_, Message> = match &self.tree {
            Some(_tree) => {
                // ===== VIRTUAL SCROLLING =====
                // Calculate which rows are visible based on scroll position
                let total_rows = self.flat_rows.len();

                // Calculate visible range (with buffer for smooth scrolling)
                let first_visible = (self.scroll_offset / ROW_HEIGHT).floor() as usize;
                let visible_count = (self.viewport_height / ROW_HEIGHT).ceil() as usize + 1;

                // Add buffer rows above and below for smoother scrolling
                let start = first_visible.saturating_sub(BUFFER_ROWS);
                let end = (first_visible + visible_count + BUFFER_ROWS).min(total_rows);

                // Build only the visible rows
                let mut elements: Vec<Element<'_, Message>> = Vec::new();

                // Add top spacer to position content correctly
                let top_offset = start as f32 * ROW_HEIGHT;
                if top_offset > 0.0 {
                    elements.push(Space::new().height(Length::Fixed(top_offset)).into());
                }

                // Render only the visible rows
                for flat_row in self.flat_rows.iter().skip(start).take(end - start) {
                    elements.push(self.render_flat_row(flat_row));
                }

                // Add bottom spacer to maintain scroll bar size
                let bottom_offset = (total_rows - end) as f32 * ROW_HEIGHT;
                if bottom_offset > 0.0 {
                    elements.push(Space::new().height(Length::Fixed(bottom_offset)).into());
                }

                let nodes_column = column(elements)
                    .spacing(0);

                scrollable(
                    container(nodes_column)
                        .padding([10, 0])
                )
                .id(self.tree_scrollable_id.clone())
                .direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::default(),
                    horizontal: scrollable::Scrollbar::default(),
                })
                .on_scroll(Message::Scrolled)  // Track scroll position
                .height(Length::Fill)
                .width(Fill)
                .into()
            }
            None => {
                // Show error screen or welcome screen
                if let Some(ref error) = self.parse_error {
                    // Error screen with detailed information
                    let error_icon = text("⚠").size(48).color(colors.error);

                    let error_title = text(format!("Failed to parse {}", error.filename))
                        .size(18)
                        .color(colors.error);

                    let error_message = text(&error.message)
                        .size(14)
                        .color(colors.text_primary);

                    // Location info (line:column)
                    let location_text = match (error.line, error.column) {
                        (Some(line), Some(col)) => format!("Line {}, Column {}", line, col),
                        (Some(line), None) => format!("Line {}", line),
                        _ => String::new(),
                    };
                    let location = text(location_text)
                        .size(13)
                        .color(colors.text_secondary);

                    // Context line with caret pointing to error
                    let context_section: Element<'_, Message> = if let Some(ref ctx_line) = error.context_line {
                        let truncated = if ctx_line.len() > 80 {
                            format!("{}...", &ctx_line[..80])
                        } else {
                            ctx_line.clone()
                        };

                        // Build caret indicator pointing to the column
                        let caret = if let Some(col) = error.column {
                            let spaces = " ".repeat(col.saturating_sub(1));
                            format!("{}^", spaces)
                        } else {
                            String::new()
                        };

                        column![
                            text(truncated)
                                .size(12)
                                .font(Font::MONOSPACE)
                                .color(colors.error_context),
                            text(caret)
                                .size(12)
                                .font(Font::MONOSPACE)
                                .color(colors.error),
                        ]
                        .spacing(0)
                        .into()
                    } else {
                        Space::new().into()
                    };

                    let try_again_button = button(text("Try Another File...").size(14))
                        .on_press(Message::OpenFileDialog)
                        .padding([8, 16])
                        .style(button_3d_style_themed(colors));

                    let error_content = column![
                        error_icon,
                        error_title,
                        Space::new().height(Length::Fixed(10.0)),
                        error_message,
                        location,
                        Space::new().height(Length::Fixed(15.0)),
                        context_section,
                        Space::new().height(Length::Fixed(20.0)),
                        try_again_button,
                    ]
                    .spacing(5)
                    .align_x(Center);

                    container(error_content)
                        .width(Fill)
                        .height(Fill)
                        .center(Fill)
                        .style(move |_theme| container::Style {
                            background: Some(colors.background.into()),
                            ..Default::default()
                        })
                        .into()
                } else {
                    // Normal welcome screen (Dadroit-style)
                    let welcome_text = text("Welcome to Unfold JSON Viewer.")
                        .size(15)
                        .color(colors.text_secondary);

                    // "Open" as a clickable underlined link
                    let open_link = button(
                        text("Open").size(15).style(|_theme| text::Style {
                            color: Some(Color::from_rgb(0.3, 0.5, 0.8)),
                        })
                    )
                    .on_press(Message::OpenFileDialog)
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        text_color: Color::from_rgb(0.3, 0.5, 0.8),
                        ..Default::default()
                    });

                    let action_row = row![
                        open_link,
                        text(" or drag and drop .json file here.").size(15).color(colors.text_secondary),
                    ];

                    // "Open in new window" link
                    let new_window_link = button(
                        text("Open in new window").size(13)
                    )
                    .on_press(Message::OpenFileInNewWindow)
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        text_color: Color::from_rgb(0.4, 0.55, 0.75),
                        ..Default::default()
                    });

                    // Theme toggle as subtle text link
                    let theme_label = match self.theme {
                        AppTheme::Dark => "Switch to Light Mode",
                        AppTheme::Light => "Switch to Dark Mode",
                    };
                    let theme_link = button(
                        text(theme_label).size(12)
                    )
                    .on_press(Message::ToggleTheme)
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        text_color: Color::from_rgb(0.5, 0.5, 0.5),
                        ..Default::default()
                    });

                    // Keyboard shortcuts section
                    let shortcuts_title = text("Keyboard Shortcuts")
                        .size(13)
                        .color(colors.text_secondary);

                    let shortcut_style = |colors: ThemeColors| move |_theme: &Theme| text::Style {
                        color: Some(colors.text_secondary),
                    };

                    let cmd_key = if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" };

                    let opt_key = if cfg!(target_os = "macos") { "Option" } else { "Alt" };
                    let shortcuts_list = column![
                        text(format!("{}+O  Open file", cmd_key)).size(11).style(shortcut_style(colors)),
                        text(format!("{}+C  Copy value", cmd_key)).size(11).style(shortcut_style(colors)),
                        text(format!("{}+Shift+C  Copy key", cmd_key)).size(11).style(shortcut_style(colors)),
                        text(format!("{}+{}+C  Copy path", cmd_key, opt_key)).size(11).style(shortcut_style(colors)),
                        text(format!("{}+/  All shortcuts", cmd_key)).size(11).style(shortcut_style(colors)),
                    ]
                    .spacing(4)
                    .align_x(Center);

                    let welcome = column![
                        welcome_text,
                        action_row,
                        new_window_link,
                        Space::new().height(Length::Fixed(20.0)),
                        theme_link,
                        Space::new().height(Length::Fixed(30.0)),
                        shortcuts_title,
                        Space::new().height(Length::Fixed(8.0)),
                        shortcuts_list,
                    ]
                    .spacing(8)
                    .align_x(Center);

                    container(welcome)
                        .width(Fill)
                        .height(Fill)
                        .center(Fill)
                        .style(move |_theme| container::Style {
                            background: Some(colors.background.into()),
                            ..Default::default()
                        })
                        .into()
                }
            }
        };

        // When file is loaded, show toolbar + tree + status bar
        if self.tree.is_some() {
            // Search option toggle buttons (Dadroit style)
            let case_button = button(text("Aa").size(11))
                .padding([4, 8])
                .style(button_toggle_style_themed(self.search_case_sensitive, colors))
                .on_press(Message::ToggleCaseSensitive);

            let regex_button = button(text(".*").size(11))
                .padding([4, 8])
                .style(button_toggle_style_themed(self.search_use_regex, colors))
                .on_press(Message::ToggleRegex);

            // Search input with ID for programmatic focus
            // on_submit is triggered when Enter is pressed while focused
            // SearchSubmit checks current_modifiers to handle Shift+Enter
            let search_input = text_input("Find...", &self.search_query)
                .id(self.search_input_id.clone())
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::SearchSubmit)
                .padding(5)
                .width(Length::Fixed(200.0));

            // Search result text (show error if regex is invalid)
            let search_result_text = if let Some(ref error) = self.search_regex_error {
                error.clone()
            } else if self.search_results.is_empty() {
                if self.search_query.is_empty() {
                    String::new()
                } else {
                    "No matches".to_string()
                }
            } else {
                let current = self.search_result_index.map(|i| i + 1).unwrap_or(0);
                format!("{} / {}", current, self.search_results.len())
            };

            // Only enable buttons if there are results
            let has_results = !self.search_results.is_empty();

            let prev_button = button(
                text("◂ Prev").size(11)
            )
            .padding([5, 12])
            .style(button_3d_style_themed(colors));

            let next_button = button(
                text("Next ▸").size(11)
            )
            .padding([5, 12])
            .style(button_3d_style_themed(colors));

            // Only add on_press if there are results
            let prev_button = if has_results {
                prev_button.on_press(Message::SearchPrev)
            } else {
                prev_button
            };

            let next_button = if has_results {
                next_button.on_press(Message::SearchNext)
            } else {
                next_button
            };

            // Theme toggle button (half-moon icons for better visual balance)
            let theme_icon = match self.theme {
                AppTheme::Dark => "◑",   // Half-dark icon for switching to light
                AppTheme::Light => "◐",  // Half-light icon for switching to dark
            };
            let theme_button = button(text(theme_icon).size(16))
                .padding([4, 10])
                .style(button_3d_style_themed(colors))
                .on_press(Message::ToggleTheme);

            let toolbar = container(
                row![
                    case_button,
                    Space::new().width(Length::Fixed(3.0)),
                    regex_button,
                    Space::new().width(Length::Fixed(8.0)),
                    search_input,
                    Space::new().width(Length::Fixed(10.0)),
                    prev_button,
                    Space::new().width(Length::Fixed(5.0)),
                    next_button,
                    Space::new().width(Length::Fixed(10.0)),
                    text(search_result_text).size(11).color(colors.text_secondary),
                    Space::new().width(Length::Fill),
                    theme_button,
                ]
                .align_y(Center)
            )
            .width(Fill)
            .padding([8, 10])
            .style(move |_theme| container::Style {
                background: Some(colors.toolbar_bg.into()),
                ..Default::default()
            });

            // Status bar
            let load_time_str: String = self.load_time
                .map(|d| format!("Load: {}ms", d.as_millis()))
                .unwrap_or_default();

            let node_count: String = self.tree.as_ref()
                .map(|t| format!("Nodes: {}", t.node_count()))
                .unwrap_or_default();

            // Get selected node's path and type info
            let path_display: String = if let Some(node_index) = self.selected_node {
                if let Some(flat_row) = self.flat_rows.iter().find(|r| r.node_index == node_index) {
                    // Get type info from the tree
                    let type_info = if let Some(tree) = &self.tree {
                        if let Some(node) = tree.get_node(node_index) {
                            match &node.value {
                                JsonValue::Null => "(null)".to_string(),
                                JsonValue::Bool(_) => "(bool)".to_string(),
                                JsonValue::Number(_) => "(number)".to_string(),
                                JsonValue::String(_) => "(string)".to_string(),
                                JsonValue::Array => format!("(array, {} items)", node.children.len()),
                                JsonValue::Object => format!("(object, {} keys)", node.children.len()),
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    format!("{} {}", flat_row.path, type_info)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let status_bar = container(
                row![
                    text(node_count).size(12).color(colors.text_secondary),
                    text("  |  ").size(12).color(colors.text_secondary),
                    text(path_display).size(12).color(colors.key),
                    Space::new().width(Length::Fill),
                    text(load_time_str).size(12).color(colors.text_secondary),
                ]
            )
            .width(Fill)
            .padding([5, 10])
            .style(move |_theme| container::Style {
                background: Some(colors.status_bar_bg.into()),
                ..Default::default()
            });

            // Wrap tree_view in a container with background
            let tree_container = container(tree_view)
                .width(Fill)
                .height(Fill)
                .style(move |_theme| container::Style {
                    background: Some(colors.background.into()),
                    ..Default::default()
                });

            let main_content: Element<'_, Message> = column![toolbar, tree_container, status_bar].into();

            // Show overlays: update dialog, help, or context menu
            if self.update_check_state != UpdateCheckState::None {
                stack![
                    main_content,
                    self.render_update_dialog(colors),
                ].into()
            } else if self.show_help {
                stack![
                    main_content,
                    self.render_help_overlay(colors),
                ].into()
            } else if self.context_menu_state.is_some() {
                stack![
                    main_content,
                    self.render_context_menu(colors),
                ].into()
            } else {
                main_content
            }
        } else {
            // Welcome screen - also support overlays
            if self.update_check_state != UpdateCheckState::None {
                stack![
                    tree_view,
                    self.render_update_dialog(colors),
                ].into()
            } else if self.show_help {
                stack![
                    tree_view,
                    self.render_help_overlay(colors),
                ].into()
            } else {
                tree_view
            }
        }
    }

    /// Render the help overlay with keyboard shortcuts
    fn render_help_overlay<'a>(&self, colors: ThemeColors) -> Element<'a, Message> {
        let cmd_key = if cfg!(target_os = "macos") { "⌘" } else { "Ctrl+" };
        let shift = if cfg!(target_os = "macos") { "⇧" } else { "Shift+" };
        let opt = if cfg!(target_os = "macos") { "⌥" } else { "Alt+" };

        // Helper to create a shortcut row
        fn shortcut_row<'a>(keys: String, desc: &'static str, colors: ThemeColors) -> Element<'a, Message> {
            row![
                container(text(keys).size(12).font(Font::MONOSPACE).color(colors.text_primary))
                    .width(Length::Fixed(120.0)),
                text(desc).size(12).color(colors.text_secondary),
            ]
            .spacing(10)
            .into()
        }

        let shortcuts = column![
            text("Keyboard Shortcuts").size(16).color(colors.text_primary),
            Space::new().height(Length::Fixed(15.0)),

            text("File").size(13).color(colors.key),
            shortcut_row(format!("{}O", cmd_key), "Open file", colors),
            shortcut_row(format!("{}N", cmd_key), "Open in new window", colors),
            Space::new().height(Length::Fixed(10.0)),

            text("Edit").size(13).color(colors.key),
            shortcut_row(format!("{}C", cmd_key), "Copy selected value", colors),
            shortcut_row(format!("{}{}C", shift, cmd_key), "Copy key name", colors),
            shortcut_row(format!("{}{}C", opt, cmd_key), "Copy node path", colors),
            Space::new().height(Length::Fixed(10.0)),

            text("Search").size(13).color(colors.key),
            shortcut_row(format!("{}F", cmd_key), "Focus search", colors),
            shortcut_row("Enter".to_string(), "Next result", colors),
            shortcut_row(format!("{}Enter", shift), "Previous result", colors),
            shortcut_row("Escape".to_string(), "Clear search", colors),
            Space::new().height(Length::Fixed(10.0)),

            text("View").size(13).color(colors.key),
            shortcut_row(format!("{}T", cmd_key), "Toggle theme", colors),
            shortcut_row(format!("{}/", cmd_key), "Toggle this help", colors),
            Space::new().height(Length::Fixed(20.0)),

            text("Press Escape or ⌘/ to close").size(11).color(colors.text_secondary),
        ]
        .spacing(4)
        .padding(25);

        let overlay_box = container(shortcuts)
            .style(move |_theme| container::Style {
                background: Some(colors.toolbar_bg.into()),
                border: Border {
                    color: colors.btn_border_top,
                    width: 1.0,
                    radius: Radius::from(8.0),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            });

        // Semi-transparent backdrop that closes on click
        let backdrop = button(Space::new().width(Fill).height(Fill))
            .on_press(Message::ToggleHelp)
            .style(|_theme, _status| button::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                ..Default::default()
            })
            .width(Fill)
            .height(Fill);

        stack![
            backdrop,
            container(overlay_box)
                .width(Fill)
                .height(Fill)
                .center(Fill),
        ].into()
    }

    /// Render the context menu overlay for right-click actions
    fn render_context_menu<'a>(&self, colors: ThemeColors) -> Element<'a, Message> {
        let (node_index, menu_x, menu_y) = self.context_menu_state.unwrap_or((0, 100.0, 100.0));
        let current_submenu = self.context_submenu;

        let has_children = if let Some(tree) = &self.tree {
            tree.get_node(node_index)
                .map(|n| !n.children.is_empty())
                .unwrap_or(false)
        } else {
            false
        };

        let menu_width = 180.0;
        let submenu_width = 150.0;

        // Regular menu item - closes any open submenu on hover
        let menu_item = |label: &'static str, msg: Message| -> Element<'a, Message> {
            let item_button = button(
                text(label).size(13).color(colors.text_primary)
            )
            .on_press(msg)
            .padding([6, 12])
            .width(Length::Fixed(menu_width - 8.0))
            .style(move |_theme, status| {
                let bg = match status {
                    ButtonStatus::Hovered => Some(colors.selected.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: colors.text_primary,
                    border: Border {
                        radius: Radius::from(4.0),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

            // Close submenu when hovering over regular items
            mouse_area(item_button)
                .on_enter(Message::OpenSubmenu(ContextSubmenu::None))
                .into()
        };

        // Submenu parent item with arrow - opens on hover
        let submenu_parent = |label: &'static str, submenu: ContextSubmenu, is_open: bool| -> Element<'a, Message> {
            let bg_color = if is_open { colors.selected } else { Color::TRANSPARENT };
            let item_content = container(
                row![
                    text(label).size(13).color(colors.text_primary),
                    Space::new().width(Length::Fill),
                    text("›").size(14).color(colors.text_secondary),
                ]
            )
            .padding([6, 12])
            .width(Length::Fixed(menu_width - 8.0))
            .style(move |_theme| container::Style {
                background: Some(bg_color.into()),
                border: Border {
                    radius: Radius::from(4.0),
                    ..Default::default()
                },
                ..Default::default()
            });

            // Use mouse_area to detect hover and open submenu
            mouse_area(item_content)
                .on_enter(Message::OpenSubmenu(submenu))
                .into()
        };

        // Submenu item
        let submenu_item = |label: &'static str, msg: Message| -> Element<'a, Message> {
            button(
                text(label).size(13).color(colors.text_primary)
            )
            .on_press(msg)
            .padding([6, 12])
            .width(Length::Fixed(submenu_width - 8.0))
            .style(move |_theme, status| {
                let bg = match status {
                    ButtonStatus::Hovered => Some(colors.selected.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: colors.text_primary,
                    border: Border {
                        radius: Radius::from(4.0),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .into()
        };

        let separator = || -> Element<'a, Message> {
            container(Space::new().width(Length::Fixed(menu_width - 16.0)).height(Length::Fixed(1.0)))
                .padding([4, 4])
                .style(move |_theme| container::Style {
                    background: Some(colors.btn_border_bottom.into()),
                    ..Default::default()
                })
                .into()
        };

        let mut menu_items: Vec<Element<'a, Message>> = vec![
            menu_item("Copy Key", Message::CopySelectedName),
            menu_item("Copy Value", Message::CopySelectedValue),
            submenu_parent("Copy Value As", ContextSubmenu::CopyValueAs, current_submenu == ContextSubmenu::CopyValueAs),
            menu_item("Copy Path", Message::CopySelectedPath),
            separator(),
            submenu_parent("Export Value As", ContextSubmenu::ExportValueAs, current_submenu == ContextSubmenu::ExportValueAs),
        ];

        // Only show expand/collapse if node has children
        if has_children {
            menu_items.push(separator());
            menu_items.push(menu_item("Expand All Children", Message::ExpandAllChildren));
            menu_items.push(menu_item("Collapse All Children", Message::CollapseAllChildren));
        }

        let menu_content = column(menu_items)
            .spacing(0)
            .padding(4);

        let menu_box = container(menu_content)
            .style(move |_theme| container::Style {
                background: Some(colors.toolbar_bg.into()),
                border: Border {
                    color: colors.btn_border_top,
                    width: 1.0,
                    radius: Radius::from(6.0),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 10.0,
                },
                ..Default::default()
            });

        // Build submenu if one is open
        let submenu_box: Option<Element<'a, Message>> = match current_submenu {
            ContextSubmenu::CopyValueAs => {
                let submenu_content = column![
                    submenu_item("Minified", Message::CopyValueMinified),
                    submenu_item("Formatted", Message::CopyValueFormatted),
                ]
                .spacing(0)
                .padding(4);

                Some(container(submenu_content)
                    .style(move |_theme| container::Style {
                        background: Some(colors.toolbar_bg.into()),
                        border: Border {
                            color: colors.btn_border_top,
                            width: 1.0,
                            radius: Radius::from(6.0),
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 10.0,
                        },
                        ..Default::default()
                    })
                    .into())
            }
            ContextSubmenu::ExportValueAs => {
                let submenu_content = column![
                    submenu_item("JSON", Message::ExportAsJson),
                    submenu_item("Minified JSON", Message::ExportAsMinifiedJson),
                    submenu_item("Formatted JSON", Message::ExportAsFormattedJson),
                ]
                .spacing(0)
                .padding(4);

                Some(container(submenu_content)
                    .style(move |_theme| container::Style {
                        background: Some(colors.toolbar_bg.into()),
                        border: Border {
                            color: colors.btn_border_top,
                            width: 1.0,
                            radius: Radius::from(6.0),
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 10.0,
                        },
                        ..Default::default()
                    })
                    .into())
            }
            ContextSubmenu::None => None,
        };

        // Transparent backdrop that closes context menu on click anywhere
        let backdrop = mouse_area(Space::new().width(Fill).height(Fill))
            .on_press(Message::HideContextMenu);

        // Position menu at cursor location
        let clamped_x = menu_x.max(10.0);
        let clamped_y = menu_y.max(10.0);

        // Calculate submenu vertical offset based on which submenu is open
        let submenu_y_offset = match current_submenu {
            ContextSubmenu::CopyValueAs => 2.0 * 31.0,  // After 2 items (Copy Name, Copy Value)
            ContextSubmenu::ExportValueAs => 5.0 * 31.0,  // After separator and more items
            ContextSubmenu::None => 0.0,
        };

        // Build menu with optional submenu
        let menu_row: Element<'a, Message> = if let Some(submenu) = submenu_box {
            row![
                Space::new().width(Length::Fixed(clamped_x)),
                menu_box,
                Space::new().width(Length::Fixed(4.0)),
                column![
                    Space::new().height(Length::Fixed(submenu_y_offset)),
                    submenu,
                ]
            ].into()
        } else {
            row![
                Space::new().width(Length::Fixed(clamped_x)),
                menu_box,
            ].into()
        };

        stack![
            backdrop,
            column![
                Space::new().height(Length::Fixed(clamped_y)),
                menu_row,
            ]
        ].into()
    }

    /// Render the update check dialog overlay
    fn render_update_dialog(&self, colors: ThemeColors) -> Element<'_, Message> {
        let content: Element<'_, Message> = match &self.update_check_state {
            UpdateCheckState::Checking => {
                column![
                    text("Checking for Updates").size(16).color(colors.text_primary),
                    Space::new().height(Length::Fixed(15.0)),
                    text("Contacting GitHub...").size(13).color(colors.text_secondary),
                    Space::new().height(Length::Fixed(20.0)),
                ]
                .spacing(4)
                .padding(25)
                .into()
            }
            UpdateCheckState::UpToDate => {
                let current_version = env!("CARGO_PKG_VERSION");
                column![
                    text("You're Up to Date").size(16).color(colors.text_primary),
                    Space::new().height(Length::Fixed(15.0)),
                    text(format!("Unfold {} is the latest version.", current_version))
                        .size(13)
                        .color(colors.text_secondary),
                    Space::new().height(Length::Fixed(20.0)),
                    button(
                        text("OK").size(13).color(colors.text_primary)
                    )
                    .on_press(Message::DismissUpdateDialog)
                    .padding([8, 20])
                    .style(move |_theme, status| {
                        let bg = match status {
                            ButtonStatus::Hovered => colors.selected,
                            _ => colors.btn_border_top,
                        };
                        button::Style {
                            background: Some(bg.into()),
                            text_color: colors.text_primary,
                            border: Border {
                                radius: Radius::from(6.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
                ]
                .spacing(4)
                .padding(25)
                .align_x(iced::Alignment::Center)
                .into()
            }
            UpdateCheckState::UpdateAvailable { version, release_url } => {
                let url = release_url.clone();
                column![
                    text("Update Available").size(16).color(colors.text_primary),
                    Space::new().height(Length::Fixed(15.0)),
                    text(format!("A new version ({}) is available!", version))
                        .size(13)
                        .color(colors.text_secondary),
                    Space::new().height(Length::Fixed(20.0)),
                    row![
                        button(
                            text("Download").size(13)
                        )
                        .on_press(Message::OpenReleaseUrl(url))
                        .padding([8, 16])
                        .style(move |_theme, status| {
                            let bg = match status {
                                ButtonStatus::Hovered => Color::from_rgb(0.2, 0.6, 0.3),
                                _ => Color::from_rgb(0.2, 0.5, 0.2),
                            };
                            button::Style {
                                background: Some(bg.into()),
                                text_color: Color::WHITE,
                                border: Border {
                                    radius: Radius::from(6.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }),
                        Space::new().width(Length::Fixed(10.0)),
                        button(
                            text("Later").size(13).color(colors.text_secondary)
                        )
                        .on_press(Message::DismissUpdateDialog)
                        .padding([8, 16])
                        .style(move |_theme, status| {
                            let bg = match status {
                                ButtonStatus::Hovered => Some(colors.selected.into()),
                                _ => None,
                            };
                            button::Style {
                                background: bg,
                                text_color: colors.text_secondary,
                                border: Border {
                                    radius: Radius::from(6.0),
                                    color: colors.btn_border_top,
                                    width: 1.0,
                                },
                                ..Default::default()
                            }
                        }),
                    ]
                ]
                .spacing(4)
                .padding(25)
                .align_x(iced::Alignment::Center)
                .into()
            }
            UpdateCheckState::Error(msg) => {
                column![
                    text("Update Check Failed").size(16).color(colors.text_primary),
                    Space::new().height(Length::Fixed(15.0)),
                    text(msg).size(13).color(Color::from_rgb(0.9, 0.3, 0.3)),
                    Space::new().height(Length::Fixed(20.0)),
                    button(
                        text("OK").size(13).color(colors.text_primary)
                    )
                    .on_press(Message::DismissUpdateDialog)
                    .padding([8, 20])
                    .style(move |_theme, status| {
                        let bg = match status {
                            ButtonStatus::Hovered => colors.selected,
                            _ => colors.btn_border_top,
                        };
                        button::Style {
                            background: Some(bg.into()),
                            text_color: colors.text_primary,
                            border: Border {
                                radius: Radius::from(6.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
                ]
                .spacing(4)
                .padding(25)
                .align_x(iced::Alignment::Center)
                .into()
            }
            UpdateCheckState::None => {
                // Should not be rendered, but handle gracefully
                Space::new().into()
            }
        };

        let overlay_box = container(content)
            .style(move |_theme| container::Style {
                background: Some(colors.toolbar_bg.into()),
                border: Border {
                    color: colors.btn_border_top,
                    width: 1.0,
                    radius: Radius::from(8.0),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            });

        // Semi-transparent backdrop that closes on click (only if not checking)
        let backdrop = button(Space::new().width(Fill).height(Fill))
            .on_press_maybe(
                if self.update_check_state == UpdateCheckState::Checking {
                    None
                } else {
                    Some(Message::DismissUpdateDialog)
                }
            )
            .style(|_theme, _status| button::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                ..Default::default()
            })
            .width(Fill)
            .height(Fill);

        stack![
            backdrop,
            container(overlay_box)
                .width(Fill)
                .height(Fill)
                .center(Fill),
        ].into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::build_tree;
    use serde_json::json;

    #[test]
    fn test_menu_ids_are_unique() {
        // Ensure all menu IDs are distinct
        let ids = vec![
            menu_ids::CHECK_UPDATES,
            menu_ids::OPEN_FILE,
            menu_ids::OPEN_NEW_WINDOW,
            menu_ids::COPY_VALUE,
            menu_ids::COPY_KEY,
            menu_ids::COPY_PATH,
            menu_ids::TOGGLE_THEME,
            menu_ids::KEYBOARD_SHORTCUTS,
            menu_ids::EXPORT_JSON,
            menu_ids::EXPAND_ALL,
            menu_ids::COLLAPSE_ALL,
        ];

        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len(), "Menu IDs must be unique");
    }

    #[test]
    fn test_set_expanded_recursive() {
        // Create a tree with nested objects
        let value = json!({
            "level1": {
                "level2": {
                    "level3": "value"
                }
            }
        });
        let mut tree = build_tree(&value);

        // Get root index before mutably borrowing
        let root_index = tree.root_index();

        // Initially all nodes should be collapsed (expanded = false)
        // Expand all recursively from root
        App::set_expanded_recursive(&mut tree, root_index, true);

        // Check that expandable nodes are now expanded
        for i in 0..tree.node_count() {
            if let Some(node) = tree.get_node(i) {
                if node.is_expandable() {
                    assert!(node.expanded, "Node {} should be expanded", i);
                }
            }
        }

        // Collapse all recursively
        App::set_expanded_recursive(&mut tree, root_index, false);

        // Check that all nodes are collapsed
        for i in 0..tree.node_count() {
            if let Some(node) = tree.get_node(i) {
                if node.is_expandable() {
                    assert!(!node.expanded, "Node {} should be collapsed", i);
                }
            }
        }
    }

    #[test]
    fn test_node_to_json_string_primitives() {
        // Test primitive value serialization
        let value = json!({"str": "hello", "num": 42, "bool": true, "null": null});
        let tree = build_tree(&value);

        // Get root node and test children
        if let Some(root) = tree.get_node(tree.root_index()) {
            for &child_idx in &root.children {
                let json_str = App::node_to_json_string(&tree, child_idx);
                // Each child is a key-value pair, so we can verify it's valid JSON-ish
                assert!(!json_str.is_empty());
            }
        }
    }

    #[test]
    fn test_node_to_json_string_nested() {
        // Test nested object serialization
        let value = json!({"nested": {"key": "value"}});
        let tree = build_tree(&value);

        let json_str = App::node_to_json_string(&tree, tree.root_index());
        // Should contain the nested structure
        assert!(json_str.contains("nested"));
        assert!(json_str.contains("key"));
        assert!(json_str.contains("value"));
    }

    #[test]
    fn test_flatten_visible_nodes() {
        // Create a simple tree
        let value = json!({"a": 1, "b": 2});
        let tree = build_tree(&value);

        let flat_rows = App::flatten_visible_nodes(&tree);

        // Should have at least the root's children visible
        // (root is usually skipped, children shown)
        assert!(!flat_rows.is_empty());
    }

    #[test]
    fn test_search_nodes_basic() {
        let value = json!({"name": "Unfold", "version": "1.0"});
        let tree = build_tree(&value);

        // Search for "Unfold"
        let (results, error) = App::search_nodes(&tree, "Unfold", false, false);
        assert!(error.is_none());
        assert!(!results.is_empty(), "Should find 'Unfold'");

        // Search for non-existent
        let (results, error) = App::search_nodes(&tree, "nonexistent", false, false);
        assert!(error.is_none());
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_nodes_case_sensitive() {
        let value = json!({"Name": "Test"});
        let tree = build_tree(&value);

        // Case insensitive should find it
        let (results, _) = App::search_nodes(&tree, "name", false, false);
        assert!(!results.is_empty());

        // Case sensitive should not find lowercase
        let (results, _) = App::search_nodes(&tree, "name", true, false);
        assert!(results.is_empty());

        // Case sensitive should find exact match
        let (results, _) = App::search_nodes(&tree, "Name", true, false);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_nodes_regex() {
        let value = json!({"email": "test@example.com"});
        let tree = build_tree(&value);

        // Regex search for email pattern
        let (results, error) = App::search_nodes(&tree, r".*@.*\.com", false, true);
        assert!(error.is_none());
        assert!(!results.is_empty());

        // Invalid regex should return error
        let (results, error) = App::search_nodes(&tree, r"[invalid", false, true);
        assert!(error.is_some());
        assert!(results.is_empty());
    }

    #[test]
    fn test_theme_colors() {
        // Test that theme colors are valid
        let dark = get_theme_colors(AppTheme::Dark);
        let light = get_theme_colors(AppTheme::Light);

        // Colors should be different between themes
        assert_ne!(dark.background, light.background);
        assert_ne!(dark.text_primary, light.text_primary);
    }

    // ===== Context Menu Tests =====

    #[test]
    fn test_context_submenu_enum() {
        // Test that ContextSubmenu values are distinct and comparable
        assert_eq!(ContextSubmenu::None, ContextSubmenu::None);
        assert_ne!(ContextSubmenu::None, ContextSubmenu::CopyValueAs);
        assert_ne!(ContextSubmenu::CopyValueAs, ContextSubmenu::ExportValueAs);
    }

    #[test]
    fn test_update_check_state_enum() {
        // Test that UpdateCheckState values are distinct and comparable
        assert_eq!(UpdateCheckState::None, UpdateCheckState::None);
        assert_eq!(UpdateCheckState::Checking, UpdateCheckState::Checking);
        assert_eq!(UpdateCheckState::UpToDate, UpdateCheckState::UpToDate);

        assert_ne!(UpdateCheckState::None, UpdateCheckState::Checking);
        assert_ne!(UpdateCheckState::Checking, UpdateCheckState::UpToDate);

        // Test UpdateAvailable variant
        let update1 = UpdateCheckState::UpdateAvailable {
            version: "v1.0.0".to_string(),
            release_url: "https://github.com/test".to_string(),
        };
        let update2 = UpdateCheckState::UpdateAvailable {
            version: "v1.0.0".to_string(),
            release_url: "https://github.com/test".to_string(),
        };
        assert_eq!(update1, update2);

        // Test Error variant
        let error1 = UpdateCheckState::Error("Network error".to_string());
        let error2 = UpdateCheckState::Error("Network error".to_string());
        assert_eq!(error1, error2);
        assert_ne!(UpdateCheckState::Error("a".to_string()), UpdateCheckState::Error("b".to_string()));
    }

    #[test]
    fn test_node_to_json_string_minified() {
        // Test minified JSON output has no spaces
        let value = json!({"key": "value", "nested": {"a": 1, "b": 2}});
        let tree = build_tree(&value);

        let minified = App::node_to_json_string_minified(&tree, tree.root_index());

        // Minified should not have spaces after colons or commas
        assert!(!minified.contains(": "), "Minified should not have ': ' (colon-space)");
        assert!(!minified.contains(", "), "Minified should not have ', ' (comma-space)");

        // But should still have colons and commas
        assert!(minified.contains(":"), "Should contain colons");
        assert!(minified.contains(","), "Should contain commas");
    }

    #[test]
    fn test_node_to_json_string_regular_vs_minified() {
        // Test that regular has spaces, minified doesn't
        let value = json!({"a": 1, "b": 2});
        let tree = build_tree(&value);

        let regular = App::node_to_json_string(&tree, tree.root_index());
        let minified = App::node_to_json_string_minified(&tree, tree.root_index());

        // Regular should be longer due to spaces
        assert!(regular.len() > minified.len(), "Regular should be longer than minified");

        // Both should produce valid JSON structure
        assert!(regular.starts_with('{'));
        assert!(regular.ends_with('}'));
        assert!(minified.starts_with('{'));
        assert!(minified.ends_with('}'));
    }

    #[test]
    fn test_minified_json_array() {
        // Test minified JSON for arrays
        let value = json!([1, 2, 3, "test"]);
        let tree = build_tree(&value);

        let minified = App::node_to_json_string_minified(&tree, tree.root_index());

        // Should be compact
        assert!(!minified.contains(", "));
        assert!(minified.starts_with('['));
        assert!(minified.ends_with(']'));
    }

    #[test]
    fn test_minified_json_with_special_chars() {
        // Test that special characters in strings are properly escaped
        let value = json!({"text": "line1\nline2\ttab"});
        let tree = build_tree(&value);

        let minified = App::node_to_json_string_minified(&tree, tree.root_index());

        // Should have escaped newline and tab
        assert!(minified.contains("\\n"), "Should escape newlines");
        assert!(minified.contains("\\t"), "Should escape tabs");
    }

    #[test]
    fn test_minified_json_with_quotes() {
        // Test that quotes in strings are properly escaped
        let value = json!({"text": "he said \"hello\""});
        let tree = build_tree(&value);

        let minified = App::node_to_json_string_minified(&tree, tree.root_index());

        // Should have escaped quotes
        assert!(minified.contains("\\\""), "Should escape quotes");
    }

    #[test]
    fn test_json_primitives_minified() {
        // Test primitive values
        let null_val = json!(null);
        let bool_val = json!(true);
        let num_val = json!(42);
        let str_val = json!("hello");

        let null_tree = build_tree(&null_val);
        let bool_tree = build_tree(&bool_val);
        let num_tree = build_tree(&num_val);
        let str_tree = build_tree(&str_val);

        assert_eq!(App::node_to_json_string_minified(&null_tree, null_tree.root_index()), "null");
        assert_eq!(App::node_to_json_string_minified(&bool_tree, bool_tree.root_index()), "true");
        assert_eq!(App::node_to_json_string_minified(&num_tree, num_tree.root_index()), "42");
        assert_eq!(App::node_to_json_string_minified(&str_tree, str_tree.root_index()), "\"hello\"");
    }

    #[test]
    fn test_format_node_value_for_copy() {
        // Test the copy value formatting
        let value = json!({"str": "hello", "num": 42});
        let tree = build_tree(&value);

        // Find the string child node
        if let Some(root) = tree.get_node(tree.root_index()) {
            for &child_idx in &root.children {
                let copy_value = App::format_node_value_for_copy(&tree, child_idx);
                // Should produce a non-empty string
                assert!(!copy_value.is_empty());
            }
        }
    }

    #[test]
    fn test_deeply_nested_minified() {
        // Test deeply nested structure
        let value = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": [1, 2, 3]
                    }
                }
            }
        });
        let tree = build_tree(&value);

        let minified = App::node_to_json_string_minified(&tree, tree.root_index());

        // Should be compact throughout all levels
        assert!(!minified.contains(": "));
        assert!(!minified.contains(", "));

        // Should contain all the nested keys
        assert!(minified.contains("level1"));
        assert!(minified.contains("level2"));
        assert!(minified.contains("level3"));
        assert!(minified.contains("value"));
    }

    #[test]
    fn test_empty_object_and_array() {
        let empty_obj = json!({});
        let empty_arr = json!([]);

        let obj_tree = build_tree(&empty_obj);
        let arr_tree = build_tree(&empty_arr);

        assert_eq!(App::node_to_json_string_minified(&obj_tree, obj_tree.root_index()), "{}");
        assert_eq!(App::node_to_json_string_minified(&arr_tree, arr_tree.root_index()), "[]");
    }
}
