# Unfold - Project Specification

## Project Overview

A high-performance, native JSON viewer built in Rust with Iced GUI framework, designed to handle large JSON files (gigabytes) efficiently with advanced features like diffing, formatting, and search.

**Project Name:** `unfold`  
**Binary name:** `unfold`  
**Tagline:** "Unfold your JSON"

**Target Audience:**
- Backend developers debugging API responses
- DevOps engineers analyzing log files
- Data engineers working with large JSON datasets
- QA engineers comparing test outputs

## Core Value Propositions

1. **Native Performance**: Handle multi-gigabyte JSON files with minimal memory overhead
2. **Smart Memory Management**: Stream and lazy-load data, 1:1 file-to-RAM ratio target
3. **Developer-Focused**: Built by developers, for developers - practical features over fancy UI
4. **Cross-Platform**: Single codebase for Windows, macOS, Linux
5. **Dual Interface**: Primary GUI application with optional CLI for automation

**Note on Interfaces:**
- **Primary**: Iced GUI application for interactive viewing and analysis
- **Future**: Optional CLI tool for scripting and automation (post-v1.0)
- Both share the same core parsing and processing logic

## Technical Goals

### Performance Targets
- Load time: < 2 seconds for 100MB files
- Memory usage: ~1.1x file size (including index overhead)
- Search speed: > 50,000 results/second
- Tree navigation: 60fps smooth scrolling even with millions of nodes
- Startup time: < 500ms cold start

### Reliability Targets
- Handle malformed JSON gracefully with clear error messages
- No crashes on OOM - warn user and offer chunked viewing
- Auto-save user preferences and window state
- Undo/redo support for editing operations

## Feature Set

### Phase 1: Core Viewer (MVP)
**Goal:** Prove the concept with high-performance viewing

#### Must Have
- [x] Open and parse JSON files (up to 2GB)
- [x] Tree view with expand/collapse
- [x] Virtual scrolling (render only visible nodes)
- [x] Syntax highlighting
- [x] Basic search (text match)
- [x] Copy node value
- [x] Show node path (e.g., `root.users[2].email`)
- [x] File info display (size, node count, depth)

#### Nice to Have
- [ ] Multiple file tabs
- [ ] Dark/light theme toggle
- [ ] Keyboard navigation (vim-style bindings)
- [ ] Bookmarks for frequently accessed nodes

### Phase 2: Advanced Features
**Goal:** Differentiate from competitors

#### Must Have
- [x] JSON formatting (pretty print, minify, custom indent)
- [x] RegEx search support
- [x] Search result navigation (next/previous)
- [x] Export selected node to file
- [x] JSON validation with error highlighting

#### Nice to Have
- [ ] JSON Schema validation
- [ ] Auto-format on open (configurable)
- [ ] Search history
- [ ] Custom color schemes

### Phase 3: Comparison & Collaboration
**Goal:** Make it indispensable for API development

#### Must Have
- [x] Side-by-side JSON diff view
- [x] Structural comparison (semantic diff)
- [x] Navigate between differences
- [x] Highlight additions, deletions, modifications
- [x] Show diff path/location

#### Nice to Have
- [ ] Three-way merge view
- [ ] Export diff report (HTML, Markdown)
- [ ] Ignore patterns (e.g., timestamps, IDs)
- [ ] Snapshot/compare modes

### Phase 4: Power Features
**Goal:** Professional-grade tooling

#### Must Have
- [ ] Multiple file merge view
- [ ] JSON-Lines / ndjson support
- [ ] Auto-refresh for log file monitoring
- [ ] Filter by JSON path (e.g., `$.users[*].email`)

#### Nice to Have
- [ ] API integration (fetch and view)
- [ ] Export to CSV/XML
- [ ] Scripting/plugin system
- [ ] CLI version for automation

### Phase 5: Polish & Distribution
**Goal:** Production-ready release

#### Must Have
- [ ] Comprehensive error handling
- [ ] User documentation
- [ ] Installer packages (Windows MSI, macOS DMG, Linux AppImage)
- [ ] Auto-update mechanism
- [ ] Crash reporting (opt-in)

#### Nice to Have
- [ ] Tutorial/onboarding
- [ ] Performance profiling mode
- [ ] Community plugin marketplace

## Non-Functional Requirements

### Performance
- Must handle 1GB+ files without freezing UI
- Search must be interruptible (user can cancel)
- File operations must be async (non-blocking UI)
- Memory usage must be monitored and displayed

### Usability
- Intuitive keyboard shortcuts (discoverable via help menu)
- Responsive UI - no operation should block for > 100ms
- Clear error messages with actionable suggestions
- Preserve user preferences across sessions

### Portability
- Single executable, no runtime dependencies
- Support Windows 10+, macOS 10.15+, Ubuntu 20.04+
- Consistent behavior across platforms
- File paths handled with platform abstractions

### Security
- No telemetry by default (opt-in only)
- Local-only processing (no cloud uploads)
- Safe file handling (no arbitrary code execution)
- Clear privacy policy

## Technical Constraints

### Must Use
- Rust (latest stable)
- Iced GUI framework
- `serde_json` for JSON parsing
- `tokio` for async runtime

### Cannot Use
- Web technologies (electron, tauri) for main app
- Unsafe Rust unless absolutely necessary and documented
- GPL-licensed dependencies (prefer MIT/Apache-2.0)

### Nice to Have
- `regex` crate for search
- `memmap2` for large file handling
- `rayon` for parallel processing
- `similar` or `diff` crate for comparison
- `syntect` for syntax highlighting

## Success Metrics

### Technical Metrics
- Load time 10x faster than VSCode for 100MB+ files
- Memory usage within 20% of file size
- Zero crashes in 1000 hours of testing
- Startup time < 1 second

### User Metrics
- Primary use case supported in < 3 clicks
- 90% of features discoverable without documentation
- Positive feedback from 5 beta testers
- Used daily by creator (dogfooding)

### Business Metrics (Portfolio Value)
- Demonstrates systems programming skills
- Shows understanding of performance optimization
- Proves ability to complete complex projects
- GitHub stars/engagement (target: 100+ stars)

## Out of Scope (v1.0)

- JSON editing (only viewing/formatting/comparing)
- Cloud storage integration
- Real-time collaboration
- Mobile versions
- Browser extension
- JSON generation/mocking
- GraphQL/API testing features
- Built-in file compression

## Future Considerations

- JSON editing mode (post v1.0)
- Plugin system for extensibility
- Cloud sync for preferences
- Team collaboration features
- Integration with API testing tools (Postman, Insomnia)
- VS Code extension version

## References & Inspiration

- **Dadroit JSON Viewer**: Performance benchmarks and feature ideas
- **jq**: Command-line JSON processing power
- **VS Code**: UI/UX patterns for code editors
- **Delta (diff tool)**: Diff visualization approaches
- **Sublime Text**: Performance characteristics for large files

## Open Questions

1. Should we support YAML/TOML viewing as well?
2. Licensing: MIT or Apache-2.0?
3. Name: `json-forge`, `rapidjson-viewer`, `json-lens`, other?
4. Should Phase 1 include file watching/auto-refresh?
5. Minimum supported Rust version (MSRV)?

## Document Version

- Version: 0.1.0
- Last Updated: 2025-12-12
- Author: Mau
- Status: Draft
