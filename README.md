# Unfold

A high-performance JSON viewer built in Rust with the Iced GUI framework.

## Features

- **Fast**: Virtual scrolling handles large JSON files smoothly
- **Tree View**: Expand/collapse nodes with Dadroit-style alignment
- **Syntax Highlighting**: Color-coded keys, strings, numbers, booleans, null
- **Search**: Text and RegEx search with case-sensitivity toggle
- **Copy Values**: Click any node to copy its value and see its JSON path
- **Dark/Light Theme**: Toggle between dark and light modes
- **Keyboard Shortcuts**: Navigate efficiently without touching the mouse
- **Better Errors**: Parse errors show line numbers for easy debugging

## Screenshot

*(Coming soon)*

## Installation

### macOS

Download the latest `.dmg` from [Releases](https://github.com/maumercado/unfold/releases), open it, and drag Unfold to your Applications folder.

### From Source

```bash
# Clone the repository
git clone https://github.com/maumercado/unfold.git
cd unfold

# Build and run
cargo run --release

# Or create a macOS .app bundle
cargo install cargo-bundle
cargo bundle --release
```

### Requirements

- Rust 1.75+ (2024 edition)
- macOS, Windows, or Linux

## Usage

### Opening Files

- Click "Open File" on the welcome screen, or
- Press `Cmd+O` (macOS) / `Ctrl+O` (Windows/Linux)
- Pass a file path as a command-line argument

### Navigation

- Click nodes to expand/collapse
- Scroll to navigate large files
- Use search to find specific values

### Copy & Path

- Click any value to copy it to clipboard
- The status bar shows the JSON path (e.g., `root.users[2].email`)

### Search

1. Press `Cmd+F` to focus the search input
2. Type your query
3. Press `Enter` for next result, `Shift+Enter` for previous
4. Toggle options:
   - **Aa** - Case-sensitive search
   - **.*** - RegEx search

### Theme

- Press `Cmd+T` (macOS) / `Ctrl+T` (Windows/Linux) to toggle dark/light mode
- Or click the theme button in the toolbar

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl+O` | Open file |
| `Cmd/Ctrl+F` | Focus search |
| `Cmd/Ctrl+T` | Toggle theme |
| `Cmd/Ctrl+N` | New window |
| `Enter` | Next search result |
| `Shift+Enter` | Previous search result |
| `Escape` | Clear search |
| `Cmd/Ctrl+G` | Next result (alternative) |
| `Cmd/Ctrl+Shift+G` | Previous result |

## Development

```bash
# Run in development mode
cargo run

# Run with a specific file
cargo run -- path/to/file.json

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Create macOS .app bundle
cargo bundle --release
```

## Version History

### v1.0.0

- [x] Tree view with expand/collapse
- [x] Virtual scrolling for large files
- [x] Syntax highlighting
- [x] Text and RegEx search
- [x] Keyboard shortcuts
- [x] Copy node value to clipboard
- [x] Show JSON path on click
- [x] Dark/light theme toggle
- [x] Better error messages with line numbers
- [x] CLI argument support
- [x] New window command

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
