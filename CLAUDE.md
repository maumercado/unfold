# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Unfold** is a high-performance JSON viewer built in Rust with the Iced GUI framework. The project aims to handle multi-gigabyte JSON files with native performance, efficient memory usage, and developer-focused features.

**Current Status**: Core viewer implemented with search, keyboard shortcuts, and virtual scrolling.

## Project Structure

```
unfold/
├── Cargo.toml
├── CLAUDE.md              # This file
├── README.md              # User-facing documentation
├── docs/
│   └── DESIGN_DECISIONS.md  # Architecture decisions and trade-offs
└── src/
    ├── main.rs            # App state, UI, keyboard handling, view logic
    └── parser/
        ├── mod.rs         # Module exports
        ├── node.rs        # JsonNode and JsonValue types
        ├── tree.rs        # JsonTree - flat array storage
        └── builder.rs     # Build tree from serde_json::Value
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
# Run in development mode
cargo run

# Run with release optimizations (faster)
cargo run --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Type-check without building
cargo check
```

## Current Dependencies

```toml
[dependencies]
iced = { version = "0.14.0", features = ["tokio", "advanced"] }
rfd = "0.16.0"           # File dialogs
regex = "1.11"           # RegEx search
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Implemented Features (v0.1.0)

### Core Viewer
- [x] Open and parse JSON files
- [x] Tree view with expand/collapse (Dadroit-style alignment)
- [x] Virtual scrolling (render only visible nodes)
- [x] Syntax highlighting (keys, strings, numbers, booleans, null)
- [x] Zebra striping for readability
- [x] Dynamic window title showing filename
- [x] Status bar with node count and load time

### Search
- [x] Text search with highlighting
- [x] Case-sensitive toggle (Aa button)
- [x] RegEx search toggle (.* button)
- [x] Navigate between results (Prev/Next buttons)
- [x] Auto-expand path to search results
- [x] Auto-scroll to current result

### Keyboard Shortcuts
- [x] `Cmd/Ctrl+O` - Open file
- [x] `Cmd/Ctrl+F` - Focus search input
- [x] `Enter` - Next search result (works in search input too)
- [x] `Shift+Enter` - Previous search result
- [x] `Escape` - Clear search
- [x] `Cmd/Ctrl+G` / `Cmd/Ctrl+Shift+G` - Navigate results (alternative)

## Remaining for v1.0

### High Priority
- [ ] Copy node value to clipboard
- [ ] Show node path on selection (e.g., `root.users[2].email`)
- [ ] Better error messages for parse failures

### Medium Priority
- [ ] Dark/light theme toggle
- [ ] Multiple file tabs
- [ ] JSON formatting (pretty print, minify)

### Lower Priority (maybe v1.1)
- [ ] JSON-Lines / ndjson support
- [ ] File watching and auto-refresh
- [ ] Structural diff / comparison view
- [ ] Export selected node

## Architecture

### Key Data Structures

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

pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array,
    Object,
}
```

### App State (in main.rs)

```rust
struct App {
    tree: Option<JsonTree>,
    current_file: Option<PathBuf>,
    flat_rows: Vec<FlatRow>,      // Pre-computed for virtual scrolling
    scroll_offset: f32,
    viewport_height: f32,
    // Search state
    search_query: String,
    search_results: Vec<usize>,
    search_result_index: Option<usize>,
    search_case_sensitive: bool,
    search_use_regex: bool,
    current_modifiers: Modifiers,  // For Shift+Enter detection
    // Widget IDs
    tree_scrollable_id: WidgetId,
    search_input_id: WidgetId,
}
```

### Message-Driven Architecture (Elm-style)

```rust
enum Message {
    OpenFileDialog,
    FileSelected(Option<PathBuf>),
    ToggleNode(usize),
    Scrolled(Viewport),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrev,
    SearchSubmit,          // From text input, checks modifiers
    ToggleCaseSensitive,
    ToggleRegex,
    KeyPressed(Key, Modifiers),
    ModifiersChanged(Modifiers),
    ClearSearch,
    FocusSearch,
}
```

## Design Decisions

See `docs/DESIGN_DECISIONS.md` for detailed rationale on:
- Why Iced (not Tauri)
- Why flat array (not tree pointers)
- Why index-based children (not Box/Rc)

### Key Patterns Used

1. **Virtual Scrolling**: Only render visible rows + buffer
2. **FlatRow Pre-computation**: Flatten tree on change, not on render
3. **Modifier Tracking**: Track keyboard modifiers globally for Shift+Enter in text input
4. **Widget Operations**: Use `operate(focusable::focus(id))` for programmatic focus

## Performance Considerations

- **Virtual scrolling**: Only visible rows are rendered
- **Pre-computed FlatRows**: Tree flattening happens once per expand/collapse
- **Index-based tree**: Better cache locality than pointer-based
- **Subscription filtering**: Only emit messages for events we handle

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

## Testing

Tests are in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

Run with: `cargo test`
