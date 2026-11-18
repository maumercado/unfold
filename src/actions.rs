//! Every user command is a GPUI *action*.
//!
//! An action is a small type that key bindings, menu items and buttons can all
//! trigger. GPUI dispatches it from the focused element up to the window root,
//! so the innermost handler wins: when the search box is focused, `cmd-c` hits
//! the text input's own Copy binding; anywhere else it reaches `CopyValue`.
//! No "was this key already handled?" checks are needed.

use gpui_kit::{Action, App, KeyBinding, KeyBindingContextPredicate, actions};

actions!(
    unfold,
    [
        // File
        OpenFile,
        NewWindow,
        OpenFileInNewWindow,
        OpenInExternalEditor,
        CloseWindow,
        Quit,
        // App menu (macOS)
        About,
        Hide,
        HideOthers,
        ShowAll,
        Minimize,
        Zoom,
        // Edit
        CopyValue,
        CopyKey,
        CopyPath,
        CopyValueMinified,
        CopyValueFormatted,
        PasteIntoSearch,
        // Context menu
        ExportJson,
        ExportMinifiedJson,
        ExportFormattedJson,
        ExpandAllChildren,
        CollapseAllChildren,
        // Search
        FocusSearch,
        SearchNext,
        SearchPrev,
        ClearSearch,
        ToggleCaseSensitive,
        ToggleRegex,
        // Tree navigation
        SelectPrev,
        SelectNext,
        ExpandSelected,
        CollapseSelected,
        // View / Help
        ToggleTheme,
        ShowShortcuts,
        CheckForUpdates,
        InstallCli,
    ]
);

/// Key context set on the viewer's root element.
pub const VIEWER_CONTEXT: &str = "Viewer";

type Binding = (&'static str, Box<dyn Action>, Option<&'static str>);

/// Every shortcut. `secondary` means Cmd on macOS and Ctrl elsewhere, the
/// usual convention for desktop apps.
fn bindings() -> Vec<Binding> {
    vec![
        ("secondary-o", Box::new(OpenFile), None),
        ("secondary-n", Box::new(NewWindow), None),
        (
            "secondary-shift-e",
            Box::new(OpenInExternalEditor),
            Some(VIEWER_CONTEXT),
        ),
        ("secondary-w", Box::new(CloseWindow), None),
        ("secondary-q", Box::new(Quit), None),
        #[cfg(target_os = "macos")]
        ("cmd-h", Box::new(Hide), None),
        #[cfg(target_os = "macos")]
        ("cmd-alt-h", Box::new(HideOthers), None),
        #[cfg(target_os = "macos")]
        ("cmd-m", Box::new(Minimize), None),
        ("secondary-c", Box::new(CopyValue), Some(VIEWER_CONTEXT)),
        ("secondary-shift-c", Box::new(CopyKey), Some(VIEWER_CONTEXT)),
        ("secondary-alt-c", Box::new(CopyPath), Some(VIEWER_CONTEXT)),
        (
            "secondary-v",
            Box::new(PasteIntoSearch),
            Some(VIEWER_CONTEXT),
        ),
        ("secondary-f", Box::new(FocusSearch), Some(VIEWER_CONTEXT)),
        ("enter", Box::new(SearchNext), Some(VIEWER_CONTEXT)),
        ("shift-enter", Box::new(SearchPrev), Some(VIEWER_CONTEXT)),
        ("secondary-g", Box::new(SearchNext), Some(VIEWER_CONTEXT)),
        (
            "secondary-shift-g",
            Box::new(SearchPrev),
            Some(VIEWER_CONTEXT),
        ),
        ("escape", Box::new(ClearSearch), Some(VIEWER_CONTEXT)),
        ("up", Box::new(SelectPrev), Some(VIEWER_CONTEXT)),
        ("down", Box::new(SelectNext), Some(VIEWER_CONTEXT)),
        ("right", Box::new(ExpandSelected), Some(VIEWER_CONTEXT)),
        ("left", Box::new(CollapseSelected), Some(VIEWER_CONTEXT)),
        ("secondary-t", Box::new(ToggleTheme), None),
        ("secondary-/", Box::new(ShowShortcuts), None),
        ("secondary-?", Box::new(ShowShortcuts), None),
    ]
}

pub fn bind_keys(cx: &mut App) {
    let bindings = bindings()
        .into_iter()
        .filter_map(|(keys, action, context)| {
            let context = context.map(|c| {
                KeyBindingContextPredicate::parse(c)
                    .expect("valid context")
                    .into()
            });
            KeyBinding::load(keys, action, context, false, None, cx)
                .inspect_err(|e| eprintln!("invalid key binding {keys}: {e:?}"))
                .ok()
        })
        .collect::<Vec<_>>();
    cx.bind_keys(bindings);
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_binding_parses() {
        use gpui_kit::{Keystroke, OperatingSystem};
        for os in [
            OperatingSystem::Mac,
            OperatingSystem::Linux,
            OperatingSystem::Windows,
        ] {
            for (keys, _, _) in super::bindings() {
                let ok = Keystroke::parse_with_os(keys, os).is_ok();
                assert!(ok, "{keys} on {os:?}");
            }
        }
    }
}
