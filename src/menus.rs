//! The native menu bar, built from GPUI actions.
//!
//! Menu items carry the same actions as the key bindings, so shortcuts shown
//! in the menu come from `actions::bind_keys` and selecting an item runs the
//! same handler as pressing its keys. GPUI draws this as the macOS menu bar; on Windows and Linux it is
//! not shown, and all commands stay reachable by keyboard and context menu.

use gpui_kit::{App, Menu, MenuItem, SystemMenuType};

use crate::actions::*;

pub fn install(cx: &mut App) {
    cx.set_menus([
        Menu::new("Unfold").items([
            MenuItem::action("About Unfold", About),
            MenuItem::separator(),
            MenuItem::action("Check for Updates...", CheckForUpdates),
            MenuItem::action("Install Command Line Tool...", InstallCli),
            MenuItem::separator(),
            MenuItem::os_submenu("Services", SystemMenuType::Services),
            MenuItem::separator(),
            MenuItem::action("Hide Unfold", Hide),
            MenuItem::action("Hide Others", HideOthers),
            MenuItem::action("Show All", ShowAll),
            MenuItem::separator(),
            MenuItem::action("Quit Unfold", Quit),
        ]),
        Menu::new("File").items([
            MenuItem::action("Open...", OpenFile),
            MenuItem::action("New Window", NewWindow),
            MenuItem::action("Open in New Window...", OpenFileInNewWindow),
            MenuItem::separator(),
            MenuItem::action("Open in External Editor", OpenInExternalEditor),
            MenuItem::separator(),
            MenuItem::action("Close Window", CloseWindow),
        ]),
        Menu::new("Edit").items([
            MenuItem::action("Copy Value", CopyValue),
            MenuItem::action("Copy Key", CopyKey),
            MenuItem::action("Copy Path", CopyPath),
            MenuItem::separator(),
            MenuItem::action("Find", FocusSearch),
            MenuItem::action("Find Next", SearchNext),
            MenuItem::action("Find Previous", SearchPrev),
        ]),
        Menu::new("View").items([MenuItem::action("Toggle Theme", ToggleTheme)]),
        Menu::new("Window").items([
            MenuItem::action("Minimize", Minimize),
            MenuItem::action("Zoom", Zoom),
        ]),
        Menu::new("Help").items([
            MenuItem::action("Keyboard Shortcuts", ShowShortcuts),
            MenuItem::separator(),
            MenuItem::action("Check for Updates...", CheckForUpdates),
            MenuItem::action("Install Command Line Tool...", InstallCli),
        ]),
    ]);
}
