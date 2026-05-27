# Window Open and Search Fixes

Status: **Completed**

## Goal

Fix three UX regressions without breaking cross-platform behavior:
- Opening files should never trigger recursive process spawning.
- Pasting into search should work reliably.
- `Cmd/Ctrl+N` should open new windows offset from current window.

## Architecture

The app uses separate processes for additional windows. This plan introduces explicit launch options for child processes so each process knows whether it was launched as a handoff child, and where its initial window should appear.

Key pieces:
- `LaunchOptions` parser in `src/main.rs` for file path, spawn-handoff marker, and optional window position.
- Window position tracking via runtime window events (`Opened`, `Moved`) to compute cascaded placement.
- macOS open-file handler guard so handoff children do not re-spawn from open-file delegate callbacks.
- Keyboard event routing through `event::listen_with` so captured events are not reprocessed as app-level shortcuts.

## Setup Steps

1. Add launch options parsing for:
   - `--window-x <f32>`
   - `--window-y <f32>`
   - `--spawn-handoff`
2. Spawn child windows with explicit options:
   - Position for cascaded `Cmd/Ctrl+N`
   - Handoff marker for macOS file-open child processes
3. Capture `window::Event::Opened` and `window::Event::Moved` and store last known position.
4. Use fixed cascade offset (`32x32`) for new windows.
5. Prevent app-level key handling for captured widget events.
6. Add search paste fallback (`Cmd/Ctrl+V`) when key event is ignored.
7. Add/update unit tests for launch-option parsing and path de-duplication.
8. Run `cargo fmt`, `cargo test`, and `cargo check`.

## Migration Strategy

This is an in-place behavioral fix; no data migration required.

Changes are localized to:
- `src/main.rs`
- `src/macos_open.rs`
- `src/message.rs`
- `src/menu.rs`

## What Does NOT Change

- JSON parsing and tree representation.
- Search matching algorithm and highlighting behavior.
- Export and copy-node feature logic.
- Theme and rendering architecture.

## Future Improvements

- Persist last window position to config for cross-process restoration after app restart.
- Add monitor-bounds clamping for cascaded windows.
- Replace process-per-window with single-process multi-window architecture if Iced support and complexity trade-offs justify it.
