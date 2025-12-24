mod parser;

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::widget::scrollable::Viewport;
use iced::{Element, Font, Length, Center, Fill, Color, Size, Task, window};

// Color scheme for syntax highlighting
const COLOR_KEY: Color = Color::from_rgb(0.4, 0.7, 0.9);       // Light blue for keys
const COLOR_STRING: Color = Color::from_rgb(0.6, 0.8, 0.5);    // Green for strings
const COLOR_NUMBER: Color = Color::from_rgb(0.9, 0.7, 0.4);    // Orange for numbers
const COLOR_BOOL: Color = Color::from_rgb(0.8, 0.5, 0.7);      // Purple for booleans
const COLOR_NULL: Color = Color::from_rgb(0.6, 0.6, 0.6);      // Gray for null
const COLOR_BRACKET: Color = Color::from_rgb(0.7, 0.7, 0.7);   // Light gray for brackets
const COLOR_INDICATOR: Color = Color::from_rgb(0.5, 0.5, 0.5); // Dim for expand indicator
const COLOR_ROW_ODD: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.03); // Subtle alternating stripe

// Virtual scrolling constants
const ROW_HEIGHT: f32 = 16.0;      // Fixed height per row (tight for connected tree lines)
const BUFFER_ROWS: usize = 5;      // Extra rows above/below (reduced for performance)

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
        .title("Unfold - JSON Viewer")
        .run()
}

// The application state (Model)
struct App {
    // The loaded JSON tree (None if no file loaded)
    tree: Option<JsonTree>,
    // Status message to show the user
    status: String,
    // Currently loaded file path
    current_file: Option<PathBuf>,
    // Display preferences
    preferences: Preferences,
    // Time taken to load and parse the file
    load_time: Option<Duration>,
    // Flattened rows for virtual scrolling (rebuilt when tree changes)
    flat_rows: Vec<FlatRow>,
    // Viewport height in pixels (updated on resize)
    viewport_height: f32,
    // Current scroll offset in pixels (for virtual scrolling)
    scroll_offset: f32,
}

// User-configurable display preferences
#[derive(Debug, Clone)]
struct Preferences {
    // Number of spaces per indent level (2, 4, etc.)
    indent_size: usize,
    // Show tree connector lines (├─, └─, │)
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
        }
    }
}

// Messages that can be sent to update the app
#[derive(Debug, Clone)]
enum Message {
    OpenFileDialog,                    // User clicked "Open File" button
    FileSelected(Option<PathBuf>),     // File dialog returned (None if cancelled)
    ToggleNode(usize),
    Scrolled(Viewport),                // User scrolled the tree view
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

        // Build prefix (same logic as collect_nodes)
        let (current_prefix, child_prefix) = if is_root {
            (String::new(), String::new())
        } else if node.depth == 1 {
            let connector = if is_last { "└─ " } else { "├─ " };
            let child = if is_last { "   ".to_string() } else { "│  ".to_string() };
            (connector.to_string(), child)
        } else {
            let connector = if is_last { "└─ " } else { "├─ " };
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
            let indicator = if flat_row.is_expanded { "⊟" } else { "⊞" };

            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(COLOR_BRACKET).into(),
                text(indicator).font(Font::MONOSPACE).size(13).color(COLOR_INDICATOR).into(),
                text(" ").font(Font::MONOSPACE).size(13).into(),
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
            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(COLOR_BRACKET).into(),
                text("  ").font(Font::MONOSPACE).size(13).into(),
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

        // Wrap in container with zebra striping
        // Use large min-width so zebra extends beyond viewport for horizontal scroll
        let row_container = if flat_row.row_index % 2 == 1 {
            container(node_row)
                .width(Length::Fixed(5000.0))  // Wide enough for most content
                .height(Length::Fixed(ROW_HEIGHT))
                .style(|_theme| container::Style {
                    background: Some(COLOR_ROW_ODD.into()),
                    ..Default::default()
                })
        } else {
            container(node_row)
                .width(Length::Fixed(5000.0))  // Wide enough for most content
                .height(Length::Fixed(ROW_HEIGHT))
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

                                        // Auto-resize window to fit content
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

                let open_button = button(text("Open File..."))
                    .on_press(Message::OpenFileDialog)
                    .padding(10);

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

        // When file is loaded, show tree + status bar
        if self.tree.is_some() {
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

            column![tree_view, status_bar].into()
        } else {
            tree_view  // This is the welcome screen
        }
    }
}
