mod parser;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::widget::scrollable::Viewport;
use iced::{Element, Font, Length, Center, Fill, Color, Size, Task, window, Border, Shadow, Subscription, clipboard};
use iced::border::Radius;
use iced::advanced::widget::{Id as WidgetId, operate};
use iced::advanced::widget::operation::scrollable::{scroll_to, AbsoluteOffset};
use iced::advanced::widget::operation::focusable;
use iced::keyboard::{self, Key, Modifiers, key::Named};
use std::collections::HashSet;
use regex::Regex;

// Color scheme for syntax highlighting
const COLOR_KEY: Color = Color::from_rgb(0.4, 0.7, 0.9);       // Light blue for keys
const COLOR_STRING: Color = Color::from_rgb(0.6, 0.8, 0.5);    // Green for strings
const COLOR_NUMBER: Color = Color::from_rgb(0.9, 0.7, 0.4);    // Orange for numbers
const COLOR_BOOL: Color = Color::from_rgb(0.8, 0.5, 0.7);      // Purple for booleans
const COLOR_NULL: Color = Color::from_rgb(0.6, 0.6, 0.6);      // Gray for null
const COLOR_BRACKET: Color = Color::from_rgb(0.7, 0.7, 0.7);   // Light gray for brackets
const COLOR_INDICATOR: Color = Color::from_rgb(0.5, 0.5, 0.5); // Dim for expand indicator
const COLOR_ROW_ODD: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.03); // Subtle alternating stripe
const COLOR_SEARCH_MATCH: Color = Color::from_rgba(0.9, 0.7, 0.2, 0.3); // Yellow highlight for search matches
const COLOR_SEARCH_CURRENT: Color = Color::from_rgba(0.9, 0.5, 0.1, 0.5); // Orange for current result
const COLOR_SELECTED: Color = Color::from_rgba(0.3, 0.5, 0.8, 0.3); // Blue highlight for selected node
const COLOR_ERROR: Color = Color::from_rgb(0.9, 0.4, 0.4);         // Red for error messages
const COLOR_ERROR_CONTEXT: Color = Color::from_rgb(0.7, 0.7, 0.5); // Muted yellow for context line

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

// Button colors for 3D effect
const COLOR_BTN_BG: Color = Color::from_rgb(0.28, 0.28, 0.30);
const COLOR_BTN_BG_HOVER: Color = Color::from_rgb(0.32, 0.32, 0.35);
const COLOR_BTN_BORDER_TOP: Color = Color::from_rgb(0.45, 0.45, 0.48);
const COLOR_BTN_BORDER_BOTTOM: Color = Color::from_rgb(0.15, 0.15, 0.17);
const COLOR_BTN_DISABLED: Color = Color::from_rgb(0.22, 0.22, 0.24);

// Virtual scrolling constants
const ROW_HEIGHT: f32 = 16.0;      // Fixed height per row (tight for connected tree lines)
const BUFFER_ROWS: usize = 5;      // Extra rows above/below (reduced for performance)

use iced::widget::button::Status as ButtonStatus;

/// Custom 3D button style with raised appearance
fn button_3d_style(_theme: &iced::Theme, status: ButtonStatus) -> button::Style {
    let (bg_color, text_color, border_color) = match status {
        ButtonStatus::Active => (COLOR_BTN_BG, Color::from_rgb(0.9, 0.9, 0.9), COLOR_BTN_BORDER_TOP),
        ButtonStatus::Hovered => (COLOR_BTN_BG_HOVER, Color::WHITE, COLOR_BTN_BORDER_TOP),
        ButtonStatus::Pressed => (COLOR_BTN_BORDER_BOTTOM, Color::from_rgb(0.8, 0.8, 0.8), COLOR_BTN_BORDER_BOTTOM),
        ButtonStatus::Disabled => (COLOR_BTN_DISABLED, Color::from_rgb(0.5, 0.5, 0.5), COLOR_BTN_DISABLED),
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

/// Toggle button style - highlighted when active
fn button_toggle_style(is_active: bool) -> impl Fn(&iced::Theme, ButtonStatus) -> button::Style {
    move |_theme: &iced::Theme, status: ButtonStatus| {
        let active_bg = Color::from_rgb(0.3, 0.5, 0.7);      // Blue when active
        let active_border = Color::from_rgb(0.4, 0.6, 0.8);

        let (bg_color, text_color, border_color) = match (is_active, status) {
            (true, ButtonStatus::Active) => (active_bg, Color::WHITE, active_border),
            (true, ButtonStatus::Hovered) => (Color::from_rgb(0.35, 0.55, 0.75), Color::WHITE, active_border),
            (true, ButtonStatus::Pressed) => (Color::from_rgb(0.25, 0.45, 0.65), Color::WHITE, active_border),
            (false, ButtonStatus::Active) => (COLOR_BTN_BG, Color::from_rgb(0.7, 0.7, 0.7), COLOR_BTN_BORDER_TOP),
            (false, ButtonStatus::Hovered) => (COLOR_BTN_BG_HOVER, Color::from_rgb(0.9, 0.9, 0.9), COLOR_BTN_BORDER_TOP),
            (false, ButtonStatus::Pressed) => (COLOR_BTN_BORDER_BOTTOM, Color::from_rgb(0.8, 0.8, 0.8), COLOR_BTN_BORDER_BOTTOM),
            (_, ButtonStatus::Disabled) => (COLOR_BTN_DISABLED, Color::from_rgb(0.5, 0.5, 0.5), COLOR_BTN_DISABLED),
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
    /// Color for the value
    value_color: Color,
    /// Is this node expandable (has children)?
    is_expandable: bool,
    /// Is this node currently expanded?
    is_expanded: bool,
    /// Row index in flattened list (for zebra striping)
    row_index: usize,
    /// JSON path to this node (e.g., "users[2].email")
    path: String,
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

impl Default for App {
    fn default() -> Self {
        App {
            tree: None,
            status: String::from("No file loaded"),
            current_file: None,
            preferences: Preferences::default(),
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
    // Copy selected node's path to clipboard
    CopySelectedPath,
}

impl App {
    // Initialize the application (called once at startup)
    // Checks for CLI arguments: `unfold myfile.json`
    fn boot() -> (Self, Task<Message>) {
        let app = App::default();

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

    // Subscription: Listen for keyboard events
    // This is called continuously - Iced manages the event loop
    //
    // Key Rust concept: `keyboard::listen()` returns a Subscription<keyboard::Event>
    // We use `.filter_map()` to only emit messages for events we care about.
    // filter_map combines filter (skip some) and map (transform) in one step:
    // - Return Some(value) to include the transformed value
    // - Return None to skip the event entirely
    fn subscription(&self) -> Subscription<Message> {
        // keyboard::listen() subscribes to all keyboard events that aren't
        // consumed by widgets (like text input). We handle KeyPressed and
        // ModifiersChanged (to track Shift state for search input).
        keyboard::listen().filter_map(|event| {
            match event {
                keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    // Handle key presses
                    Some(Message::KeyPressed(key, modifiers))
                }
                keyboard::Event::ModifiersChanged(modifiers) => {
                    // Track modifier state (for Shift+Enter in search input)
                    Some(Message::ModifiersChanged(modifiers))
                }
                // Ignore key releases
                _ => None
            }
        })
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
        let (value_display, value_color) = match &node.value {
            JsonValue::Null => ("null".to_string(), COLOR_NULL),
            JsonValue::Bool(b) => (b.to_string(), COLOR_BOOL),
            JsonValue::Number(n) => (n.to_string(), COLOR_NUMBER),
            JsonValue::String(s) => (format!("\"{}\"", s), COLOR_STRING),
            JsonValue::Array => {
                if node.expanded {
                    (":".to_string(), COLOR_BRACKET)
                } else {
                    ("[...]".to_string(), COLOR_KEY)
                }
            }
            JsonValue::Object => {
                if node.expanded {
                    (":".to_string(), COLOR_BRACKET)
                } else {
                    ("{...}".to_string(), COLOR_KEY)
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
            value_color,
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
        // Build the row element
        let node_row: Element<'a, Message> = if flat_row.is_expandable {
            // Expandable node - make it clickable
            // Icon replaces the "─" part of the connector for alignment
            let indicator = if flat_row.is_expanded { "⊟ " } else { "⊞ " };

            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(COLOR_BRACKET).into(),
                text(indicator).font(Font::MONOSPACE).size(13).color(COLOR_INDICATOR).into(),
            ];

            // Show key if it exists (empty keys shown as "" for visibility)
            if let Some(k) = &flat_row.key {
                let display_key = if k.is_empty() { "\"\"".to_string() } else { k.clone() };
                row_elements.push(
                    text(display_key)
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(COLOR_KEY)
                        .into()
                );
                row_elements.push(
                    text(": ")
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(COLOR_BRACKET)
                        .into()
                );
            }

            // Show value preview for collapsed containers ({...} or [...])
            if !flat_row.is_expanded {
                row_elements.push(
                    text(flat_row.value_display.clone())
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(flat_row.value_color)
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
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(COLOR_BRACKET).into(),
                text("─ ").font(Font::MONOSPACE).size(13).color(COLOR_BRACKET).into(),
            ];

            // Show key if it exists (empty keys shown as "" for visibility)
            if let Some(k) = &flat_row.key {
                let display_key = if k.is_empty() { "\"\"".to_string() } else { k.clone() };
                row_elements.push(
                    text(display_key)
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(COLOR_KEY)
                        .into()
                );
                row_elements.push(
                    text(": ")
                        .font(Font::MONOSPACE)
                        .size(13)
                        .color(COLOR_BRACKET)
                        .into()
                );
            }

            row_elements.push(
                text(flat_row.value_display.clone())
                    .font(Font::MONOSPACE)
                    .size(13)
                    .color(flat_row.value_color)
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
            Some(COLOR_SEARCH_CURRENT)
        } else if is_selected {
            Some(COLOR_SELECTED)
        } else if is_match {
            Some(COLOR_SEARCH_MATCH)
        } else if flat_row.row_index % 2 == 1 {
            Some(COLOR_ROW_ODD)
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

        row_container.into()
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
                    // Escape: Clear search
                    Key::Named(Named::Escape) => {
                        self.update(Message::ClearSearch)
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
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && !modifiers.shift() => {
                        self.update(Message::CopySelectedValue)
                    }
                    // Cmd/Ctrl+Shift+C: Copy selected node path to clipboard
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && modifiers.shift() => {
                        self.update(Message::CopySelectedPath)
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
                if let Some(node_index) = self.selected_node {
                    // Find the path from flat_rows
                    if let Some(flat_row) = self.flat_rows.iter().find(|r| r.node_index == node_index) {
                        return clipboard::write(flat_row.path.clone());
                    }
                }
                Task::none()
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
        if let Some(node) = tree.get_node(node_index) {
            match &node.value {
                JsonValue::Null => "null".to_string(),
                JsonValue::Bool(b) => b.to_string(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
                JsonValue::Array => {
                    let items: Vec<String> = node.children.iter()
                        .map(|&child_idx| Self::node_to_json_string(tree, child_idx))
                        .collect();
                    format!("[{}]", items.join(", "))
                }
                JsonValue::Object => {
                    let items: Vec<String> = node.children.iter()
                        .filter_map(|&child_idx| {
                            tree.get_node(child_idx).map(|child| {
                                let key = child.key.as_deref().unwrap_or("");
                                let value = Self::node_to_json_string(tree, child_idx);
                                format!("\"{}\": {}", key, value)
                            })
                        })
                        .collect();
                    format!("{{{}}}", items.join(", "))
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
                    let error_icon = text("⚠").size(48).color(COLOR_ERROR);

                    let error_title = text(format!("Failed to parse {}", error.filename))
                        .size(18)
                        .color(COLOR_ERROR);

                    let error_message = text(&error.message)
                        .size(14)
                        .color(Color::WHITE);

                    // Location info (line:column)
                    let location_text = match (error.line, error.column) {
                        (Some(line), Some(col)) => format!("Line {}, Column {}", line, col),
                        (Some(line), None) => format!("Line {}", line),
                        _ => String::new(),
                    };
                    let location = text(location_text)
                        .size(13)
                        .color(COLOR_BRACKET);

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
                                .color(COLOR_ERROR_CONTEXT),
                            text(caret)
                                .size(12)
                                .font(Font::MONOSPACE)
                                .color(COLOR_ERROR),
                        ]
                        .spacing(0)
                        .into()
                    } else {
                        Space::new().into()
                    };

                    let try_again_button = button(text("Try Another File...").size(14))
                        .on_press(Message::OpenFileDialog)
                        .padding([8, 16])
                        .style(button_3d_style);

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
                        .into()
                } else {
                    // Normal welcome screen
                    let header = column![
                        text("Unfold").size(32),
                        text("JSON Viewer").size(16).color(COLOR_BRACKET),
                    ]
                    .spacing(5)
                    .align_x(Center);

                    let open_button = button(text("Open File...").size(14))
                        .on_press(Message::OpenFileDialog)
                        .padding([8, 16])
                        .style(button_3d_style);

                    let open_new_window_button = button(text("Open in New Window...").size(12))
                        .on_press(Message::OpenFileInNewWindow)
                        .padding([6, 12])
                        .style(button_3d_style);

                    let welcome = column![
                        header,
                        open_button,
                        open_new_window_button,
                    ]
                    .spacing(15)
                    .align_x(Center);

                    container(welcome)
                        .width(Fill)
                        .height(Fill)
                        .center(Fill)
                        .into()
                }
            }
        };

        // When file is loaded, show toolbar + tree + status bar
        if self.tree.is_some() {
            // Search option toggle buttons (Dadroit style)
            let case_button = button(text("Aa").size(11))
                .padding([4, 8])
                .style(button_toggle_style(self.search_case_sensitive))
                .on_press(Message::ToggleCaseSensitive);

            let regex_button = button(text(".*").size(11))
                .padding([4, 8])
                .style(button_toggle_style(self.search_use_regex))
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
            .style(button_3d_style);

            let next_button = button(
                text("Next ▸").size(11)
            )
            .padding([5, 12])
            .style(button_3d_style);

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
                    text(search_result_text).size(11).color(COLOR_BRACKET),
                ]
                .align_y(Center)
            )
            .width(Fill)
            .padding([8, 10])
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.12, 0.12, 0.12).into()),
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
                    text(node_count).size(12).color(COLOR_BRACKET),
                    text("  |  ").size(12).color(COLOR_BRACKET),
                    text(path_display).size(12).color(COLOR_KEY),
                    Space::new().width(Length::Fill),
                    text(load_time_str).size(12).color(COLOR_BRACKET),
                ]
            )
            .width(Fill)
            .padding([5, 10])
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.15, 0.15, 0.15).into()),
                ..Default::default()
            });

            column![toolbar, tree_view, status_bar].into()
        } else {
            tree_view  // This is the welcome screen
        }
    }
}
