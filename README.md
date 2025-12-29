# Unfold

A high-performance JSON viewer built in Rust with the Iced GUI framework.

## Features

- **Fast**: Virtual scrolling handles large JSON files smoothly
- **Tree View**: Expand/collapse nodes with Dadroit-style alignment
- **Syntax Highlighting**: Color-coded keys, strings, numbers, booleans, null
- **Search**: Text and RegEx search with case-sensitivity toggle
- **Keyboard Shortcuts**: Navigate efficiently without touching the mouse

## Screenshot

*(Coming soon)*

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/unfold.git
cd unfold

# Build and run
cargo run --release
```

### Requirements

- Rust 1.75+ (2024 edition)
- macOS, Windows, or Linux

## Usage

### Opening Files

- Click "Open File..." on the welcome screen, or
- Press `Cmd+O` (macOS) / `Ctrl+O` (Windows/Linux)

### Navigation

- Click nodes to expand/collapse
- Scroll to navigate large files
- Use search to find specific values

### Search

1. Press `Cmd+F` to focus the search input
2. Type your query
3. Press `Enter` for next result, `Shift+Enter` for previous
4. Toggle options:
   - **Aa** - Case-sensitive search
   - **.\*** - RegEx search

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl+O` | Open file |
| `Cmd/Ctrl+F` | Focus search |
| `Enter` | Next search result |
| `Shift+Enter` | Previous search result |
| `Escape` | Clear search |
| `Cmd/Ctrl+G` | Next result (alternative) |
| `Cmd/Ctrl+Shift+G` | Previous result |

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Roadmap

### Current (v0.1.0)

- [x] Tree view with expand/collapse
- [x] Virtual scrolling
- [x] Syntax highlighting
- [x] Text and RegEx search
- [x] Keyboard shortcuts

### Planned (v1.0)

- [ ] Copy node value to clipboard
- [ ] Show JSON path on selection
- [ ] Dark/light theme toggle
- [ ] Multiple file tabs

### Future

- [ ] JSON formatting (pretty print, minify)
- [ ] Structural diff / comparison
- [ ] JSON-Lines support

## Tech Stack

- **Language**: Rust
- **GUI**: [Iced](https://github.com/iced-rs/iced) 0.14
- **JSON Parsing**: serde_json
- **File Dialogs**: rfd
- **RegEx**: regex crate

## License

MIT

## Contributing

Contributions welcome! Please open an issue first to discuss what you'd like to change.
