mod parser;

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Font, Length, Center, Fill, Color, Size, Task, window};

// Color scheme for syntax highlighting
const COLOR_KEY: Color = Color::from_rgb(0.4, 0.7, 0.9);       // Light blue for keys
const COLOR_STRING: Color = Color::from_rgb(0.6, 0.8, 0.5);    // Green for strings
const COLOR_NUMBER: Color = Color::from_rgb(0.9, 0.7, 0.4);    // Orange for numbers
const COLOR_BOOL: Color = Color::from_rgb(0.8, 0.5, 0.7);      // Purple for booleans
const COLOR_NULL: Color = Color::from_rgb(0.6, 0.6, 0.6);      // Gray for null
const COLOR_BRACKET: Color = Color::from_rgb(0.7, 0.7, 0.7);   // Light gray for brackets
const COLOR_INDICATOR: Color = Color::from_rgb(0.5, 0.5, 0.5); // Dim for expand indicator
use parser::{JsonTree, JsonValue};
use std::fs;
use std::path::PathBuf;

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
        }
    }
}

// Messages that can be sent to update the app
#[derive(Debug, Clone)]
enum Message {
    OpenFileDialog,                    // User clicked "Open File" button
    FileSelected(Option<PathBuf>),     // File dialog returned (None if cancelled)
    ToggleNode(usize),
}

impl App {
    // Initialize the application (called once at startup)
    fn boot() -> (Self, Task<Message>) {
        (App::default(), Task::none())
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
        // prefix (3 chars per depth) + indicator (2) + key + " : " + value
        let prefix_len = depth * 3;
        let indicator_len = 2;
        let key_len = node.key.as_ref().map(|k| k.len() + 3).unwrap_or(0);  // key + " : "
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
                        // Try to load the file
                        match fs::read_to_string(&path) {
                            Ok(contents) => {
                                match serde_json::from_str::<serde_json::Value>(&contents) {
                                    Ok(json_value) => {
                                        let tree = parser::build_tree(&json_value);
                                        let filename = path.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "unknown".to_string());
                                        self.status = format!("✓ {} ({} nodes)", filename, tree.node_count());
                                        self.tree = Some(tree);
                                        self.current_file = Some(path);

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
                }
                Task::none()  // No async work needed
            }
        }
    }

    // Render the UI
    fn view(&self) -> Element<'_, Message> {
        // Tree display section
        let tree_view: Element<'_, Message> = match &self.tree {
            Some(tree) => {
                // Build interactive tree view
                let mut elements: Vec<Element<'_, Message>> = Vec::new();

                // Skip the root node, directly render its children (like Dadroit)
                if let Some(root) = tree.get_node(tree.root_index()) {
                    let child_count = root.children.len();
                    for (i, &child_index) in root.children.iter().enumerate() {
                        let is_last = i == child_count - 1;
                        self.collect_nodes(tree, child_index, &mut elements, "", is_last, false);
                    }
                }

                let nodes_column = column(elements).spacing(0);  // No spacing so │ lines connect

                scrollable(
                    container(nodes_column)
                        .padding([10, 5])  // Less horizontal padding, tree starts near border
                )
                .height(Length::Fill)
                .width(Fill)
                .direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::default(),
                    horizontal: scrollable::Scrollbar::default(),
                })
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

        // When file is loaded, just show the tree (full screen)
        if self.tree.is_some() {
            tree_view
        } else {
            tree_view  // This is the welcome screen
        }
    }

    // Recursively collect tree nodes into a Vec
    // `prefix` contains the tree lines for ancestor levels (│ or space)
    // `is_last` indicates if this node is the last child of its parent
    // `is_root` indicates if this is the root node (no prefix needed)
    fn collect_nodes<'a>(
        &self,
        tree: &JsonTree,
        index: usize,
        elements: &mut Vec<Element<'a, Message>>,
        prefix: &str,
        is_last: bool,
        is_root: bool,
    ) {
        let Some(node) = tree.get_node(index) else {
            return;
        };

        let prefs = &self.preferences;

        // Build the tree line prefix for this node
        // Dadroit style: first level has no connectors, deeper levels have simple connectors
        let (current_prefix, child_prefix) = if is_root {
            // Root node has no prefix
            (String::new(), String::new())
        } else if node.depth == 1 {
            // First level (direct children of root) - no connectors, like Dadroit
            (String::new(), "   ".to_string())  // Children get indentation
        } else {
            // Deeper levels: simple connectors with indentation
            let connector = if is_last { "└─ " } else { "├─ " };
            let current = format!("{}{}", prefix, connector);

            // Children get more indentation (spaces only, no vertical lines)
            let child = format!("{}   ", prefix);  // 3 spaces for alignment

            (current, child)
        };

        // Format the value part with appropriate color
        // Collapsed containers show {...} or [...], expanded just show :
        let (value_str, value_color) = match &node.value {
            JsonValue::Null => ("null".to_string(), COLOR_NULL),
            JsonValue::Bool(b) => (b.to_string(), COLOR_BOOL),
            JsonValue::Number(n) => (n.to_string(), COLOR_NUMBER),
            JsonValue::String(s) => (format!("\"{}\"", s), COLOR_STRING),
            JsonValue::Array => {
                if node.expanded {
                    (":".to_string(), COLOR_BRACKET)
                } else {
                    ("[...]".to_string(), COLOR_KEY)  // Collapsed array preview
                }
            }
            JsonValue::Object => {
                if node.expanded {
                    (":".to_string(), COLOR_BRACKET)
                } else {
                    ("{...}".to_string(), COLOR_KEY)  // Collapsed object preview
                }
            }
        };

        // Build the row for this node
        let node_row: Element<'a, Message> = if node.is_expandable() {
            // Expandable node - make it clickable
            let indicator = if node.expanded { "⊟ " } else { "⊞ " };

            // Build row with colored parts inside button
            // Clone current_prefix to transfer ownership (avoids lifetime issues)
            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(current_prefix.clone()).font(Font::MONOSPACE).size(14).color(COLOR_BRACKET).into(),
                text(indicator).font(Font::MONOSPACE).size(14).color(COLOR_INDICATOR).into(),
            ];

            // Add key if present (no quotes, cleaner look)
            if let Some(k) = &node.key {
                row_elements.push(
                    text(k.clone())
                        .font(Font::MONOSPACE)
                        .size(14)
                        .color(COLOR_KEY)
                        .into()
                );
                row_elements.push(
                    text(" : ")
                        .font(Font::MONOSPACE)
                        .size(14)
                        .color(COLOR_BRACKET)
                        .into()
                );
            }

            button(row(row_elements).spacing(0))
                .on_press(Message::ToggleNode(index))
                .padding(0)  // No padding to align with leaf nodes
                .style(button::text)
                .into()
        } else {
            // Leaf node - not clickable, but still styled
            // Add same spacing as indicator "⊟ " (2 chars) to align with expandable nodes
            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(current_prefix).font(Font::MONOSPACE).size(14).color(COLOR_BRACKET).into(),
                text("  ").font(Font::MONOSPACE).size(14).into(),  // 2 chars to match "⊟ "
            ];

            // Add key if present (no quotes)
            if let Some(k) = &node.key {
                row_elements.push(
                    text(k.clone())
                        .font(Font::MONOSPACE)
                        .size(14)
                        .color(COLOR_KEY)
                        .into()
                );
                row_elements.push(
                    text(" : ")
                        .font(Font::MONOSPACE)
                        .size(14)
                        .color(COLOR_BRACKET)
                        .into()
                );
            }

            // Add value with its color
            row_elements.push(
                text(value_str)
                    .font(Font::MONOSPACE)
                    .size(14)
                    .color(value_color)
                    .into()
            );

            row(row_elements).spacing(0).into()
        };

        // Add the row directly
        elements.push(node_row);

        // Collect children if expanded
        if node.expanded {
            let children = &node.children;
            let child_count = children.len();
            for (i, &child_index) in children.iter().enumerate() {
                let is_last_child = i == child_count - 1;
                self.collect_nodes(tree, child_index, elements, &child_prefix, is_last_child, false);
            }
        }
    }
}
