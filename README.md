# Unfold

A high-performance JSON viewer built in Rust with [GPUI](https://www.gpui.rs/) (via [gpui-kit](https://gpui-kit.com)).

## Features

- **Fast**: Virtual scrolling handles large JSON files smoothly
- **Tree View**: Expand/collapse nodes with Dadroit-style alignment
- **Syntax Highlighting**: Color-coded keys, strings, numbers, booleans, null
- **Search**: Text and RegEx search with case-sensitivity toggle
- **Keeps file order**: Object keys show in the order they appear in the file
- **Copy Options**: Copy value, key, or JSON path with keyboard shortcuts or context menu
- **Context Menu**: Right-click for copy options, export, and expand/collapse children
- **Native Menu Bar**: Full macOS menu bar with all actions
- **Dark/Light Theme**: Toggle between dark and light modes (Cmd+T)
- **Keyboard Shortcuts**: Navigate efficiently without touching the mouse (Cmd+/ to see all)
- **Check for Updates**: Stay up to date with the latest version
- **Better Errors**: Parse errors show line numbers for easy debugging
- **Multiple Windows**: One app, many windows; files opened from Finder reuse an empty window

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
cargo install cargo-bundle --locked
cargo bundle --release
```

### Requirements

- Recent stable Rust (2024 edition)
- macOS, Windows, or Linux
- Linux build packages: `libxkbcommon-x11-dev libwayland-dev libfontconfig-dev libx11-xcb-dev libvulkan-dev`

## Usage

### Opening Files

- Click "Open" on the welcome screen, or drag and drop a file onto the window, or
- Press `Cmd+O` (macOS) / `Ctrl+O` (Windows/Linux)
- Pass a file path as a command-line argument

### Navigation

- Click nodes to expand/collapse
- `↑` / `↓` move the selection, `→` / `←` expand and collapse
- Scroll to navigate large files
- Use search to find specific values

### Copy Options

Select a node and use keyboard shortcuts or right-click for copy options:

- `Cmd+C` - Copy value
- `Cmd+Shift+C` - Copy key name
- `Cmd+Option+C` - Copy JSON path

### Context Menu (Right-Click)

Right-click any node for:
- **Copy Key** - Copy the key name
- **Copy Value** - Copy the value
- **Copy Value As** - Copy as minified or formatted JSON
- **Copy Path** - Copy the JSON path
- **Export Value As** - Export to JSON file (minified or formatted)
- **Expand/Collapse All Children** - Expand or collapse all nested nodes

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

Press `Cmd+/` to see all shortcuts in-app.

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl+O` | Open file |
| `Cmd/Ctrl+N` | New window |
| `Cmd/Ctrl+W` | Close window |
| `Cmd/Ctrl+Shift+E` | Open in external editor |
| `Cmd/Ctrl+F` | Focus search |
| `Cmd/Ctrl+T` | Toggle theme |
| `Cmd/Ctrl+/` | Show keyboard shortcuts |
| `Cmd/Ctrl+C` | Copy selected value |
| `Cmd/Ctrl+Shift+C` | Copy key name |
| `Cmd/Ctrl+Option+C` | Copy JSON path |
| `Enter` | Next search result |
| `Shift+Enter` | Previous search result |
| `Escape` | Clear search / close dialogs |
| `↑` `↓` `→` `←` | Move selection, expand, collapse |

## Development

```bash
# Run in development mode
cargo run

# Run with a specific file
cargo run -- path/to/file.json

# Run tests (app and core crate)
cargo test --workspace

# Format code
cargo fmt

# Lint
cargo clippy

# Create macOS .app bundle
cargo bundle --release
```

## Version History

### v1.0.0 (Current)

- Tree view with expand/collapse
- Virtual scrolling for large files
- Syntax highlighting
- Text and RegEx search
- Dark/light theme toggle
- JSON path display in status bar on selection
- Native macOS menu bar
- Right-click context menu with submenus
- Copy value, key, or path (keyboard shortcuts + context menu)
- Copy/Export as minified or formatted JSON
- Expand/collapse all children
- Help overlay with keyboard shortcuts (Cmd+/)
- Check for updates from GitHub
- Open in external editor
- Better error messages with line numbers
- CLI argument support
- Multi-window support

### Future

- [ ] Multiple file tabs
- [ ] Structural diff / comparison
- [ ] JSON-Lines support
- [ ] JSON formatting (full file reformat/minify) - *partial: copy as formatted/minified already works*

## Tech Stack

- **Language**: Rust
- **GUI**: [GPUI](https://www.gpui.rs/) through [gpui-kit](https://gpui-kit.com) 0.10 (components, native menus, file dialogs)
- **JSON Parsing**: serde_json
- **HTTP Client**: reqwest (for update checks)
- **Version Comparison**: semver

## License

MIT

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

We use [Conventional Commits](https://www.conventionalcommits.org/) for automatic versioning:

```bash
feat: add new feature      # Minor version bump
fix: fix a bug             # Patch version bump
feat!: breaking change     # Major version bump
```
