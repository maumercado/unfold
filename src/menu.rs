//! Native menu bar and context menu support using muda.
//!
//! Provides cross-platform native menus for macOS, Windows, and Linux.

use muda::{
    Menu, Submenu, MenuItem, PredefinedMenuItem, MenuEvent,
    accelerator::{Accelerator, Modifiers as MudaModifiers, Code},
    AboutMetadata,
};

use crate::message::Message;

/// Menu item identifiers for handling events
pub mod menu_ids {
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

// Global menu storage - must persist for app lifetime
// Using thread_local since Menu is not Send/Sync
thread_local! {
    static APP_MENU: std::cell::RefCell<Option<Menu>> = const { std::cell::RefCell::new(None) };
    static CONTEXT_MENU: std::cell::RefCell<Option<Menu>> = const { std::cell::RefCell::new(None) };
    static MENU_INIT_COUNTER: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Create the native application menu bar
pub fn create_app_menu() -> Menu {
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
pub fn create_context_menu() -> Menu {
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

/// Initialize the native menu bar (called after a delay to ensure NSApp exists)
/// Returns true if menu was just initialized
pub fn try_initialize_menu() -> bool {
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

/// Convert a menu event to a Message
pub fn menu_event_to_message(event: &muda::MenuEvent) -> Message {
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
}

/// Try to receive a menu event (non-blocking)
pub fn try_receive_menu_event() -> Option<Message> {
    MenuEvent::receiver().try_recv().ok().map(|event| menu_event_to_message(&event))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
