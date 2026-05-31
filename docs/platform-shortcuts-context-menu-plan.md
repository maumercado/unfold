# Platform Shortcuts and Context Menu Targeting Fix

Status: **Completed**

## Goal

Fix two UX bugs:

- macOS paste should use `Cmd+V`, not require `Ctrl+V`.
- Right-click copy on large JSON trees should target the clicked property and avoid scroll jumps caused by stale virtual-row coordinates.

## Architecture

Keyboard shortcuts are routed through `event::listen_with` in `src/main.rs`. Iced's `Modifiers::command()` already maps to `Cmd` on macOS and `Ctrl` on other platforms, so app-level shortcuts should use that platform-primary modifier directly.

Context menu rendering also lives in `src/main.rs`. The current menu position is estimated from flattened row index and scroll offset. That estimate is fragile with large virtualized trees. Cursor tracking gives the menu a stable screen position, and the menu action should use the right-clicked node from `context_menu_state` instead of relying only on current selection.

Native menu setup lives in `src/menu.rs`. Native predefined paste/copy menu items can intercept macOS accelerators before Iced widgets receive them, so app-owned shortcuts should not compete with them.

## Setup Steps

1. Replace `modifiers.command() || modifiers.control()` with `modifiers.command()` in app shortcut handling.
2. Rename shortcut helper to `primary_modifier` and update all shortcut checks.
3. Remove native predefined `Copy` and `Paste` menu items from the Edit menu.
4. Track cursor movement through `Event::Mouse(mouse::Event::CursorMoved { position })`.
5. Add `Message::CursorMoved(Point)` and store `last_cursor_position` in `App`.
6. Change right-click message to `ShowContextMenu(usize)` and use stored cursor position when opening the menu.
7. Prefer the node stored in `context_menu_state` for context-menu copy/export/expand/collapse actions.
8. Keep keyboard copy shortcuts using `selected_node`.
9. Run `cargo fmt`, `cargo check`, and `cargo test`.

## Migration Strategy

No data migration needed. Changes are local to event routing, menu setup, and context menu state.

## What Does NOT Change

- JSON parsing and tree storage.
- Search matching and highlighting.
- Copy/export serialization format.
- Virtual scrolling data model.
- Theme and visual design.

## Future Improvements

- Add GUI smoke tests if the project later gains UI automation.
- Revisit native context menus if Iced and muda integration can preserve widget shortcuts cleanly.
