# Unfold - Technical Architecture

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Iced Application                        │
│                      (GUI Layer)                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  Tree View   │  │  Diff View   │  │ Search Panel │    │
│  │   Widget     │  │   Widget     │  │   Widget     │    │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                  │                  │             │
├─────────┼──────────────────┼──────────────────┼─────────────┤
│         │    Application State & Message Bus  │             │
│         │                  │                  │             │
├─────────┼──────────────────┼──────────────────┼─────────────┤
│                     Core Engine Layer                       │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐    │
│  │   Parser     │  │  Diff Engine │  │    Search    │    │
│  │   Module     │  │    Module    │  │    Engine    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Tree Builder │  │  Formatter   │  │ File Handler │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    Storage Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Memory Map   │  │    Cache     │  │ Preferences  │    │
│  │   Handler    │  │   Manager    │  │   Storage    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Core Modules

### 1. Parser Module (`parser/`)

**Responsibility**: Efficient JSON parsing with streaming support

#### Components

**`streaming_parser.rs`**
- Stream-based JSON parsing using `serde_json::StreamDeserializer`
- Incremental parsing for large files
- Error recovery and reporting
- Progress callback for UI updates

**`json_tree.rs`**
- Lightweight tree representation
- Lazy-loading child nodes
- Node metadata (depth, type, size)
- Path tracking for each node

**`validator.rs`**
- JSON syntax validation
- Schema validation (optional)
- Error position reporting
- Suggestion engine for common mistakes

#### Data Structures

```rust
/// Represents a single node in the JSON tree
pub struct JsonNode {
    pub key: Option<String>,        // Object key (None for array items)
    pub value: JsonValue,            // Node value
    pub node_type: JsonType,         // Object, Array, String, etc.
    pub depth: usize,                // Tree depth
    pub offset: usize,               // Byte offset in file
    pub children: Option<Vec<usize>>, // Indices of child nodes (lazy-loaded)
}

/// Efficient JSON value representation
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(Arc<str>),               // Shared string to reduce memory
    Array(usize),                   // Length only, children lazy-loaded
    Object(usize),                  // Size only, children lazy-loaded
}

/// Complete parsed tree
pub struct JsonTree {
    nodes: Vec<JsonNode>,            // Flat array of all nodes
    root_index: usize,               // Index of root node
    file_size: u64,                  // Original file size
    node_count: usize,               // Total node count
    max_depth: usize,                // Maximum tree depth
}
```

#### Performance Strategies

1. **Streaming**: Don't load entire JSON into memory
2. **Lazy children**: Only parse child nodes when expanded in UI
3. **Shared strings**: Use `Arc<str>` for duplicate keys/values
4. **Flat array**: Store tree as flat Vec for cache efficiency
5. **Index-based**: Use indices instead of pointers/references

### 2. Tree View Widget (`ui/tree_view/`)

**Responsibility**: Render tree with virtual scrolling

#### Components

**`virtual_scroller.rs`**
- Calculate visible node range
- Handle scroll events
- Pre-render buffer (render slightly more than visible)
- Smooth scrolling animation

**`tree_renderer.rs`**
- Render individual tree nodes
- Expand/collapse animations
- Syntax highlighting
- Selection highlighting

**`tree_state.rs`**
- Track expanded/collapsed nodes
- Current selection
- Scroll position
- Viewport dimensions

#### Virtual Scrolling Algorithm

```rust
pub struct VirtualScroller {
    viewport_height: f32,        // Visible area height
    node_height: f32,             // Height of each node
    scroll_offset: f32,           // Current scroll position
    total_nodes: usize,           // Total renderable nodes
    buffer_size: usize,           // Extra nodes to render (e.g., 20)
}

impl VirtualScroller {
    pub fn visible_range(&self) -> Range<usize> {
        let start = (self.scroll_offset / self.node_height) as usize;
        let visible_count = (self.viewport_height / self.node_height).ceil() as usize;
        let end = (start + visible_count).min(self.total_nodes);
        
        // Add buffer
        let buffered_start = start.saturating_sub(self.buffer_size);
        let buffered_end = (end + self.buffer_size).min(self.total_nodes);
        
        buffered_start..buffered_end
    }
}
```

### 3. Diff Engine (`diff/`)

**Responsibility**: Compare two JSON structures

#### Components

**`structural_diff.rs`**
- Semantic JSON comparison
- Generate diff tree
- Support different comparison modes

**`diff_algorithms.rs`**
- LCS (Longest Common Subsequence) for arrays
- Object key matching
- Value comparison with type checking

**`diff_visualizer.rs`**
- Format diffs for display
- Color coding (add/delete/modify)
- Diff navigation helpers

#### Diff Data Structure

```rust
pub enum DiffType {
    Equal,
    Added,
    Removed,
    Modified { old: JsonValue, new: JsonValue },
}

pub struct DiffNode {
    pub path: String,              // JSON path (e.g., "root.users[2].email")
    pub diff_type: DiffType,
    pub left_node: Option<usize>,  // Index in left tree
    pub right_node: Option<usize>, // Index in right tree
}

pub struct DiffResult {
    pub diffs: Vec<DiffNode>,
    pub stats: DiffStats,
}

pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub unchanged: usize,
}
```

#### Comparison Modes

```rust
pub enum ComparisonMode {
    Strict,        // Exact match, order matters
    Semantic,      // Ignore array order, focus on content
    Flexible,      // Ignore key order, whitespace
    Schema,        // Compare only structure/types
}
```

### 4. Search Engine (`search/`)

**Responsibility**: Fast text and RegEx search

#### Components

**`text_search.rs`**
- Simple text matching
- Case-sensitive/insensitive options
- Whole word matching

**`regex_search.rs`**
- RegEx pattern matching
- Pattern validation
- Performance-optimized RegEx compilation

**`search_index.rs`** (Phase 2 optimization)
- Build inverted index for repeated searches
- Update index incrementally
- Memory-efficient index structure

#### Search Algorithm

```rust
pub struct SearchEngine {
    tree: Arc<JsonTree>,
    regex_cache: HashMap<String, Regex>,
}

impl SearchEngine {
    pub async fn search(
        &self,
        pattern: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Search in background thread
        tokio::task::spawn_blocking(move || {
            // Iterate through all nodes
            // Match against keys and values
            // Return matching node indices with context
        }).await?
    }
}

pub struct SearchResult {
    pub node_index: usize,
    pub match_location: MatchLocation,
    pub context: String,           // Surrounding text
}

pub enum MatchLocation {
    Key,
    Value,
    Both,
}
```

### 5. Formatter Module (`format/`)

**Responsibility**: JSON formatting operations

#### Components

**`pretty_printer.rs`**
- Configurable indentation
- Line length limits
- Custom formatting rules

**`minifier.rs`**
- Remove all unnecessary whitespace
- Optimize for file size

**`sorter.rs`**
- Sort object keys alphabetically
- Preserve or reorder arrays

#### Formatting Configuration

```rust
pub struct FormatConfig {
    pub indent: Indent,
    pub line_width: Option<usize>,
    pub sort_keys: bool,
    pub trailing_comma: bool,
    pub quote_style: QuoteStyle,
}

pub enum Indent {
    Spaces(usize),
    Tabs,
}

pub enum QuoteStyle {
    Double,
    Single,
    Preserve,
}
```

### 6. File Handler (`file/`)

**Responsibility**: File I/O operations

#### Components

**`file_loader.rs`**
- Async file reading
- Memory-mapped files for large files
- Progress tracking
- Encoding detection (UTF-8, UTF-16, etc.)

**`file_watcher.rs`**
- Monitor file changes
- Auto-refresh on external modifications
- Debounce rapid changes

**`file_exporter.rs`**
- Export to various formats (JSON, CSV, XML)
- Save formatted JSON
- Export diff reports

#### Memory Mapping Strategy

```rust
pub struct FileLoader {
    path: PathBuf,
    mmap: Option<Mmap>,            // Memory-mapped file
    size: u64,
}

impl FileLoader {
    pub async fn load(&mut self) -> Result<JsonTree, LoadError> {
        if self.size > MMAP_THRESHOLD {
            // Use memory mapping for large files
            self.mmap = Some(unsafe { 
                MmapOptions::new().map(&file)? 
            });
            // Parse directly from mmap
        } else {
            // Read entire file for small files
            let content = tokio::fs::read_to_string(&self.path).await?;
            // Parse from string
        }
    }
}

const MMAP_THRESHOLD: u64 = 10 * 1024 * 1024; // 10MB
```

## Application State Management

### Iced Application Structure

```rust
pub struct JsonViewerApp {
    // Core state
    current_file: Option<LoadedFile>,
    comparison_file: Option<LoadedFile>,
    
    // UI state
    view_mode: ViewMode,
    tree_state: TreeState,
    diff_state: Option<DiffState>,
    search_state: SearchState,
    
    // Settings
    preferences: Preferences,
    theme: Theme,
}

pub enum ViewMode {
    SingleFile,
    SideBySide,
    Diff,
}

pub struct LoadedFile {
    path: PathBuf,
    tree: Arc<JsonTree>,
    metadata: FileMetadata,
}
```

### Message System

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // File operations
    OpenFile,
    FileOpened(Result<LoadedFile, String>),
    CloseFile,
    ReloadFile,
    
    // Tree navigation
    NodeExpanded(usize),
    NodeCollapsed(usize),
    NodeSelected(usize),
    ScrollToNode(usize),
    
    // Search
    SearchQueryChanged(String),
    SearchSubmitted,
    SearchResultsReady(Vec<SearchResult>),
    NextSearchResult,
    PreviousSearchResult,
    
    // Formatting
    FormatRequested(FormatConfig),
    FormatCompleted(Result<String, String>),
    
    // Comparison
    StartComparison(PathBuf),
    ComparisonReady(DiffResult),
    NextDifference,
    PreviousDifference,
    
    // Settings
    ThemeChanged(Theme),
    PreferencesUpdated(Preferences),
}
```

## Performance Optimizations

### Memory Management

1. **Arena Allocation**: Use typed-arena for node allocation
2. **String Interning**: Deduplicate common strings (keys)
3. **Lazy Loading**: Load node children on-demand
4. **Smart Caching**: Cache recently accessed nodes

### Concurrency

1. **Async I/O**: All file operations async with tokio
2. **Parallel Parsing**: Use rayon for independent work
3. **Background Tasks**: Search and diff in separate threads
4. **Cancellation**: Support for interrupting long operations

### Rendering Optimization

1. **Virtual Scrolling**: Render only visible nodes
2. **Dirty Tracking**: Only re-render changed portions
3. **Layer Caching**: Cache static UI elements
4. **GPU Acceleration**: Let Iced handle GPU rendering

## Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum JsonViewerError {
    #[error("Failed to parse JSON: {0}")]
    ParseError(#[from] serde_json::Error),
    
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid RegEx pattern: {0}")]
    RegexError(#[from] regex::Error),
    
    #[error("File too large: {size} bytes (max: {max})")]
    FileTooLarge { size: u64, max: u64 },
    
    #[error("Out of memory")]
    OutOfMemory,
}

pub type Result<T> = std::result::Result<T, JsonViewerError>;
```

## Testing Strategy

### Unit Tests
- Test each module independently
- Mock file I/O for parser tests
- Property-based testing for diff algorithm

### Integration Tests
- Test full file loading pipeline
- Test UI interactions via Iced testing utilities
- Test cross-module communication

### Performance Tests
- Benchmark parsing speed (target: 2GB/s)
- Benchmark search speed (target: 50k results/s)
- Memory usage profiling
- UI responsiveness testing (60fps)

### Fuzz Testing
- Random JSON generation
- Malformed JSON handling
- Edge cases (empty files, huge nesting, etc.)

## Dependencies

### Core Dependencies
```toml
[dependencies]
iced = "0.14"                    # GUI framework (Dec 2025 release)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"               # JSON parsing
tokio = { version = "1.48", features = ["full"] }
anyhow = "1.0"                   # Error handling
thiserror = "2.0"                # Error derive macros

# Performance
rayon = "1.10"                   # Parallel processing
memmap2 = "0.9"                  # Memory-mapped files

# Search
regex = "1.11"                   # RegEx support

# Diff
similar = "2.6"                  # Diff algorithms

# File watching
notify = "7.0"                   # File system notifications (Phase 4)

# Syntax highlighting (optional)
# syntect = "5.2"                # Consider for future

# Utility
chrono = "0.4"                   # Timestamp handling
```

### Dev Dependencies
```toml
[dev-dependencies]
criterion = "0.5"                # Benchmarking
proptest = "1.6"                 # Property-based testing
pretty_assertions = "1.4"        # Better test output
```

**Note**: Versions current as of December 12, 2025. Check crates.io for latest releases.

## Build & Deployment

### Release Optimization

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true                     # Remove debug symbols
panic = "abort"                  # Smaller binary
```

### Platform-Specific Builds

- **Windows**: MSVC toolchain, MSI installer via WiX
- **macOS**: Apple Silicon + Intel (universal binary), DMG with codesigning
- **Linux**: AppImage for maximum compatibility

### CI/CD Pipeline

1. Automated testing on push
2. Clippy linting
3. Cargo fmt checking
4. Build artifacts for all platforms
5. Release automation with GitHub Actions

## Monitoring & Telemetry (Opt-in)

```rust
pub struct Telemetry {
    session_id: Uuid,
    events: Vec<Event>,
}

pub struct Event {
    timestamp: DateTime<Utc>,
    event_type: EventType,
    metadata: HashMap<String, String>,
}

pub enum EventType {
    FileOpened { size: u64 },
    ParseCompleted { duration_ms: u64 },
    SearchPerformed { results_count: usize },
    // ... etc
}
```

## Future Architecture Considerations

- **Plugin System**: WASM-based plugins for extensibility
- **LSP Integration**: Act as language server for editors
- **Remote Files**: Support for S3, HTTP, etc.
- **Collaborative Features**: Operational transform for real-time collab

---

**Document Version**: 0.1.0  
**Last Updated**: 2025-12-12  
**Status**: Draft
