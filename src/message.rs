//! Application messages for the Elm-style update loop.
//!
//! Each variant represents an event that can update the application state.

use iced::Point;
use iced::event::Status as EventStatus;
use iced::keyboard::{Key, Modifiers};
use iced::widget::scrollable::Viewport;
use std::path::PathBuf;

use crate::update_check::UpdateCheckState;

/// Messages that can be sent to update the app
/// In Rust, enums can carry data - this is called "algebraic data types"
/// Each variant can have different associated data
#[derive(Debug, Clone)]
pub enum Message {
    OpenFileDialog,
    FileSelected(Option<PathBuf>),
    FileDropped(PathBuf),
    ToggleNode(usize),
    Scrolled(Viewport),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    ToggleCaseSensitive,
    ToggleRegex,
    /// Keyboard events - Key, Modifiers, and whether widget captured it
    KeyPressed(Key, Modifiers, EventStatus),
    ModifiersChanged(Modifiers),
    ClearSearch,
    FocusSearch,
    /// Search submit from text input (checks current_modifiers for Shift)
    SearchSubmit,
    /// Open file dialog, then open selected file in new window
    OpenFileInNewWindow,
    /// Open an empty window (Cmd/Ctrl+N)
    OpenEmptyWindow,
    /// File was selected for opening in new window
    FileSelectedForNewWindow(Option<PathBuf>),
    /// Window position changed/opened
    WindowPositionUpdated(Point),
    /// Select a node (for copy, path display)
    SelectNode(usize),
    /// Copy selected node's value to clipboard
    CopySelectedValue,
    /// Copy selected node's key/name to clipboard
    CopySelectedName,
    /// Copy selected node's path to clipboard
    CopySelectedPath,
    /// Toggle between dark and light theme
    ToggleTheme,
    /// Show help overlay with shortcuts
    ToggleHelp,
    /// No operation (used for menu event polling when no event)
    NoOp,
    /// Check for updates from GitHub
    CheckForUpdates,
    /// Result of update check
    UpdateCheckResult(UpdateCheckState),
    /// Dismiss update dialog
    DismissUpdateDialog,
    /// Open release URL in browser
    OpenReleaseUrl(String),
    /// Export selected node to JSON file
    ExportJson,
    /// Expand all children of selected node
    ExpandAllChildren,
    /// Collapse all children of selected node
    CollapseAllChildren,
    /// Open current file in external editor
    OpenInExternalEditor,
    /// Show context menu at position (node_index, x, y)
    ShowContextMenu(usize, f32, f32),
    /// Hide context menu
    HideContextMenu,
    /// Open submenu
    OpenSubmenu(ContextSubmenu),
    /// Copy value in specific format
    CopyValueMinified,
    CopyValueFormatted,
    /// Export value in specific format
    ExportAsJson,
    ExportAsMinifiedJson,
    ExportAsFormattedJson,
    /// Install CLI command to /usr/local/bin
    InstallCLI,
    /// Result of CLI installation attempt
    InstallCLIResult(Result<String, String>),
    /// Dismiss CLI install dialog
    DismissCLIDialog,
    /// Paste clipboard text into search query
    PasteSearchFromClipboard,
    /// Clipboard text was read for search paste
    SearchClipboardPasted(Option<String>),
}

/// Which submenu is currently open in context menu
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextSubmenu {
    None,
    CopyValueAs,
    ExportValueAs,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_submenu_enum() {
        // Test that ContextSubmenu values are distinct and comparable
        assert_eq!(ContextSubmenu::None, ContextSubmenu::None);
        assert_ne!(ContextSubmenu::None, ContextSubmenu::CopyValueAs);
        assert_ne!(ContextSubmenu::CopyValueAs, ContextSubmenu::ExportValueAs);
    }
}
