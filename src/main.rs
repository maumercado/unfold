mod parser;

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length, Center, Color, Task};
use parser::{JsonTree, JsonValue};
use std::fs;
use std::path::PathBuf;

pub fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

// The application state (Model)
struct App {
    // The loaded JSON tree (None if no file loaded)
    tree: Option<JsonTree>,
    // Status message to show the user
    status: String,
    // Currently loaded file path
    current_file: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        App {
            tree: None,
            status: String::from("No file loaded"),
            current_file: None,
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
        // Header section
        let header = column![
            text("Unfold").size(28),
            text(&self.status).size(14),
        ]
        .spacing(5);

        // Open file button
        let open_button = button(text("Open File..."))
            .on_press(Message::OpenFileDialog)
            .padding(10);

        // Tree display section
        let tree_view: Element<'_, Message> = match &self.tree {
            Some(tree) => {
                // Build interactive tree view
                let mut elements: Vec<Element<'_, Message>> = Vec::new();
                self.collect_nodes(tree, tree.root_index(), &mut elements);

                let nodes_column = column(elements).spacing(2);

                scrollable(
                    container(nodes_column).padding(10)
                )
                .height(Length::Fill)
                .into()
            }
            None => {
                text("Load a file to see its structure")
                    .size(14)
                    .into()
            }
        };

        // Main layout
        let content = column![
            header,
            open_button,
            tree_view,
        ]
        .spacing(15)
        .padding(20)
        .align_x(Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // Recursively collect tree nodes into a Vec
    fn collect_nodes<'a>(
        &self,
        tree: &JsonTree,
        index: usize,
        elements: &mut Vec<Element<'a, Message>>,
    ) {
        let Some(node) = tree.get_node(index) else {
            return;
        };

        // Create indentation string (using spaces)
        let indent = "    ".repeat(node.depth);

        // Format the key part
        let key_str = match &node.key {
            Some(k) => format!("\"{}\": ", k),
            None => String::new(),
        };

        // Format the value part with color hints
        let (value_str, value_color) = match &node.value {
            JsonValue::Null => ("null".to_string(), Color::from_rgb(0.6, 0.6, 0.6)),
            JsonValue::Bool(b) => (b.to_string(), Color::from_rgb(0.8, 0.5, 0.2)),
            JsonValue::Number(n) => (n.to_string(), Color::from_rgb(0.4, 0.7, 0.4)),
            JsonValue::String(s) => (format!("\"{}\"", s), Color::from_rgb(0.6, 0.8, 0.6)),
            JsonValue::Array => (format!("[{} items]", node.children.len()), Color::from_rgb(0.7, 0.7, 0.9)),
            JsonValue::Object => (format!("{{{} fields}}", node.children.len()), Color::from_rgb(0.9, 0.7, 0.7)),
        };

        // Build the row for this node
        let node_row: Element<'a, Message> = if node.is_expandable() {
            // Expandable node - make it clickable
            let indicator = if node.expanded { "▼ " } else { "▶ " };

            button(
                text(format!("{}{}{}{}", indent, indicator, key_str, value_str))
                    .size(14)
            )
            .on_press(Message::ToggleNode(index))
            .padding(4)
            .style(button::text)
            .into()
        } else {
            // Leaf node - not clickable
            row![
                text(format!("{}   {}", indent, key_str)).size(14),
                text(value_str).size(14).color(value_color),  // Pass owned String, not reference
            ]
            .spacing(0)
            .into()
        };

        elements.push(node_row);

        // Collect children if expanded
        if node.expanded {
            for &child_index in &node.children {
                self.collect_nodes(tree, child_index, elements);
            }
        }
    }
}
