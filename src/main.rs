mod parser;

use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length, Center};
use parser::JsonTree;
use std::fs;

pub fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

// The application state (Model)
struct App {
    // The loaded JSON tree (None if no file loaded)
    tree: Option<JsonTree>,
    // Status message to show the user
    status: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            tree: None,
            status: String::from("No file loaded"),
        }
    }
}

// Messages that can be sent to update the app
#[derive(Debug, Clone)]
enum Message {
    LoadSampleFile,
}

impl App {
    // Handle messages and update state
    fn update(&mut self, message: Message) {
        match message {
            Message::LoadSampleFile => {
                // Try to load sample.json
                match fs::read_to_string("sample.json") {
                    Ok(contents) => {
                        match serde_json::from_str::<serde_json::Value>(&contents) {
                            Ok(json_value) => {
                                let tree = parser::build_tree(&json_value);
                                self.status = format!("Loaded {} nodes", tree.node_count());
                                self.tree = Some(tree);
                            }
                            Err(e) => {
                                self.status = format!("Parse error: {}", e);
                                self.tree = None;
                            }
                        }
                    }
                    Err(e) => {
                        self.status = format!("File error: {}", e);
                        self.tree = None;
                    }
                }
            }
        }
    }

    // Render the UI
    fn view(&self) -> Element<Message> {
        // Header section
        let header = column![
            text("Unfold").size(28),
            text(&self.status).size(14),
        ]
        .spacing(5);

        // Load button
        let load_button = button(text("Load sample.json"))
            .on_press(Message::LoadSampleFile)
            .padding(10);

        // Tree display section
        let tree_view: Element<Message> = match &self.tree {
            Some(tree) => {
                // Display the tree as text for now
                let tree_text = tree.print_tree();
                scrollable(
                    container(
                        text(tree_text)
                            .size(14)
                    )
                    .padding(10)
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
            load_button,
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
}
