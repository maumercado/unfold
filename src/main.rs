mod parser;

use iced::widget::{column, container, text};
use iced::{Element, Length, Center};

pub fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

// The application state (Model)
#[derive(Default)]
struct App {
    // We'll add state here soon
}

// Messages that can be sent to update the app
#[derive(Debug, Clone)]
enum Message {
    // We'll add messages here soon
}

impl App {
    // Handle messages and update state
    fn update(&mut self, message: Message) {
        match message {
            // Handle messages here
        }
    }

    // Render the UI
    fn view(&self) -> Element<Message> {
        let content = column![
            text("Unfold").size(32),
            text("High-performance JSON Viewer").size(16),
            text(""),
            text("Drop a JSON file here or use File > Open").size(14),
        ]
        .spacing(10)
        .padding(20)
        .align_x(Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
