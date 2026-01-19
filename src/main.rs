//! Unfold - A high-performance JSON viewer built in Rust with Iced.

mod parser;
mod theme;
mod menu;
mod message;
mod update_check;
mod flat_row;
mod parse_error;
mod search;
mod json_export;

use iced::widget::{button, column, container, mouse_area, row, scrollable, stack, text, text_input, Space};
use iced::{Element, Font, Length, Center, Fill, Color, Size, Task, window, Border, Shadow, Subscription, clipboard, Theme, event, Event};
use iced::border::Radius;
use iced::advanced::widget::{Id as WidgetId, operate};
use iced::advanced::widget::operation::scrollable::{scroll_to, AbsoluteOffset};
use iced::advanced::widget::operation::focusable;
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::button::Status as ButtonStatus;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::env;
use std::process::Command;

// Re-export from modules
use theme::{AppTheme, ThemeColors, get_theme_colors, button_3d_style_themed, button_toggle_style_themed};
use menu::try_initialize_menu;
use message::{Message, ContextSubmenu};
use update_check::{UpdateCheckState, fetch_latest_release};
use flat_row::{FlatRow, ValueType, ROW_HEIGHT, BUFFER_ROWS};
use parse_error::ParseError;
use parser::{JsonTree, JsonValue};

/// Install the CLI tool by creating a symlink in /usr/local/bin
/// Uses osascript on macOS to prompt for admin privileges
#[cfg(target_os = "macos")]
fn install_cli_tool() -> Result<String, String> {
    let target_path = "/usr/local/bin/unfold";

    // Get the path to the current executable
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let source_path = exe_path.to_string_lossy();

    // Check if /usr/local/bin exists
    let bin_dir = PathBuf::from("/usr/local/bin");
    if !bin_dir.exists() {
        return Err("Directory /usr/local/bin does not exist. Please create it first.".to_string());
    }

    // Use osascript to run ln with admin privileges
    // This will show the native macOS password prompt
    let script = format!(
        r#"do shell script "ln -sf '{}' '{}'" with administrator privileges"#,
        source_path.replace("'", "'\\''"),
        target_path
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if output.status.success() {
        Ok("CLI installed! You can now use 'unfold' from the terminal.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User canceled") || stderr.contains("canceled") {
            Err("Installation canceled.".to_string())
        } else {
            Err(format!("Failed to create symlink: {}", stderr.trim()))
        }
    }
}

/// Install the CLI tool on Linux (non-macOS Unix)
#[cfg(all(unix, not(target_os = "macos")))]
fn install_cli_tool() -> Result<String, String> {
    use std::os::unix::fs::symlink;

    let target_path = PathBuf::from("/usr/local/bin/unfold");
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    // Try direct symlink first, suggest sudo if it fails
    if target_path.exists() || target_path.is_symlink() {
        let _ = fs::remove_file(&target_path);
    }

    match symlink(&exe_path, &target_path) {
        Ok(()) => Ok("CLI installed! You can now use 'unfold' from the terminal.".to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(format!(
                "Permission denied. Run this command in terminal:\nsudo ln -sf \"{}\" \"{}\"",
                exe_path.display(), target_path.display()
            ))
        }
        Err(e) => Err(format!("Failed to create symlink: {}", e)),
    }
}

#[cfg(not(unix))]
fn install_cli_tool() -> Result<String, String> {
    Err("CLI installation is only supported on Unix systems (macOS, Linux)".to_string())
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
        .subscription(App::subscription)
        .run()
}

/// The application state (Model)
struct App {
    tree: Option<JsonTree>,
    status: String,
    current_file: Option<PathBuf>,
    #[allow(dead_code)]
    preferences: Preferences,
    /// Current theme (dark/light)
    theme: AppTheme,
    /// Time taken to load and parse the file
    load_time: Option<Duration>,
    /// Flattened rows for virtual scrolling (rebuilt when tree changes)
    flat_rows: Vec<FlatRow>,
    /// Viewport height in pixels (updated on resize)
    viewport_height: f32,
    /// Current scroll offset in pixels (for virtual scrolling)
    scroll_offset: f32,
    // Search state
    search_query: String,
    search_results: Vec<usize>,
    search_result_index: Option<usize>,
    search_matches: HashSet<usize>,
    search_case_sensitive: bool,
    search_use_regex: bool,
    search_regex_error: Option<String>,
    /// Scrollable ID for programmatic scrolling
    tree_scrollable_id: WidgetId,
    /// Search input ID for programmatic focus
    search_input_id: WidgetId,
    /// Track current keyboard modifiers (for Shift+Enter in search input)
    current_modifiers: Modifiers,
    /// Currently selected node (for copy, path display, etc.)
    selected_node: Option<usize>,
    /// Parse error details (for better error display)
    parse_error: Option<ParseError>,
    /// Show help overlay with keyboard shortcuts
    show_help: bool,
    /// Context menu state: Some((node_index, x, y)) when visible
    context_menu_state: Option<(usize, f32, f32)>,
    /// Which submenu is currently open
    context_submenu: ContextSubmenu,
    /// Update check state for the dialog
    update_check_state: UpdateCheckState,
    /// CLI install result dialog: Some((success, message)) when showing
    cli_install_result: Option<(bool, String)>,
}

/// User-configurable display preferences (for future use)
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Preferences {
    indent_size: usize,
    show_tree_lines: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            indent_size: 2,
            show_tree_lines: true,
        }
    }
}

impl App {
    /// Initialize the application (called once at startup)
    fn boot() -> (Self, Task<Message>) {
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
            cli_install_result: None,
        };

        // Check if a file path was passed as CLI argument
        let args: Vec<String> = env::args().collect();

        if args.len() > 1 {
            let file_path = PathBuf::from(&args[1]);
            (app, Task::done(Message::FileSelected(Some(file_path))))
        } else {
            (app, Task::none())
        }
    }

    /// Subscription: Listen for keyboard and menu events
    fn subscription(&self) -> Subscription<Message> {
        try_initialize_menu();

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
                menu::try_receive_menu_event().unwrap_or(Message::NoOp)
            }),
        ])
    }

    /// Flatten the tree into a Vec<FlatRow> for virtual scrolling
    fn flatten_visible_nodes(tree: &JsonTree) -> Vec<FlatRow> {
        let mut rows = Vec::new();

        if let Some(root) = tree.get_node(tree.root_index()) {
            let root_is_array = matches!(root.value, JsonValue::Array);
            let child_count = root.children.len();
            for (i, &child_index) in root.children.iter().enumerate() {
                let is_last = i == child_count - 1;
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

        let row_index = rows.len();

        rows.push(FlatRow::new(
            index,
            current_prefix,
            node.key.as_ref().map(|k| k.to_string()),
            value_display,
            value_type,
            node.is_expandable(),
            node.expanded,
            row_index,
            current_path.to_string(),
        ));

        if node.expanded {
            let is_array = matches!(node.value, JsonValue::Array);
            let child_count = node.children.len();
            for (i, &child_index) in node.children.iter().enumerate() {
                let is_last_child = i == child_count - 1;
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
        let colors = get_theme_colors(self.theme);
        let value_color = flat_row.value_type.color(&colors);

        let node_row: Element<'a, Message> = if flat_row.is_expandable {
            let indicator = if flat_row.is_expanded { "⊟ " } else { "⊞ " };

            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(colors.bracket).into(),
                text(indicator).font(Font::MONOSPACE).size(13).color(colors.indicator).into(),
            ];

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
            let mut row_elements: Vec<Element<'a, Message>> = vec![
                text(flat_row.prefix.clone()).font(Font::MONOSPACE).size(13).color(colors.bracket).into(),
                text("─ ").font(Font::MONOSPACE).size(13).color(colors.bracket).into(),
            ];

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

            button(row(row_elements).spacing(0))
                .on_press(Message::SelectNode(flat_row.node_index))
                .padding(0)
                .style(button::text)
                .into()
        };

        let is_selected = self.selected_node == Some(flat_row.node_index);
        let is_match = self.search_matches.contains(&flat_row.node_index);
        let is_current_result = self.search_result_index
            .map(|i| self.search_results.get(i) == Some(&flat_row.node_index))
            .unwrap_or(false);

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

        let node_index = flat_row.node_index;
        let row_index = flat_row.row_index;
        let toolbar_height = 60.0;
        let y_pos = toolbar_height + (row_index as f32 * ROW_HEIGHT) - self.scroll_offset + ROW_HEIGHT;
        let estimated_depth = flat_row.prefix.len() / 4;
        let x_pos = 50.0 + (estimated_depth as f32 * 15.0);

        mouse_area(row_container)
            .on_right_press(Message::ShowContextMenu(node_index, x_pos, y_pos))
            .into()
    }

    /// Calculate the maximum display width needed for the tree
    fn calculate_max_width(&self) -> f32 {
        let Some(tree) = &self.tree else {
            return 400.0;
        };

        let max_chars = self.max_line_chars(tree, tree.root_index(), 0);

        let char_width = 8.5;
        let padding = 80.0;
        let min_width = 500.0;
        let max_width = 1400.0;

        (max_chars as f32 * char_width + padding).clamp(min_width, max_width)
    }

    fn max_line_chars(&self, tree: &JsonTree, index: usize, depth: usize) -> usize {
        let Some(node) = tree.get_node(index) else {
            return 0;
        };

        let prefix_len = depth * 3;
        let indicator_len = 2;
        let key_len = node.key.as_ref().map(|k| k.len() + 3).unwrap_or(0);
        let value_len = match &node.value {
            JsonValue::Null => 4,
            JsonValue::Bool(b) => b.to_string().len(),
            JsonValue::Number(n) => n.to_string().len(),
            JsonValue::String(s) => s.len() + 2,
            JsonValue::Array | JsonValue::Object => 1,
        };

        let this_line = prefix_len + indicator_len + key_len + value_len;

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

    /// Handle messages and update state
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFileDialog => {
                Task::perform(
                    async {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("JSON", &["json"])
                            .add_filter("All Files", &["*"])
                            .set_title("Open JSON File")
                            .pick_file()
                            .await;
                        file.map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelected,
                )
            }
            Message::FileSelected(path_option) => {
                match path_option {
                    Some(path) => {
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
                                        self.parse_error = None;

                                        self.flat_rows = Self::flatten_visible_nodes(self.tree.as_ref().unwrap());

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
                                self.parse_error = Some(parse_error::ParseError {
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
                        Task::none()
                    }
                    None => Task::none()
                }
            }
            Message::FileDropped(path) => {
                let is_json = path.extension()
                    .map(|ext| ext.to_string_lossy().to_lowercase())
                    .map(|ext| ext == "json")
                    .unwrap_or(false);

                if is_json {
                    self.update(Message::FileSelected(Some(path)))
                } else {
                    let filename = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    self.status = format!("✗ Not a JSON file: {}", filename);
                    Task::none()
                }
            }
            Message::ToggleNode(index) => {
                self.selected_node = Some(index);
                if let Some(tree) = &mut self.tree {
                    tree.toggle_expanded(index);
                    self.flat_rows = Self::flatten_visible_nodes(tree);
                }
                Task::none()
            }
            Message::Scrolled(viewport) => {
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

                    let node_index = self.search_results[new_index];
                    self.expand_to_node(node_index);

                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }

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

                    let node_index = self.search_results[new_index];
                    self.expand_to_node(node_index);

                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }

                    self.scroll_to_node(node_index)
                } else {
                    Task::none()
                }
            }
            Message::ClearSearch => {
                self.search_query.clear();
                self.search_results.clear();
                self.search_result_index = None;
                self.search_matches.clear();
                self.search_regex_error = None;
                Task::none()
            }
            Message::FocusSearch => {
                operate(focusable::focus(self.search_input_id.clone()))
            }
            Message::ModifiersChanged(modifiers) => {
                self.current_modifiers = modifiers;
                Task::none()
            }
            Message::SearchSubmit => {
                if self.current_modifiers.shift() {
                    self.update(Message::SearchPrev)
                } else {
                    self.update(Message::SearchNext)
                }
            }
            Message::KeyPressed(key, modifiers) => {
                let cmd_or_ctrl = modifiers.command() || modifiers.control();

                match key {
                    Key::Named(Named::Escape) => {
                        if self.show_help {
                            self.update(Message::ToggleHelp)
                        } else if self.context_menu_state.is_some() {
                            self.update(Message::HideContextMenu)
                        } else {
                            self.update(Message::ClearSearch)
                        }
                    }
                    Key::Named(Named::Enter) => {
                        if modifiers.shift() {
                            self.update(Message::SearchPrev)
                        } else {
                            self.update(Message::SearchNext)
                        }
                    }
                    Key::Character(c) if c.as_str() == "o" && cmd_or_ctrl => {
                        self.update(Message::OpenFileDialog)
                    }
                    Key::Character(c) if c.as_str() == "g" && cmd_or_ctrl => {
                        if modifiers.shift() {
                            self.update(Message::SearchPrev)
                        } else {
                            self.update(Message::SearchNext)
                        }
                    }
                    Key::Character(c) if c.as_str() == "f" && cmd_or_ctrl => {
                        self.update(Message::FocusSearch)
                    }
                    Key::Character(c) if c.as_str() == "n" && cmd_or_ctrl => {
                        self.update(Message::OpenFileInNewWindow)
                    }
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && !modifiers.shift() && !modifiers.alt() => {
                        self.update(Message::CopySelectedValue)
                    }
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && modifiers.shift() && !modifiers.alt() => {
                        self.update(Message::CopySelectedName)
                    }
                    Key::Character(c) if c.as_str() == "c" && cmd_or_ctrl && modifiers.alt() => {
                        self.update(Message::CopySelectedPath)
                    }
                    Key::Character(c) if c.as_str() == "t" && cmd_or_ctrl => {
                        self.update(Message::ToggleTheme)
                    }
                    Key::Character(c) if (c.as_str() == "/" || c.as_str() == "?") && cmd_or_ctrl => {
                        self.update(Message::ToggleHelp)
                    }
                    _ => Task::none()
                }
            }
            Message::OpenFileInNewWindow => {
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
                if let Some(file_path) = path_option {
                    Self::spawn_new_window(Some(file_path));
                }
                Task::none()
            }
            Message::SelectNode(node_index) => {
                if self.selected_node == Some(node_index) {
                    self.selected_node = None;
                } else {
                    self.selected_node = Some(node_index);
                }
                Task::none()
            }
            Message::CopySelectedValue => {
                self.context_menu_state = None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node)
                    && tree.get_node(node_index).is_some() {
                        let value_string = json_export::format_node_value_for_copy(tree, node_index);
                        return clipboard::write(value_string);
                    }
                Task::none()
            }
            Message::CopySelectedPath => {
                self.context_menu_state = None;
                if let Some(node_index) = self.selected_node
                    && let Some(flat_row) = self.flat_rows.iter().find(|r| r.node_index == node_index) {
                        return clipboard::write(flat_row.path.clone());
                    }
                Task::none()
            }
            Message::CopySelectedName => {
                self.context_menu_state = None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node)
                    && let Some(node) = tree.get_node(node_index)
                        && let Some(key) = &node.key {
                            return clipboard::write(key.clone());
                        }
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme = match self.theme {
                    AppTheme::Dark => AppTheme::Light,
                    AppTheme::Light => AppTheme::Dark,
                };
                Task::none()
            }
            Message::ToggleHelp => {
                self.show_help = !self.show_help;
                Task::none()
            }
            Message::NoOp => Task::none(),
            Message::CheckForUpdates => {
                self.update_check_state = UpdateCheckState::Checking;
                Task::perform(
                    async { fetch_latest_release().await },
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
                #[cfg(target_os = "macos")]
                {
                    let _ = Command::new("open").arg(&url).spawn();
                }
                #[cfg(target_os = "windows")]
                {
                    let _ = Command::new("cmd").args(["/c", "start", &url]).spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = Command::new("xdg-open").arg(&url).spawn();
                }
                self.update_check_state = UpdateCheckState::None;
                Task::none()
            }
            Message::ExportJson => {
                self.context_menu_state = None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    let json_string = json_export::node_to_json_string(tree, node_index);
                    Task::perform(
                        async move {
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("export.json")
                                .save_file()
                                .await;
                            if let Some(handle) = file_handle {
                                let _ = fs::write(handle.path(), json_string);
                            }
                        },
                        |_| Message::NoOp
                    )
                } else {
                    Task::none()
                }
            }
            Message::ExpandAllChildren => {
                self.context_menu_state = None;
                if let Some(node_index) = self.selected_node {
                    if let Some(tree) = &mut self.tree {
                        Self::set_expanded_recursive(tree, node_index, true);
                    }
                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }
                }
                Task::none()
            }
            Message::CollapseAllChildren => {
                self.context_menu_state = None;
                if let Some(node_index) = self.selected_node {
                    if let Some(tree) = &mut self.tree {
                        Self::set_expanded_recursive(tree, node_index, false);
                    }
                    if let Some(tree) = &self.tree {
                        self.flat_rows = Self::flatten_visible_nodes(tree);
                    }
                }
                Task::none()
            }
            Message::OpenInExternalEditor => {
                if let Some(path) = &self.current_file {
                    #[cfg(target_os = "macos")]
                    {
                        let _ = Command::new("open").arg("-t").arg(path).spawn();
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let _ = Command::new("cmd")
                            .args(["/C", "start", "", path.to_str().unwrap_or("")])
                            .spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = Command::new("xdg-open").arg(path).spawn();
                    }
                }
                Task::none()
            }
            Message::ShowContextMenu(node_index, x, y) => {
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
                    let minified = json_export::node_to_json_string_minified(tree, node_index);
                    return clipboard::write(minified);
                }
                Task::none()
            }
            Message::CopyValueFormatted => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    let json = json_export::node_to_json_string(tree, node_index);
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json)
                        && let Ok(formatted) = serde_json::to_string_pretty(&value) {
                            return clipboard::write(formatted);
                        }
                    return clipboard::write(json);
                }
                Task::none()
            }
            Message::ExportAsJson => {
                self.context_menu_state = None;
                self.context_submenu = ContextSubmenu::None;
                if let (Some(tree), Some(node_index)) = (&self.tree, self.selected_node) {
                    let json_string = json_export::node_to_json_string(tree, node_index);
                    Task::perform(
                        async move {
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("export.json")
                                .save_file()
                                .await;
                            if let Some(handle) = file_handle {
                                let _ = fs::write(handle.path(), json_string);
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
                    let minified = json_export::node_to_json_string_minified(tree, node_index);
                    Task::perform(
                        async move {
                            let file_handle = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("export.min.json")
                                .save_file()
                                .await;
                            if let Some(handle) = file_handle {
                                let _ = fs::write(handle.path(), minified);
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
                    let json = json_export::node_to_json_string(tree, node_index);
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
                                let _ = fs::write(handle.path(), formatted);
                            }
                        },
                        |_| Message::NoOp
                    )
                } else {
                    Task::none()
                }
            }
            Message::InstallCLI => {
                Task::perform(
                    async {
                        install_cli_tool()
                    },
                    Message::InstallCLIResult,
                )
            }
            Message::InstallCLIResult(result) => {
                match result {
                    Ok(msg) => {
                        self.status = format!("✓ {}", msg);
                        self.cli_install_result = Some((true, msg));
                    }
                    Err(msg) => {
                        self.status = format!("✗ {}", msg);
                        self.cli_install_result = Some((false, msg));
                    }
                }
                Task::none()
            }
            Message::DismissCLIDialog => {
                self.cli_install_result = None;
                Task::none()
            }
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

    /// Recursively set expanded state for a node and all its children
    fn set_expanded_recursive(tree: &mut JsonTree, node_index: usize, expanded: bool) {
        tree.set_expanded(node_index, expanded);

        if let Some(node) = tree.get_node(node_index) {
            let children = node.children.clone();
            for child_index in children {
                Self::set_expanded_recursive(tree, child_index, expanded);
            }
        }
    }

    /// Run search with current query and options
    fn run_search(&mut self) -> Task<Message> {
        if self.search_query.is_empty() {
            self.search_results.clear();
            self.search_result_index = None;
            self.search_matches.clear();
            self.search_regex_error = None;
            return Task::none();
        }

        if let Some(tree) = &self.tree {
            let (results, error) = search::search_nodes(
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
            let path = tree.get_path_to_node(target_index);
            for &node_index in &path {
                if node_index != target_index {
                    tree.set_expanded(node_index, true);
                }
            }
        }
    }

    /// Calculate the scroll offset to make a node visible
    fn scroll_to_node(&self, target_index: usize) -> Task<Message> {
        if let Some(row_pos) = self.flat_rows.iter().position(|r| r.node_index == target_index) {
            let target_offset = row_pos as f32 * ROW_HEIGHT;
            let center_offset = self.viewport_height / 2.0;
            let scroll_y = (target_offset - center_offset).max(0.0);

            let id = self.tree_scrollable_id.clone();
            let offset = AbsoluteOffset { x: Some(0.0), y: Some(scroll_y) };
            operate(scroll_to(id, offset))
        } else {
            Task::none()
        }
    }

    /// Render the UI
    fn view(&self) -> Element<'_, Message> {
        let colors = get_theme_colors(self.theme);

        let tree_view: Element<'_, Message> = match &self.tree {
            Some(_tree) => {
                let total_rows = self.flat_rows.len();
                let first_visible = (self.scroll_offset / ROW_HEIGHT).floor() as usize;
                let visible_count = (self.viewport_height / ROW_HEIGHT).ceil() as usize + 1;

                let start = first_visible.saturating_sub(BUFFER_ROWS);
                let end = (first_visible + visible_count + BUFFER_ROWS).min(total_rows);

                let mut elements: Vec<Element<'_, Message>> = Vec::new();

                let top_offset = start as f32 * ROW_HEIGHT;
                if top_offset > 0.0 {
                    elements.push(Space::new().height(Length::Fixed(top_offset)).into());
                }

                for flat_row in self.flat_rows.iter().skip(start).take(end - start) {
                    elements.push(self.render_flat_row(flat_row));
                }

                let bottom_offset = (total_rows - end) as f32 * ROW_HEIGHT;
                if bottom_offset > 0.0 {
                    elements.push(Space::new().height(Length::Fixed(bottom_offset)).into());
                }

                let nodes_column = column(elements).spacing(0);

                scrollable(container(nodes_column).padding([10, 0]))
                    .id(self.tree_scrollable_id.clone())
                    .direction(scrollable::Direction::Both {
                        vertical: scrollable::Scrollbar::default(),
                        horizontal: scrollable::Scrollbar::default(),
                    })
                    .on_scroll(Message::Scrolled)
                    .height(Length::Fill)
                    .width(Fill)
                    .into()
            }
            None => {
                if let Some(ref error) = self.parse_error {
                    self.render_error_screen(error, colors)
                } else {
                    self.render_welcome_screen(colors)
                }
            }
        };

        if self.tree.is_some() {
            let toolbar = self.render_toolbar(colors);
            let status_bar = self.render_status_bar(colors);

            let tree_container = container(tree_view)
                .width(Fill)
                .height(Fill)
                .style(move |_theme| container::Style {
                    background: Some(colors.background.into()),
                    ..Default::default()
                });

            let main_content: Element<'_, Message> = column![toolbar, tree_container, status_bar].into();

            if self.cli_install_result.is_some() {
                stack![main_content, self.render_cli_install_dialog(colors)].into()
            } else if self.update_check_state != UpdateCheckState::None {
                stack![main_content, self.render_update_dialog(colors)].into()
            } else if self.show_help {
                stack![main_content, self.render_help_overlay(colors)].into()
            } else if self.context_menu_state.is_some() {
                stack![main_content, self.render_context_menu(colors)].into()
            } else {
                main_content
            }
        } else if self.cli_install_result.is_some() {
            stack![tree_view, self.render_cli_install_dialog(colors)].into()
        } else if self.update_check_state != UpdateCheckState::None {
            stack![tree_view, self.render_update_dialog(colors)].into()
        } else if self.show_help {
            stack![tree_view, self.render_help_overlay(colors)].into()
        } else {
            tree_view
        }
    }

    /// Render the toolbar with search controls
    fn render_toolbar<'a>(&self, colors: ThemeColors) -> Element<'a, Message> {
        let case_button = button(text("Aa").size(11))
            .padding([4, 8])
            .style(button_toggle_style_themed(self.search_case_sensitive, colors))
            .on_press(Message::ToggleCaseSensitive);

        let regex_button = button(text(".*").size(11))
            .padding([4, 8])
            .style(button_toggle_style_themed(self.search_use_regex, colors))
            .on_press(Message::ToggleRegex);

        let search_input = text_input("Find...", &self.search_query)
            .id(self.search_input_id.clone())
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::SearchSubmit)
            .padding(5)
            .width(Length::Fixed(200.0));

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

        let has_results = !self.search_results.is_empty();

        let prev_button = button(text("◂ Prev").size(11))
            .padding([5, 12])
            .style(button_3d_style_themed(colors));

        let next_button = button(text("Next ▸").size(11))
            .padding([5, 12])
            .style(button_3d_style_themed(colors));

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

        let theme_icon = match self.theme {
            AppTheme::Dark => "◑",
            AppTheme::Light => "◐",
        };
        let theme_button = button(text(theme_icon).size(16))
            .padding([4, 10])
            .style(button_3d_style_themed(colors))
            .on_press(Message::ToggleTheme);

        container(
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
        })
        .into()
    }

    /// Render the status bar
    fn render_status_bar<'a>(&self, colors: ThemeColors) -> Element<'a, Message> {
        let load_time_str: String = self.load_time
            .map(|d| format!("Load: {}ms", d.as_millis()))
            .unwrap_or_default();

        let node_count: String = self.tree.as_ref()
            .map(|t| format!("Nodes: {}", t.node_count()))
            .unwrap_or_default();

        let path_display: String = if let Some(node_index) = self.selected_node {
            if let Some(flat_row) = self.flat_rows.iter().find(|r| r.node_index == node_index) {
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

        container(
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
        })
        .into()
    }

    /// Render the welcome screen
    fn render_welcome_screen<'a>(&self, colors: ThemeColors) -> Element<'a, Message> {
        let welcome_text = text("Welcome to Unfold JSON Viewer.")
            .size(15)
            .color(colors.text_secondary);

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

        let new_window_link = button(text("Open in new window").size(13))
            .on_press(Message::OpenFileInNewWindow)
            .padding(0)
            .style(|_theme, _status| button::Style {
                background: None,
                text_color: Color::from_rgb(0.4, 0.55, 0.75),
                ..Default::default()
            });

        let theme_label = match self.theme {
            AppTheme::Dark => "Switch to Light Mode",
            AppTheme::Light => "Switch to Dark Mode",
        };
        let theme_link = button(text(theme_label).size(12))
            .on_press(Message::ToggleTheme)
            .padding(0)
            .style(|_theme, _status| button::Style {
                background: None,
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..Default::default()
            });

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

    /// Render the error screen
    fn render_error_screen<'a>(&self, error: &'a ParseError, colors: ThemeColors) -> Element<'a, Message> {
        let error_icon = text("⚠").size(48).color(colors.error);

        let error_title = text(format!("Failed to parse {}", error.filename))
            .size(18)
            .color(colors.error);

        let error_message = text(&error.message)
            .size(14)
            .color(colors.text_primary);

        let location_text = match (error.line, error.column) {
            (Some(line), Some(col)) => format!("Line {}, Column {}", line, col),
            (Some(line), None) => format!("Line {}", line),
            _ => String::new(),
        };
        let location = text(location_text)
            .size(13)
            .color(colors.text_secondary);

        let context_section: Element<'_, Message> = if let Some(ref ctx_line) = error.context_line {
            let truncated = if ctx_line.len() > 80 {
                format!("{}...", &ctx_line[..80])
            } else {
                ctx_line.clone()
            };

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
    }

    /// Render the help overlay with keyboard shortcuts
    fn render_help_overlay<'a>(&self, colors: ThemeColors) -> Element<'a, Message> {
        let cmd_key = if cfg!(target_os = "macos") { "⌘" } else { "Ctrl+" };
        let shift = if cfg!(target_os = "macos") { "⇧" } else { "Shift+" };
        let opt = if cfg!(target_os = "macos") { "⌥" } else { "Alt+" };

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

    /// Render the context menu overlay
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

            mouse_area(item_button)
                .on_enter(Message::OpenSubmenu(ContextSubmenu::None))
                .into()
        };

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

            mouse_area(item_content)
                .on_enter(Message::OpenSubmenu(submenu))
                .into()
        };

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

        if has_children {
            menu_items.push(separator());
            menu_items.push(menu_item("Expand All Children", Message::ExpandAllChildren));
            menu_items.push(menu_item("Collapse All Children", Message::CollapseAllChildren));
        }

        let menu_content = column(menu_items).spacing(0).padding(4);

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

        let backdrop = mouse_area(Space::new().width(Fill).height(Fill))
            .on_press(Message::HideContextMenu);

        let clamped_x = menu_x.max(10.0);
        let clamped_y = menu_y.max(10.0);

        let submenu_y_offset = match current_submenu {
            ContextSubmenu::CopyValueAs => 2.0 * 31.0,
            ContextSubmenu::ExportValueAs => 5.0 * 31.0,
            ContextSubmenu::None => 0.0,
        };

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
                    button(text("OK").size(13).color(colors.text_primary))
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
                        button(text("Download").size(13))
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
                        button(text("Later").size(13).color(colors.text_secondary))
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
                    button(text("OK").size(13).color(colors.text_primary))
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
            UpdateCheckState::None => Space::new().into(),
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

    /// Render the CLI installation result dialog overlay
    fn render_cli_install_dialog(&self, colors: ThemeColors) -> Element<'_, Message> {
        let (success, message) = self.cli_install_result.as_ref().unwrap();

        let title = if *success {
            "CLI Installed"
        } else {
            "CLI Installation Failed"
        };

        let title_color = if *success {
            Color::from_rgb(0.2, 0.7, 0.3)
        } else {
            Color::from_rgb(0.9, 0.3, 0.3)
        };

        let content: Element<'_, Message> = column![
            text(title).size(16).color(title_color),
            Space::new().height(Length::Fixed(15.0)),
            text(message).size(13).color(colors.text_secondary),
            Space::new().height(Length::Fixed(20.0)),
            button(text("OK").size(13).color(colors.text_primary))
                .on_press(Message::DismissCLIDialog)
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
        .into();

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

        let backdrop = button(Space::new().width(Fill).height(Fill))
            .on_press(Message::DismissCLIDialog)
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
    fn test_flatten_visible_nodes() {
        let value = json!({"a": 1, "b": 2});
        let tree = build_tree(&value);

        let flat_rows = App::flatten_visible_nodes(&tree);

        assert!(!flat_rows.is_empty());
    }

    #[test]
    fn test_set_expanded_recursive() {
        let value = json!({
            "level1": {
                "level2": {
                    "level3": "value"
                }
            }
        });
        let mut tree = build_tree(&value);
        let root_index = tree.root_index();

        App::set_expanded_recursive(&mut tree, root_index, true);

        for i in 0..tree.node_count() {
            if let Some(node) = tree.get_node(i) {
                if node.is_expandable() {
                    assert!(node.expanded, "Node {} should be expanded", i);
                }
            }
        }

        App::set_expanded_recursive(&mut tree, root_index, false);

        for i in 0..tree.node_count() {
            if let Some(node) = tree.get_node(i) {
                if node.is_expandable() {
                    assert!(!node.expanded, "Node {} should be collapsed", i);
                }
            }
        }
    }
}
