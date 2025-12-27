mod parser;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::widget::scrollable::Viewport;
use iced::{Element, Font, Length, Center, Fill, Color, Size, Task, window, Border, Shadow};
use iced::border::Radius;
use iced::advanced::widget::{Id as WidgetId, operate};
use iced::advanced::widget::operation::scrollable::{scroll_to, AbsoluteOffset};
use std::collections::HashSet;

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

use parser::{JsonTree, JsonValue};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

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
}

pub fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .window_size((900.0, 700.0))  // Default window size
        .resizable(true)               // Allow window resizing
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
    search_results: Vec<usize>,        // Node indices that match the search
    search_result_index: Option<usize>, // Current result (0-based index into search_results)
    search_matches: HashSet<usize>,    // Set of matching node indices for O(1) lookup during render
    // Scrollable ID for programmatic scrolling
    tree_scrollable_id: WidgetId,
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
            viewport_height: 600.0,  // Default, will be updated
            scroll_offset: 0.0,
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_index: None,
            search_matches: HashSet::new(),
            tree_scrollable_id: WidgetId::unique(),
        }
    }
}

// Messages that can be sent to update the app
#[derive(Debug, Clone)]
enum Message {
    OpenFileDialog,
    FileSelected(Option<PathBuf>),
    ToggleNode(usize),
    Scrolled(Viewport),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
}

impl App {
    // Initialize the application (called once at startup)
    fn boot() -> (Self, Task<Message>) {
        (App::default(), Task::none())
    }

    /// Flatten the tree into a Vec<FlatRow> for virtual scrolling
    /// This walks only expanded nodes, pre-computing all display data
    /// Note: This is a static method to avoid borrow checker issues
    fn flatten_visible_nodes(tree: &JsonTree) -> Vec<FlatRow> {
        let mut rows = Vec::new();

        // Start from root's children (skip root node like collect_nodes does)
        if let Some(root) = tree.get_node(tree.root_index()) {
            let child_count = root.children.len();
            for (i, &child_index) in root.children.iter().enumerate() {
                let is_last = i == child_count - 1;
                Self::flatten_node(tree, child_index, &mut rows, "", is_last, false);
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
        });

        // Recurse into children if expanded
        if node.expanded {
            let child_count = node.children.len();
            for (i, &child_index) in node.children.iter().enumerate() {
                let is_last_child = i == child_count - 1;
                Self::flatten_node(tree, child_index, rows, &child_prefix, is_last_child, false);
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
            // Leaf node - not clickable
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

            row(row_elements).spacing(0).into()
        };

        // Determine background color based on search state and zebra striping
        let is_match = self.search_matches.contains(&flat_row.node_index);
        let is_current_result = self.search_result_index
            .map(|i| self.search_results.get(i) == Some(&flat_row.node_index))
            .unwrap_or(false);

        let background_color = if is_current_result {
            Some(COLOR_SEARCH_CURRENT)
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
                                        self.status = format!("✗ Parse error: {}", e);
                                        self.tree = None;
                                        self.current_file = None;
                                    }
                                }
                            }
                            Err(e) => {
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
                self.search_query = query.clone();

                // Perform search if query is not empty
                if query.is_empty() {
                    self.search_results.clear();
                    self.search_result_index = None;
                    self.search_matches.clear();
                    Task::none()
                } else if let Some(tree) = &self.tree {
                    // Search all nodes for matches
                    self.search_results = Self::search_nodes(tree, &query);
                    self.search_matches = self.search_results.iter().cloned().collect();

                    // Set to first result if any found
                    if !self.search_results.is_empty() {
                        self.search_result_index = Some(0);
                        let target = self.search_results[0];
                        // Expand path to first result
                        self.expand_to_node(target);
                        // Rebuild flat_rows BEFORE scrolling (so we can find the row)
                        self.flat_rows = Self::flatten_visible_nodes(self.tree.as_ref().unwrap());
                        // Return scroll task
                        self.scroll_to_node(target)
                    } else {
                        self.search_result_index = None;
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                        Task::none()
                    }
                } else {
                    Task::none()
                }
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
        }
    }

    /// Search all nodes in the tree for matches (case-insensitive)
    fn search_nodes(tree: &JsonTree, query: &str) -> Vec<usize> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Iterate through all nodes
        for i in 0..tree.node_count() {
            if let Some(node) = tree.get_node(i) {
                // Check key
                if let Some(key) = &node.key {
                    if key.to_lowercase().contains(&query_lower) {
                        results.push(i);
                        continue;
                    }
                }

                // Check value
                let value_matches = match &node.value {
                    JsonValue::String(s) => s.to_lowercase().contains(&query_lower),
                    JsonValue::Number(n) => n.to_string().contains(&query_lower),
                    JsonValue::Bool(b) => b.to_string().contains(&query_lower),
                    JsonValue::Null => "null".contains(&query_lower),
                    _ => false,
                };

                if value_matches {
                    results.push(i);
                }
            }
        }

        results
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
                // Show welcome screen when no file loaded
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

                let welcome = column![
                    header,
                    open_button,
                ]
                .spacing(20)
                .align_x(Center);

                container(welcome)
                    .width(Fill)
                    .height(Fill)
                    .center(Fill)
                    .into()
            }
        };

        // When file is loaded, show toolbar + tree + status bar
        if self.tree.is_some() {
            // Search toolbar
            let search_input = text_input("Search...", &self.search_query)
                .on_input(Message::SearchQueryChanged)
                .padding(5)
                .width(Length::Fixed(250.0));

            let search_result_text = if self.search_results.is_empty() {
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
                    search_input,
                    Space::new().width(Length::Fixed(15.0)),
                    prev_button,
                    Space::new().width(Length::Fixed(5.0)),
                    next_button,
                    Space::new().width(Length::Fixed(15.0)),
                    text(search_result_text).size(12).color(COLOR_BRACKET),
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

            let status_bar = container(
                row![
                    text(node_count).size(12).color(COLOR_BRACKET),
                    text("  ").size(12),
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
