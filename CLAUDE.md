# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Unfold** is a high-performance JSON viewer built in Rust with **GPUI** (through **gpui-kit**). The project aims to handle multi-gigabyte JSON files with native performance, efficient memory usage, and developer-focused features.

**Current Status**: Full viewer (search, copy/export, themes, context menu, menu bar, multiple windows). Moved from Iced to GPUI in v2.0; see `docs/gpui-migration-plan.md` and `docs/DESIGN_DECISIONS.md` (decision 1b).

## Project Structure

```
unfold/
├── Cargo.toml             # App package (`unfold` binary) + workspace root
├── build.rs               # Windows icon resource
├── assets/                # App icons (icns, ico, png)
├── scripts/               # Bench JSON generator, macOS document-type plist fixer
├── docs/                  # Design decisions, plans, GPUI spike results
├── crates/
│   └── unfold-core/       # UI-free core: no GPUI, must stay that way
│       ├── examples/bench.rs  # Headless load benchmark
│       └── src/
│           ├── parser/    # JsonNode, JsonTree (flat array), builder
│           ├── flat_row.rs   # FlatRow, ValueType, flatten_visible_nodes
│           ├── search.rs, json_export.rs, config.rs, parse_error.rs
└── src/                   # GPUI app
    ├── main.rs            # Startup, CLI args, windows, Finder open events
    ├── viewer.rs          # The window view: toolbar, tree, status bar, screens, UI tests
    ├── document.rs        # Open file + selection + search state, no GPUI (unit tested)
    ├── row.rs             # FlatRow -> one line of text + color spans
    ├── actions.rs         # Every command as a GPUI action + key bindings
    ├── menus.rs           # Native menu bar
    ├── dialogs.rs         # Shortcuts, About, update check, messages
    ├── settings.rs        # Global persisted Config (theme, CLI installed)
    ├── theme.rs           # Dark/light colors
    ├── platform.rs        # CLI symlink install, open in text editor
    └── update_check.rs    # GitHub releases check (blocking, background thread)
```

## Learning-Focused Development

**The developer is a Rust novice learning Rust through this project.**

### Teaching Approach:
1. **Explain BEFORE coding** - Describe what we're about to do and why
2. **Teach concepts** - Explain Rust concepts as they come up
3. **Library explanations** - When adding dependencies, explain what they do
4. **Celebrate mistakes** - Errors are learning opportunities; explain compiler messages
5. **Write tests alongside code** - Use TDD when appropriate

## Development Commands

```bash
cargo run -- path/to/file.json        # Run (debug)
cargo run --release -- file.json      # Run (fast)
cargo test --workspace                # All tests (app + core)
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings   # Same as CI
cargo bundle --release                # macOS .app (cargo install cargo-bundle --locked)
python3 scripts/gen_test_json.py      # Large test files into bench-data/ (git-ignored)
```

## Dependencies

```toml
unfold-core = { path = "crates/unfold-core" }  # parser, search, export, config
gpui-kit = "=0.10.0"   # GPUI + components (Button, Input, Dialog, NativeMenu, Scrollbar)
futures = "0.3"        # Channel for Finder open events
reqwest = { ..., features = ["blocking", "json", "rustls-tls"] }  # Update check
semver, serde, serde_json, image (window icon)
```

`unfold-core` uses `serde_json` with `preserve_order`, so object keys keep file order.

## Architecture

### Key Data Structures (unfold-core)

```rust
// Flat array storage for cache efficiency
pub struct JsonTree {
    nodes: Vec<JsonNode>,
    root_index: usize,
}

// Index-based references (not pointers)
pub struct JsonNode {
    pub key: Option<String>,
    pub value: JsonValue,
    pub depth: usize,
    pub children: Vec<usize>,  // Indices into nodes array
    pub expanded: bool,
}
```

`flatten_visible_nodes` turns the tree into `Vec<FlatRow>` (one per visible line), rebuilt after expand/collapse.

### GPUI App

- **Views**: `Viewer` (one per window) implements `Render`. It lives in an `Entity<Viewer>`; handlers get `&mut Viewer` through `cx.listener(...)`, change state, and call `cx.notify()`.
- **State split**: `Viewer` holds GPUI things (focus, search input, scroll handle) and a `LoadState` (`Empty`, `Loading`, `Loaded(Document)`, `Failed(ParseError)`). `Document` holds the tree, rows, selection and search, with no GPUI types, so it is unit tested directly.
- **Actions, not messages**: every command is an action in `actions.rs`. Key bindings, menu items, context menu items and buttons all trigger the same actions. Handlers are registered with `.on_action(...)` on the viewer's root element.
- **Key contexts**: viewer bindings use the `Viewer` context. A focused text input (`Input` context) is deeper, so its own Cmd+C / Cmd+V / Enter bindings win. Enter in the search box propagates to our `enter` binding, so do not also handle `InputEvent::PressEnter`.
- **Focus**: actions only reach the viewer when focus is inside it. The viewer re-focuses itself when the window has no focus; the app-level `OpenFile` fallback prompts in the active window (deferred, because a window cannot be read during its own update).
- **Rows**: GPUI `uniform_list` (fixed 16 px rows, renders only visible rows) with `Unconstrained` horizontal sizing, width measured from the widest row. Each row is a single `StyledText` with color highlights from `row::build_row_text`.
- **Context menu**: right click selects the row, then shows a `NativeMenu`; menu actions act on the selection.
- **Windows**: one process, many windows (Cmd+N, cascade +32 px). Finder/Dock opens arrive through `Application::on_open_urls` and reuse an empty window if there is one. The app quits when the last window closes.
- **Background work**: file loading, update checks, CLI install and export writes run in `cx.background_spawn`, then update the view with `cx.spawn_in(window, ...)`. A load generation counter drops stale loads.
- **Settings**: `Config` lives in a GPUI global (`settings.rs`); theme changes apply to every window and to gpui-component's theme.
- **Platform modifier**: bindings write `secondary-x`, resolved to Cmd on macOS and Ctrl elsewhere via `KeyBinding::load` (`KeyBinding::new` does not understand `secondary`).

## Design Decisions

See `docs/DESIGN_DECISIONS.md` for the rationale on the UI framework (Iced, then GPUI), the flat array tree, and index-based children.

## Performance Considerations

- **Virtual scrolling**: `uniform_list` renders only visible rows (about 1 ms CPU per frame at 1M rows)
- **Pre-computed FlatRows**: flattening happens once per expand/collapse, never per frame
- **One text element per row**: color spans instead of one element per colored piece
- **Index-based tree**: better cache locality than pointer-based
- **Background parsing**: the window stays responsive while big files load

## Coding Guidelines

### DO:
```rust
// Use references when possible
fn process_node(node: &JsonNode) { }

// Use iterators
nodes.iter().filter(|n| n.depth > 5).count()

// Handle Option/Result properly
if let Some(node) = tree.get_node(index) { }
```

### DON'T:
```rust
// Unnecessary clones
fn process_node(node: JsonNode) { }  // Takes ownership!

// Collect when not needed
nodes.iter().collect::<Vec<_>>().len()  // Use .count()
```

- Keep GPUI types out of `unfold-core` and `document.rs`/`row.rs`.
- gpui-kit is pinned with `=`; upgrade on purpose, then rerun `cargo test --workspace`. If the lockfile complains about `wgpu`, run `cargo update -p wgpu --precise <version gpui-kit wants>`.

## Testing

Tests live in the same file as the code. Plain logic uses `#[test]`; UI behavior uses gpui-kit's headless harness:

```rust
#[gpui_kit::test]
fn cmd_c_copies_the_selected_value(cx: &mut TestAppContext) {
    let (window, _) = setup(cx, json!({"name": "Ada"}));   // see viewer.rs ui_tests
    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click(("row", 0usize), cx);
        window.press("secondary-c", cx);
    }).unwrap();
}
```

- Elements must call `.test_support()` (before `.track_focus`) to be found by id.
- Do not `use super::*` in test modules of files that `use gpui_kit::*`: the glob brings GPUI's `test` macro, which shadows `#[test]`. Import what you need explicitly.
- Frame-time check (ignored by default): `cargo test --release -- --ignored --nocapture frame_time`

Run all with: `cargo test --workspace`
