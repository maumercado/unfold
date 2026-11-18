//! The window content: toolbar, JSON tree, status bar, and the welcome,
//! loading and error screens.
//!
//! In GPUI a *view* is a struct that implements `Render`. It lives in an
//! `Entity<Viewer>` owned by the app; event handlers get `&mut Viewer` back
//! through `cx.listener(...)`, change it, and call `cx.notify()` to redraw.
//! There is no `Message` enum: each action has its own small handler method.

use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::native_menu::NativeMenu;
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::{
    ActiveTheme, Disableable, Selectable, Sizable, WindowExt, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use unfold_core::json_export;
use unfold_core::parse_error::ParseError;

use crate::actions::*;
use crate::dialogs;
use crate::document::{self, Document};
use crate::row::{self, Highlight};
use crate::settings;
use crate::theme::{AppTheme, Colors};
use crate::update_check::{self, UpdateCheck};

/// Fixed height per row. Tight so the tree lines connect.
pub const ROW_HEIGHT: f32 = 16.0;
const FONT_SIZE: f32 = 13.0;

enum LoadState {
    Empty,
    Loading(PathBuf),
    Loaded(Box<Document>),
    Failed(ParseError),
}

pub struct Viewer {
    focus_handle: FocusHandle,
    search_input: Entity<InputState>,
    scroll_handle: UniformListScrollHandle,
    state: LoadState,
    /// Bumped on every `open_path`, so a slow load that finishes after a
    /// newer one was started is dropped instead of replacing it.
    load_generation: u64,
    _subscriptions: Vec<Subscription>,
}

impl Viewer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("Find..."));
        let subscription = cx.subscribe_in(&search_input, window, Self::on_search_input_event);
        Viewer {
            focus_handle: cx.focus_handle(),
            search_input,
            scroll_handle: UniformListScrollHandle::new(),
            state: LoadState::Empty,
            load_generation: 0,
            _subscriptions: vec![subscription],
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// True while the window shows the welcome screen. A file opened from
    /// Finder reuses such a window instead of opening another one.
    pub fn is_empty(&self) -> bool {
        matches!(self.state, LoadState::Empty)
    }

    fn doc(&self) -> Option<&Document> {
        match &self.state {
            LoadState::Loaded(doc) => Some(doc),
            _ => None,
        }
    }

    fn doc_mut(&mut self) -> Option<&mut Document> {
        match &mut self.state {
            LoadState::Loaded(doc) => Some(doc),
            _ => None,
        }
    }

    // ---- Loading files ----

    /// Load `path` into this window. Parsing runs on a background thread so
    /// the window stays responsive for large files.
    pub fn open_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        window.set_window_title(&format!("{} - Unfold", document::file_name(&path)));
        // Keep the search query and options when another file is opened.
        let previous_search = self.doc().map(|d| {
            let s = &d.search;
            (s.query.clone(), s.case_sensitive, s.use_regex)
        });
        self.state = LoadState::Loading(path.clone());
        self.load_generation += 1;
        let generation = self.load_generation;
        cx.notify();

        let task = cx.background_spawn(async move { document::load(path) });
        cx.spawn_in(window, async move |this, cx| {
            let mut result = task.await;
            if let (Ok(doc), Some((query, case_sensitive, use_regex))) =
                (&mut result, previous_search)
            {
                doc.search.query = query;
                doc.search.case_sensitive = case_sensitive;
                doc.search.use_regex = use_regex;
            }
            this.update_in(cx, |this, window, cx| {
                if this.load_generation == generation {
                    this.finish_load(result, window, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    fn finish_load(
        &mut self,
        result: Result<Document, ParseError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(doc) => {
                if std::env::var_os("UNFOLD_TIMING").is_some() {
                    eprintln!(
                        "loaded {} in {} ms: {} nodes, {} rows",
                        doc.path.display(),
                        doc.load_time.as_millis(),
                        doc.tree.node_count(),
                        doc.rows.len()
                    );
                }
                let width = doc
                    .widest_row
                    .map(|i| row::row_char_width(&doc.rows[i]))
                    .unwrap_or(0);
                self.state = LoadState::Loaded(Box::new(doc));
                self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                self.rerun_search(cx);
                fit_window_width(width, window);
            }
            Err(err) => {
                window.set_window_title("Unfold - JSON Viewer");
                self.state = LoadState::Failed(err);
            }
        }
        cx.notify();
    }

    pub fn prompt_and_open(
        &mut self,
        new_window: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Open".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            this.update_in(cx, |this, window, cx| {
                let mut paths = paths.into_iter();
                if !new_window && let Some(first) = paths.next() {
                    this.open_path(first, window, cx);
                }
                for path in paths {
                    crate::open_window(Some(path), cx);
                }
            })
            .ok();
        })
        .detach();
    }

    fn on_drop(&mut self, paths: &ExternalPaths, window: &mut Window, cx: &mut Context<Self>) {
        let mut paths = paths.paths().iter().cloned();
        if let Some(first) = paths.next() {
            self.open_path(first, window, cx);
        }
        for path in paths {
            crate::open_window(Some(path), cx);
        }
    }

    // ---- Tree interaction ----

    fn click_row(
        &mut self,
        node: usize,
        expandable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        if let Some(doc) = self.doc_mut() {
            if expandable {
                doc.toggle(node);
            } else {
                doc.toggle_selection(node);
            }
            cx.notify();
        }
    }

    /// Right click: select the node under the cursor, then show a native menu.
    /// Menu actions act on the selection, which is now this node.
    fn show_context_menu(
        &mut self,
        node: usize,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        let Some(doc) = self.doc_mut() else { return };
        doc.selected = Some(node);
        let has_children = doc.has_children(node);
        cx.notify();

        let mut menu = NativeMenu::new()
            .menu("Copy Key", Box::new(CopyKey))
            .menu("Copy Value", Box::new(CopyValue))
            .submenu(
                "Copy Value As",
                NativeMenu::new()
                    .menu("Minified", Box::new(CopyValueMinified))
                    .menu("Formatted", Box::new(CopyValueFormatted)),
            )
            .menu("Copy Path", Box::new(CopyPath))
            .separator()
            .submenu(
                "Export Value As",
                NativeMenu::new()
                    .menu("JSON", Box::new(ExportJson))
                    .menu("Minified JSON", Box::new(ExportMinifiedJson))
                    .menu("Formatted JSON", Box::new(ExportFormattedJson)),
            );
        if has_children {
            menu = menu
                .separator()
                .menu("Expand All Children", Box::new(ExpandAllChildren))
                .menu("Collapse All Children", Box::new(CollapseAllChildren));
        }
        menu.show(position, window, cx);
    }

    /// Center `row` in the view (used for search results).
    fn scroll_to_row(&self, row: Option<usize>) {
        if let Some(row) = row {
            self.scroll_handle
                .scroll_to_item(row, ScrollStrategy::Center);
        }
    }

    fn select_prev(&mut self, _: &SelectPrev, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selection(-1, cx);
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selection(1, cx);
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let row = self.doc_mut().and_then(|d| d.move_selection(delta));
        self.scroll_into_view(row);
        cx.notify();
    }

    /// Scroll just enough to show `row` (used for keyboard navigation).
    fn scroll_into_view(&self, row: Option<usize>) {
        if let Some(row) = row {
            self.scroll_handle
                .scroll_to_item(row, ScrollStrategy::Nearest);
        }
    }

    fn expand_selected(&mut self, _: &ExpandSelected, _: &mut Window, cx: &mut Context<Self>) {
        let row = self.doc_mut().and_then(|d| d.expand_selected(true));
        self.scroll_into_view(row);
        cx.notify();
    }

    fn collapse_selected(&mut self, _: &CollapseSelected, _: &mut Window, cx: &mut Context<Self>) {
        let row = self.doc_mut().and_then(|d| d.expand_selected(false));
        self.scroll_into_view(row);
        cx.notify();
    }

    fn expand_all(&mut self, _: &ExpandAllChildren, _: &mut Window, cx: &mut Context<Self>) {
        self.set_expanded_recursive(true, cx);
    }

    fn collapse_all(&mut self, _: &CollapseAllChildren, _: &mut Window, cx: &mut Context<Self>) {
        self.set_expanded_recursive(false, cx);
    }

    fn set_expanded_recursive(&mut self, expanded: bool, cx: &mut Context<Self>) {
        if let Some(doc) = self.doc_mut()
            && let Some(node) = doc.selected
        {
            doc.set_expanded_recursive(node, expanded);
            cx.notify();
        }
    }

    // ---- Copy and export ----

    fn copy(&self, text: Option<String>, cx: &mut Context<Self>) {
        if let Some(text) = text {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn selected(&self) -> Option<(&Document, usize)> {
        let doc = self.doc()?;
        Some((doc, doc.selected?))
    }

    fn copy_value(&mut self, _: &CopyValue, _: &mut Window, cx: &mut Context<Self>) {
        let text = self
            .selected()
            .map(|(d, n)| json_export::format_node_value_for_copy(&d.tree, n));
        self.copy(text, cx);
    }

    fn copy_key(&mut self, _: &CopyKey, _: &mut Window, cx: &mut Context<Self>) {
        let text = self
            .selected()
            .and_then(|(d, n)| d.tree.get_node(n)?.key.clone());
        self.copy(text, cx);
    }

    fn copy_path(&mut self, _: &CopyPath, _: &mut Window, cx: &mut Context<Self>) {
        let text = self.selected().and_then(|(d, n)| d.path_of(n));
        self.copy(text, cx);
    }

    fn copy_minified(&mut self, _: &CopyValueMinified, _: &mut Window, cx: &mut Context<Self>) {
        let text = self
            .selected()
            .map(|(d, n)| json_export::node_to_json_string_minified(&d.tree, n));
        self.copy(text, cx);
    }

    fn copy_formatted(&mut self, _: &CopyValueFormatted, _: &mut Window, cx: &mut Context<Self>) {
        let text = self.selected().map(|(d, n)| formatted_json(d, n));
        self.copy(text, cx);
    }

    fn export_json(&mut self, _: &ExportJson, window: &mut Window, cx: &mut Context<Self>) {
        let content = self
            .selected()
            .map(|(d, n)| json_export::node_to_json_string(&d.tree, n));
        self.save_to_file(content, "export.json", window, cx);
    }

    fn export_minified(
        &mut self,
        _: &ExportMinifiedJson,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let content = self
            .selected()
            .map(|(d, n)| json_export::node_to_json_string_minified(&d.tree, n));
        self.save_to_file(content, "export.min.json", window, cx);
    }

    fn export_formatted(
        &mut self,
        _: &ExportFormattedJson,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let content = self.selected().map(|(d, n)| formatted_json(d, n));
        self.save_to_file(content, "export.json", window, cx);
    }

    fn save_to_file(
        &mut self,
        content: Option<String>,
        suggested_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (Some(content), Some(doc)) = (content, self.doc()) else {
            return;
        };
        let dir = doc
            .path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let target = cx.prompt_for_new_path(&dir, Some(suggested_name));
        cx.spawn_in(window, async move |_, cx| {
            let Ok(Ok(Some(path))) = target.await else {
                return;
            };
            let result = cx
                .background_spawn(async move { fs::write(&path, content) })
                .await;
            if let Err(e) = result {
                cx.update(|window, cx| {
                    dialogs::message(window, cx, "Export Failed", e.to_string(), false);
                })
                .ok();
            }
        })
        .detach();
    }

    fn open_in_external_editor(
        &mut self,
        _: &OpenInExternalEditor,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        if let Some(doc) = self.doc() {
            crate::platform::open_in_text_editor(&doc.path);
        }
    }

    // ---- Search ----

    fn on_search_input_event(
        &mut self,
        input: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Enter is not handled here: the input lets the keystroke continue to
        // our own `enter` / `shift-enter` bindings (SearchNext / SearchPrev),
        // so handling `PressEnter` too would step twice.
        if let InputEvent::Change = event {
            let query = input.read(cx).value().to_string();
            if let Some(doc) = self.doc_mut() {
                doc.search.query = query;
            }
            self.rerun_search(cx);
        }
    }

    fn rerun_search(&mut self, cx: &mut Context<Self>) {
        let row = self.doc_mut().and_then(|d| d.run_search());
        self.scroll_to_row(row);
        cx.notify();
    }

    fn step_search(&mut self, forward: bool, cx: &mut Context<Self>) {
        let row = self.doc_mut().and_then(|d| d.step_result(forward));
        self.scroll_to_row(row);
        cx.notify();
    }

    fn search_next(&mut self, _: &SearchNext, _: &mut Window, cx: &mut Context<Self>) {
        self.step_search(true, cx);
    }

    fn search_prev(&mut self, _: &SearchPrev, _: &mut Window, cx: &mut Context<Self>) {
        self.step_search(false, cx);
    }

    fn clear_search(&mut self, _: &ClearSearch, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(doc) = self.doc_mut() {
            doc.clear_search();
        }
        self.search_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        cx.notify();
    }

    fn focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.search_input
            .update(cx, |input, cx| input.focus(window, cx));
    }

    /// Cmd+V while the tree is focused appends the clipboard to the search,
    /// so a copied value can be searched quickly. Inside the search box the
    /// input's own paste runs.
    fn paste_into_search(
        &mut self,
        _: &PasteIntoSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        if text.is_empty() || self.doc().is_none() {
            return;
        }
        let query = format!("{}{}", self.search_input.read(cx).value(), text);
        self.search_input.update(cx, |input, cx| {
            input.set_value(query.clone(), window, cx);
            input.focus(window, cx);
        });
        if let Some(doc) = self.doc_mut() {
            doc.search.query = query;
        }
        self.rerun_search(cx);
    }

    fn toggle_case(&mut self, _: &ToggleCaseSensitive, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(doc) = self.doc_mut() {
            doc.search.case_sensitive = !doc.search.case_sensitive;
        }
        self.rerun_search(cx);
    }

    fn toggle_regex(&mut self, _: &ToggleRegex, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(doc) = self.doc_mut() {
            doc.search.use_regex = !doc.search.use_regex;
        }
        self.rerun_search(cx);
    }

    // ---- App-level commands that need a window ----

    fn open_file(&mut self, _: &OpenFile, window: &mut Window, cx: &mut Context<Self>) {
        self.prompt_and_open(false, window, cx);
    }

    fn open_file_in_new_window(
        &mut self,
        _: &OpenFileInNewWindow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.prompt_and_open(true, window, cx);
    }

    fn close_window(&mut self, _: &CloseWindow, window: &mut Window, _: &mut Context<Self>) {
        window.remove_window();
    }

    fn show_shortcuts(&mut self, _: &ShowShortcuts, window: &mut Window, cx: &mut Context<Self>) {
        dialogs::toggle_shortcuts(window, cx);
    }

    fn about(&mut self, _: &About, window: &mut Window, cx: &mut Context<Self>) {
        dialogs::about(window, cx);
    }

    fn check_for_updates(
        &mut self,
        _: &CheckForUpdates,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        dialogs::update_check(&UpdateCheck::Checking, window, cx);
        let task = cx.background_spawn(async { update_check::fetch_latest_release() });
        cx.spawn_in(window, async move |_, cx| {
            let result = task.await;
            cx.update(|window, cx| {
                window.close_all_dialogs(cx);
                dialogs::update_check(&result, window, cx);
            })
            .ok();
        })
        .detach();
    }

    fn install_cli(&mut self, _: &InstallCli, window: &mut Window, cx: &mut Context<Self>) {
        let task = cx.background_spawn(async { crate::platform::install_cli_tool() });
        cx.spawn_in(window, async move |_, cx| {
            let result = task.await;
            cx.update(|window, cx| {
                if result.is_ok() {
                    settings::update(cx, |config| config.cli_installed = true);
                }
                let (title, ok, msg) = match result {
                    Ok(msg) => ("CLI Installed", true, msg),
                    Err(msg) => ("CLI Installation Failed", false, msg),
                };
                dialogs::message(window, cx, title, msg, ok);
            })
            .ok();
        })
        .detach();
    }

    fn toggle_theme(&mut self, _: &ToggleTheme, window: &mut Window, cx: &mut Context<Self>) {
        settings::toggle_theme(Some(window), cx);
    }

    /// Clicking empty space must not leave the window with nothing focused,
    /// or keyboard shortcuts would have no view to reach.
    fn ensure_focus(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.contains_focused(window, cx) {
            window.focus(&self.focus_handle, cx);
        }
    }
}

fn formatted_json(doc: &Document, node: usize) -> String {
    let json = json_export::node_to_json_string(&doc.tree, node);
    serde_json::from_str::<serde_json::Value>(&json)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or(json)
}

/// Widen the window to fit the widest visible row.
fn fit_window_width(max_chars: usize, window: &mut Window) {
    let char_width = 8.5;
    let width = (max_chars as f32 * char_width + 80.0).clamp(500.0, 1400.0);
    let height = window.bounds().size.height;
    window.resize(size(px(width), height));
}

// ---- Rendering ----

impl Render for Viewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::for_theme(settings::theme(cx));

        // Keep keyboard shortcuts working: if focus was lost (a closed dialog,
        // a click on a native menu), give it back to the viewer.
        if window.focused(cx).is_none() {
            window.focus(&self.focus_handle, cx);
        }

        let body = match &self.state {
            LoadState::Empty => self.render_welcome(colors, cx).into_any_element(),
            LoadState::Loading(path) => centered(colors)
                .text_color(colors.text_secondary)
                .child(format!("Loading {}…", document::file_name(path)))
                .into_any_element(),
            LoadState::Failed(err) => render_error(err, colors, cx).into_any_element(),
            LoadState::Loaded(_) => v_flex()
                .size_full()
                .child(self.render_toolbar(colors, window, cx))
                .child(self.render_tree(colors, cx))
                .child(self.render_status_bar(colors))
                .into_any_element(),
        };

        div()
            .id("viewer")
            .key_context(VIEWER_CONTEXT)
            .test_support()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.background)
            .text_color(colors.text_primary)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::ensure_focus))
            .on_drop(cx.listener(Self::on_drop))
            .on_action(cx.listener(Self::open_file))
            .on_action(cx.listener(Self::open_file_in_new_window))
            .on_action(cx.listener(Self::open_in_external_editor))
            .on_action(cx.listener(Self::close_window))
            .on_action(cx.listener(Self::copy_value))
            .on_action(cx.listener(Self::copy_key))
            .on_action(cx.listener(Self::copy_path))
            .on_action(cx.listener(Self::copy_minified))
            .on_action(cx.listener(Self::copy_formatted))
            .on_action(cx.listener(Self::paste_into_search))
            .on_action(cx.listener(Self::export_json))
            .on_action(cx.listener(Self::export_minified))
            .on_action(cx.listener(Self::export_formatted))
            .on_action(cx.listener(Self::expand_all))
            .on_action(cx.listener(Self::collapse_all))
            .on_action(cx.listener(Self::focus_search))
            .on_action(cx.listener(Self::search_next))
            .on_action(cx.listener(Self::search_prev))
            .on_action(cx.listener(Self::clear_search))
            .on_action(cx.listener(Self::toggle_case))
            .on_action(cx.listener(Self::toggle_regex))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::expand_selected))
            .on_action(cx.listener(Self::collapse_selected))
            .on_action(cx.listener(Self::toggle_theme))
            .on_action(cx.listener(Self::show_shortcuts))
            .on_action(cx.listener(Self::about))
            .on_action(cx.listener(Self::check_for_updates))
            .on_action(cx.listener(Self::install_cli))
            .child(body)
    }
}

impl Viewer {
    fn render_toolbar(
        &self,
        colors: Colors,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let search = &self.doc().expect("toolbar needs a document").search;
        let has_results = !search.results.is_empty();
        let theme_icon = match settings::theme(cx) {
            AppTheme::Dark => "◑",
            AppTheme::Light => "◐",
        };

        h_flex()
            .w_full()
            .flex_none()
            .px_2p5()
            .py_2()
            .gap_1()
            .items_center()
            .bg(colors.toolbar_bg)
            .child(
                Button::new("case")
                    .label("Aa")
                    .small()
                    .selected(search.case_sensitive)
                    .tooltip("Case sensitive")
                    .on_click(
                        cx.listener(|this, _, w, cx| this.toggle_case(&ToggleCaseSensitive, w, cx)),
                    ),
            )
            .child(
                Button::new("regex")
                    .label(".*")
                    .small()
                    .selected(search.use_regex)
                    .tooltip("Regular expression")
                    .on_click(cx.listener(|this, _, w, cx| this.toggle_regex(&ToggleRegex, w, cx))),
            )
            .child(
                div()
                    .w(px(200.))
                    .ml_1()
                    .child(Input::new(&self.search_input).id("search").small()),
            )
            .child(
                Button::new("prev")
                    .label("◂ Prev")
                    .small()
                    .ml_1()
                    .disabled(!has_results)
                    .on_click(cx.listener(|this, _, _, cx| this.step_search(false, cx))),
            )
            .child(
                Button::new("next")
                    .label("Next ▸")
                    .small()
                    .disabled(!has_results)
                    .on_click(cx.listener(|this, _, _, cx| this.step_search(true, cx))),
            )
            .child(
                div()
                    .ml_2()
                    .text_xs()
                    .text_color(colors.text_secondary)
                    .child(search.summary()),
            )
            .child(div().flex_1())
            .child(
                Button::new("theme")
                    .label(theme_icon)
                    .small()
                    .ghost()
                    .tooltip("Toggle theme")
                    .on_click(cx.listener(|this, _, w, cx| this.toggle_theme(&ToggleTheme, w, cx))),
            )
    }

    fn render_tree(&self, colors: Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let doc = self.doc().expect("tree needs a document");
        let mono = cx.theme().mono_font_family.clone();

        let list = uniform_list(
            "json-tree",
            doc.rows.len(),
            cx.processor(move |this, range: Range<usize>, _window, cx| {
                this.render_rows(range, colors, cx)
            }),
        )
        .track_scroll(&self.scroll_handle)
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
        .with_width_from_item(doc.widest_row)
        .font_family(mono)
        .text_size(px(FONT_SIZE))
        .py_2p5()
        .size_full();

        div()
            .relative()
            .flex_1()
            .min_h_0()
            .w_full()
            .child(list)
            .child(Scrollbar::new(&self.scroll_handle))
    }

    fn render_rows(
        &self,
        range: Range<usize>,
        colors: Colors,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let Some(doc) = self.doc() else {
            return Vec::new();
        };
        let search = &doc.search;
        let current = search.current_node();
        let highlight = Highlight {
            query: &search.query,
            case_sensitive: search.case_sensitive,
            use_regex: search.use_regex,
        };

        range
            .filter_map(|ix| doc.rows.get(ix).map(|r| (ix, r)))
            .map(|(ix, flat_row)| {
                let node = flat_row.node_index;
                let is_match = search.matches.contains(&node);
                let is_current = current == Some(node);
                let text =
                    row::build_row_text(flat_row, (is_match || is_current).then_some(&highlight));

                let bg = if is_current {
                    Some(colors.search_current)
                } else if doc.selected == Some(node) {
                    Some(colors.selected)
                } else if is_match {
                    Some(colors.search_match)
                } else if ix % 2 == 1 {
                    Some(colors.row_odd)
                } else {
                    None
                };

                let highlights = text.spans.into_iter().map(|(range, span)| {
                    (
                        range,
                        HighlightStyle {
                            color: Some(colors.span(span)),
                            ..Default::default()
                        },
                    )
                });
                let expandable = flat_row.is_expandable;

                div()
                    .id(("row", ix))
                    .test_support()
                    .h(px(ROW_HEIGHT))
                    .min_w_full()
                    .pl_1()
                    .pr_4()
                    .flex()
                    .items_center()
                    .whitespace_nowrap()
                    .when_some(bg, |d, bg| d.bg(bg))
                    .cursor_pointer()
                    .child(StyledText::new(text.text).with_highlights(highlights))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.click_row(node, expandable, window, cx)
                    }))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                            this.show_context_menu(node, ev.position, window, cx)
                        }),
                    )
                    .into_any_element()
            })
            .collect()
    }

    fn render_status_bar(&self, colors: Colors) -> impl IntoElement {
        let doc = self.doc().expect("status bar needs a document");
        h_flex()
            .w_full()
            .flex_none()
            .px_2p5()
            .py_1()
            .text_xs()
            .bg(colors.status_bar_bg)
            .text_color(colors.text_secondary)
            .child(format!("Nodes: {}", doc.tree.node_count()))
            .child("  |  ")
            .child(
                div()
                    .text_color(colors.key)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(doc.selection_summary()),
            )
            .child(div().flex_1())
            .child(format!("Load: {}ms", doc.load_time.as_millis()))
    }

    fn render_welcome(&self, colors: Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let (cmd, opt) = if cfg!(target_os = "macos") {
            ("Cmd", "Option")
        } else {
            ("Ctrl", "Alt")
        };
        let theme_label = match settings::theme(cx) {
            AppTheme::Dark => "Switch to Light Mode",
            AppTheme::Light => "Switch to Dark Mode",
        };

        let link = |id: &'static str, label: &'static str, color: Hsla| {
            div()
                .id(id)
                .text_color(color)
                .cursor_pointer()
                .hover(|s| s.underline())
                .child(label)
        };

        centered(colors)
            .gap_2()
            .text_color(colors.text_secondary)
            .child(
                div()
                    .text_size(px(15.))
                    .child("Welcome to Unfold JSON Viewer."),
            )
            .child(
                h_flex()
                    .text_size(px(15.))
                    .child(
                        link("open", "Open", colors.link).on_click(
                            cx.listener(|this, _, w, cx| this.prompt_and_open(false, w, cx)),
                        ),
                    )
                    .child(" or drag and drop .json file here."),
            )
            .child(
                link("open-new", "Open in new window", colors.link)
                    .text_sm()
                    .on_click(cx.listener(|this, _, w, cx| this.prompt_and_open(true, w, cx))),
            )
            .child(
                link("theme", theme_label, colors.text_secondary)
                    .mt_5()
                    .text_xs()
                    .on_click(cx.listener(|this, _, w, cx| this.toggle_theme(&ToggleTheme, w, cx))),
            )
            .child(div().mt_8().text_sm().child("Keyboard Shortcuts"))
            .child(
                v_flex()
                    .items_center()
                    .gap_1()
                    .text_xs()
                    .child(format!("{cmd}+O  Open file"))
                    .child(format!("{cmd}+C  Copy value"))
                    .child(format!("{cmd}+Shift+C  Copy key"))
                    .child(format!("{cmd}+{opt}+C  Copy path"))
                    .child(format!("{cmd}+/  All shortcuts")),
            )
    }
}

fn centered(colors: Colors) -> Div {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .bg(colors.background)
}

fn render_error(err: &ParseError, colors: Colors, cx: &mut Context<Viewer>) -> impl IntoElement {
    let mono = cx.theme().mono_font_family.clone();
    let location = match (err.line, err.column) {
        (Some(line), Some(col)) => format!("Line {}, Column {}", line, col),
        (Some(line), None) => format!("Line {}", line),
        _ => String::new(),
    };
    let context = err.context_line.as_ref().map(|line| {
        let truncated = if line.chars().count() > 80 {
            format!("{}...", line.chars().take(80).collect::<String>())
        } else {
            line.clone()
        };
        let caret = err
            .column
            .map(|c| format!("{}^", " ".repeat(c.saturating_sub(1))))
            .unwrap_or_default();
        v_flex()
            .mt_4()
            .font_family(mono)
            .text_xs()
            .child(div().text_color(colors.error_context).child(truncated))
            .child(div().text_color(colors.error).child(caret))
    });

    centered(colors)
        .gap_1()
        .child(div().text_size(px(48.)).text_color(colors.error).child("⚠"))
        .child(
            div()
                .text_size(px(18.))
                .text_color(colors.error)
                .child(format!("Failed to parse {}", err.filename)),
        )
        .child(div().mt_2().text_sm().child(err.message.clone()))
        .child(
            div()
                .text_sm()
                .text_color(colors.text_secondary)
                .child(location),
        )
        .children(context)
        .child(
            Button::new("try-again")
                .label("Try Another File...")
                .mt_5()
                .on_click(cx.listener(|this, _, w, cx| this.prompt_and_open(false, w, cx))),
        )
}

#[cfg(test)]
mod ui_tests {
    //! Headless UI tests: a real window, real key bindings and real focus,
    //! driven through gpui-kit's test harness.

    use super::Viewer;
    use crate::document::Document;
    use gpui_kit::test::TestWindowExt as _;
    use gpui_kit::{
        AppContext as _, Bounds, ClipboardItem, Entity, TestAppContext, WindowBounds, WindowHandle,
        WindowOptions, base::Root, point, px, size,
    };
    use serde_json::json;
    use std::time::Duration;
    use unfold_core::config::Config;
    use unfold_core::parser::build_tree;

    fn setup(
        cx: &mut TestAppContext,
        value: serde_json::Value,
    ) -> (WindowHandle<Root>, Entity<Viewer>) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::settings::init_with(Config::default(), false, cx);
            crate::actions::bind_keys(cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: size(px(900.), px(600.)),
                })),
                ..Default::default()
            };
            let (window, viewer) = gpui_kit::open_window(options, cx, |window, cx| {
                let viewer = cx.new(|cx| Viewer::new(window, cx));
                let doc = Document::new("t.json".into(), build_tree(&value), Duration::ZERO);
                viewer.update(cx, |v, cx| {
                    window.focus(v.focus_handle(), cx);
                    v.finish_load(Ok(doc), window, cx);
                });
                viewer
            })
            .unwrap();
            (window.downcast::<Root>().unwrap(), viewer)
        })
    }

    fn clipboard(cx: &mut TestAppContext) -> Option<String> {
        cx.update(|cx| cx.read_from_clipboard().and_then(|c| c.text()))
    }

    fn row_count(viewer: &Entity<Viewer>, cx: &mut TestAppContext) -> usize {
        cx.update(|cx| viewer.read(cx).doc().map(|d| d.rows.len()).unwrap_or(0))
    }

    #[gpui_kit::test]
    fn clicking_a_container_row_expands_it(cx: &mut TestAppContext) {
        let (window, viewer) = setup(cx, json!({"a": {"x": 1, "y": 2}, "b": 3}));
        assert_eq!(row_count(&viewer, cx), 2);
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click(("row", 0usize), cx);
        })
        .unwrap();
        assert_eq!(row_count(&viewer, cx), 4);
    }

    #[gpui_kit::test]
    fn cmd_c_copies_the_selected_value_when_the_tree_has_focus(cx: &mut TestAppContext) {
        let (window, _) = setup(cx, json!({"name": "Ada", "n": 1}));
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click(("row", 0usize), cx);
            window.press("secondary-c", cx);
        })
        .unwrap();
        assert_eq!(clipboard(cx).as_deref(), Some("Ada"));
    }

    /// The bug class fixed in PR 19 and PR 20: with the search box focused,
    /// Cmd+C and Cmd+V must edit the search text, not copy the node.
    #[gpui_kit::test]
    fn search_box_keeps_its_own_copy_and_paste(cx: &mut TestAppContext) {
        let (window, viewer) = setup(cx, json!({"alpha": "first", "beta": "second"}));
        cx.update(|cx| cx.write_to_clipboard(ClipboardItem::new_string("bet".into())));
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click(("row", 0usize), cx); // select a node first
            window.click("search", cx);
            window.press("secondary-v", cx);
            window.press("secondary-a", cx);
            window.press("secondary-c", cx);
        })
        .unwrap();
        assert_eq!(clipboard(cx).as_deref(), Some("bet"));
        let query = cx.update(|cx| viewer.read(cx).doc().unwrap().search.query.clone());
        assert_eq!(query, "bet");
    }

    #[gpui_kit::test]
    fn typing_searches_enter_steps_and_escape_clears(cx: &mut TestAppContext) {
        let (window, viewer) = setup(cx, json!({"a": {"key1": 1}, "key2": 2, "key3": 3}));
        let current =
            |cx: &mut TestAppContext| cx.update(|cx| viewer.read(cx).doc().unwrap().search.current);
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click("search", cx);
            window.input("key", cx);
        })
        .unwrap();
        assert_eq!(current(cx), Some(0));
        // The nested match was revealed by expanding its parent.
        assert_eq!(row_count(&viewer, cx), 4);

        cx.update_window(window.into(), |_, window, cx| window.press("enter", cx))
            .unwrap();
        assert_eq!(current(cx), Some(1));
        cx.update_window(window.into(), |_, window, cx| {
            window.press("shift-enter", cx)
        })
        .unwrap();
        assert_eq!(current(cx), Some(0));

        cx.update_window(window.into(), |_, window, cx| window.press("escape", cx))
            .unwrap();
        assert_eq!(current(cx), None);
        let query = cx.update(|cx| viewer.read(cx).search_input.read(cx).value().to_string());
        assert_eq!(query, "");
    }

    #[gpui_kit::test]
    fn arrow_keys_move_and_expand(cx: &mut TestAppContext) {
        let (window, viewer) = setup(cx, json!({"a": {"x": 1}, "b": 2}));
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.press("down", cx);
            window.press("right", cx);
            window.press("down", cx);
        })
        .unwrap();
        let selected_key = cx.update(|cx| {
            let doc = viewer.read(cx).doc().unwrap();
            let node = doc.selected.unwrap();
            doc.tree.get_node(node).unwrap().key.clone()
        });
        assert_eq!(selected_key.as_deref(), Some("x"));
    }

    /// Context menu actions act on the row under the cursor, even when a
    /// different row was selected before (the PR 20 targeting bug).
    #[gpui_kit::test]
    fn right_click_targets_the_row_under_the_cursor(cx: &mut TestAppContext) {
        let (window, viewer) = setup(cx, json!({"a": 1, "b": 2}));
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click(("row", 0usize), cx);
            window.right_click(("row", 1usize), cx);
            window.dispatch_action(Box::new(crate::actions::CopyKey), cx);
        })
        .unwrap();
        assert_eq!(clipboard(cx).as_deref(), Some("b"));
        let _ = viewer;
    }

    /// Frame cost with a million visible rows. Ignored by default; run with
    /// `cargo test --release -- --ignored --nocapture frame_time`.
    #[gpui_kit::test]
    #[ignore]
    fn frame_time_with_a_million_rows(cx: &mut TestAppContext) {
        use gpui_kit::{ScrollDelta, ScrollStrategy};
        let value = serde_json::Value::Array((0..1_000_000).map(|i| json!(i)).collect());
        let (window, viewer) = setup(cx, value);
        assert_eq!(row_count(&viewer, cx), 1_000_000);
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            let mut worst = Duration::ZERO;
            let start = std::time::Instant::now();
            for i in 0..200 {
                let t = std::time::Instant::now();
                if i % 20 == 0 {
                    // jump far, like dragging the scrollbar
                    viewer
                        .read(cx)
                        .scroll_handle
                        .scroll_to_item(i * 5_000, ScrollStrategy::Top);
                } else {
                    window.scroll(
                        "viewer",
                        ScrollDelta::Pixels(gpui_kit::point(px(0.), px(-300.))),
                        cx,
                    );
                }
                window.render_frame(cx);
                worst = worst.max(t.elapsed());
            }
            eprintln!(
                "200 frames: avg {:?}, worst {:?}",
                start.elapsed() / 200,
                worst
            );
        })
        .unwrap();
    }

    /// Cmd+O / File > Open in a window must use that window, even when
    /// nothing inside it has keyboard focus (e.g. right after launch or
    /// after a dialog closed). It used to fall through to the app-level
    /// handler, which always opened a second window.
    #[gpui_kit::test]
    fn open_file_without_focus_reuses_the_window(cx: &mut TestAppContext) {
        let (window, _) = setup(cx, json!({"a": 1}));
        cx.update(crate::register_app_actions);
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.blur(cx);
            window.dispatch_action(Box::new(crate::actions::OpenFile), cx);
        })
        .unwrap();
        assert_eq!(cx.update(|cx| cx.windows().len()), 1);
    }
}
