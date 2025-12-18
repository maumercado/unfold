# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Unfold** is a high-performance JSON viewer built in Rust with the Iced GUI framework. The project aims to handle multi-gigabyte JSON files with native performance, efficient memory usage (1:1 file-to-RAM ratio target), and developer-focused features like diffing and formatting.

**Current Status**: Planning phase - documentation complete, implementation not yet started.

## 🎓 IMPORTANT: Learning-Focused Development

**The developer is a Rust novice learning Rust through this project.**

### Teaching Approach - ALWAYS:
1. **Explain BEFORE coding** - Describe what we're about to do and why
2. **Teach concepts** - Explain Rust concepts as they come up (ownership, borrowing, traits, lifetimes, etc.)
3. **Library explanations** - When adding dependencies, explain what they do, why we chose them, how they work
4. **Incremental learning** - Break complex topics into small, digestible pieces
5. **Guide, don't build** - Encourage the developer to write code with guidance, rather than providing complete solutions
6. **Check understanding** - Ask questions to ensure concepts are clear
7. **Connect to prior knowledge** - Relate Rust concepts to other languages when helpful
8. **Celebrate mistakes** - Errors are learning opportunities; explain compiler messages
9. **Provide resources** - Suggest relevant chapters from Rust Book, articles, or examples for deeper learning
10. **Be patient and thorough** - Take time to explain "why" not just "what"

### What NOT to do:
- Don't write large amounts of code without explanation
- Don't assume Rust knowledge (ownership, traits, lifetimes, etc.)
- Don't skip over "obvious" concepts - they may not be obvious to a Rust beginner
- Don't rush through library choices without explaining alternatives
- Don't use advanced Rust features without teaching them first

### Test-Driven Development:
**ALWAYS write tests alongside code.**
- Explain what we're testing and why
- Teach Rust's testing conventions (`#[test]`, `#[cfg(test)]`)
- Show how to run tests (`cargo test`)
- Use tests to demonstrate behavior
- Write tests BEFORE or DURING implementation, not after

**Target Performance Goals**:
- Load 100MB files in < 2 seconds
- Memory usage ~1.2x file size
- 60 FPS scrolling
- Search speed > 50k results/sec

## Development Commands

### Initial Project Setup (When Starting Implementation)
```bash
# Initialize Rust project
cargo new unfold
cd unfold

# Add core dependencies (versions as of Dec 2025)
cargo add iced@0.14 --features tokio
cargo add serde@1.0 --features derive
cargo add serde_json@1.0
cargo add tokio@1.48 --features full
cargo add anyhow@1.0
cargo add thiserror@2.0
cargo add rayon@1.10
cargo add memmap2@0.9
cargo add regex@1.11

# Later phases (diff and file watching)
cargo add similar@2.6    # Phase 3: Diff functionality
cargo add notify@7.0     # Phase 4: File watching
```

### Common Development Commands
```bash
# Run in development mode
cargo run

# Run with release optimizations (faster)
cargo run --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint code
cargo clippy

# Type-check without building
cargo check
```

## Architecture & Design Principles

### Core Architecture Layers

1. **Parser Module** (`parser/`)
   - Streaming JSON parser using `serde_json::StreamDeserializer`
   - Lazy-loading child nodes for memory efficiency
   - Flat `Vec<JsonNode>` storage with index-based references (better cache locality)
   - String interning via `Arc<str>` for duplicate keys/values (30-50% memory savings)

2. **Tree View Widget** (`ui/tree_view/`)
   - Custom virtual scrolling (render only visible nodes + buffer)
   - Virtual scrolling formula: `visible_range = (scroll_offset / node_height) ± buffer`
   - Tree state tracking (expanded/collapsed, selection, scroll position)
   - Pre-render buffer to ensure smooth 60 FPS scrolling

3. **Diff Engine** (`diff/`)
   - Structural (semantic) diff, not text-based
   - Understands JSON structure, can ignore whitespace/order
   - Multiple comparison modes: Strict, Semantic, Flexible, Schema
   - Uses `similar` crate for LCS algorithm

4. **Search Engine** (`search/`)
   - Phase 1: Linear scan (simpler, fast enough for most cases)
   - Phase 2: Optional inverted index for repeated searches
   - Both text and RegEx search support
   - Background thread execution to avoid UI blocking

5. **File Handler** (`file/`)
   - Memory-mapped files (memmap2) for files > 10MB
   - Direct file reading for smaller files
   - Async I/O using Tokio
   - File watching with debouncing (Phase 4)

### Key Data Structures

```rust
// Flat array storage for cache efficiency
pub struct JsonTree {
    nodes: Vec<JsonNode>,       // Flat array of all nodes
    root_index: usize,
    file_size: u64,
    node_count: usize,
    max_depth: usize,
}

// Index-based references (not pointers)
pub struct JsonNode {
    pub key: Option<Arc<str>>,      // String interning
    pub value: JsonValue,
    pub node_type: JsonType,
    pub depth: usize,
    pub children: Option<Vec<usize>>, // Indices, not pointers
}
```

### Application State (Iced)

```rust
pub struct JsonViewerApp {
    current_file: Option<LoadedFile>,
    comparison_file: Option<LoadedFile>,
    view_mode: ViewMode,          // SingleFile, SideBySide, Diff
    tree_state: TreeState,
    search_state: SearchState,
    preferences: Preferences,
    theme: Theme,
}
```

### Message-Driven Architecture

Iced uses Elm-inspired architecture with immutable messages:
- File operations (OpenFile, FileOpened, CloseFile)
- Tree navigation (NodeExpanded, NodeCollapsed, NodeSelected)
- Search (SearchQueryChanged, SearchSubmitted, SearchResultsReady)
- Formatting (FormatRequested, FormatCompleted)
- Comparison (StartComparison, ComparisonReady)

## Design Decisions & Trade-offs

### Why Iced (not Tauri)?
- Native performance for rendering millions of nodes
- Direct access to data structures without serialization
- Lower memory overhead (no web engine)
- **Trade-off**: Less familiar than web tech, smaller ecosystem, but maximum performance

### Why Flat Array (not Tree Pointers)?
- Better cache locality (contiguous memory)
- No lifetime annotations needed
- Predictable memory usage
- **Trade-off**: Slight indirection cost, must validate indices

### Why Streaming Parser?
- Handle arbitrarily large files
- Faster time-to-first-render
- Memory scales with visible nodes, not file size
- **Trade-off**: More complex implementation

### Why Structural Diff (not Text-based)?
- Understands JSON structure, not just text
- Can ignore cosmetic changes (whitespace, order)
- Path-based diff reporting
- **Trade-off**: More complex, slower for very large files

## Performance Optimization Strategies

### Memory Management
1. **Arena Allocation**: Use typed-arena for node allocation
2. **String Interning**: Deduplicate common strings (keys)
3. **Lazy Loading**: Load node children on-demand
4. **Smart Caching**: Cache recently accessed nodes

### Concurrency
1. **Async I/O**: All file operations async with Tokio
2. **Parallel Parsing**: Use rayon for independent work
3. **Background Tasks**: Search and diff in separate threads
4. **Cancellation**: Support interrupting long operations

### Rendering
1. **Virtual Scrolling**: Render only visible nodes
2. **Dirty Tracking**: Only re-render changed portions
3. **Layer Caching**: Cache static UI elements
4. **GPU Acceleration**: Iced handles GPU rendering

## Modularity & Future Extensibility

### v1.0 Approach: Concrete JSON Implementation
- Write JSON-specific code first
- Don't over-abstract prematurely
- Physical separation into modules, but not traits yet

### Post-v1.0 Refactoring Path
When adding TOML/YAML support (v2.0):
1. Extract minimal core traits (`Tree`, `TreeNode`, `Parser`)
2. Move JSON code to `unfold-json` crate
3. Make UI generic over traits
4. Add new format crates that implement same traits

### Workspace Structure (Future)
```
unfold/
├── unfold-core/      # Core abstractions (minimal traits)
├── unfold-json/      # JSON implementation
├── unfold-ui/        # Iced GUI (generic over traits)
├── unfold-cli/       # CLI binary
└── unfold-toml/      # Future: TOML support
```

## Phased Development Timeline

### Phase 1: Core Viewer (Weeks 1-2) - MVP
- Parse and display JSON files (up to 2GB)
- Tree view with expand/collapse
- Virtual scrolling
- Basic text search
- **Success Criteria**: View 100MB+ files smoothly

### Phase 2: Advanced Features (Weeks 3-4)
- JSON formatting (pretty print, minify, custom indent)
- RegEx search
- Multiple file tabs
- Dark/light theme toggle

### Phase 3: Comparison Engine (Weeks 5-6)
- Structural diff algorithm
- Side-by-side comparison view
- Diff navigation
- Export diff reports

### Phase 4: Power Features (Weeks 7-8)
- JSON-Lines / ndjson support
- File watching and auto-refresh
- JSON path filtering
- Multi-file merge

### Phase 5: Polish & Release (Weeks 9-12)
- Comprehensive testing and benchmarking
- Documentation
- Cross-platform installers
- Public v1.0.0 release

## Coding Guidelines

### Performance Best Practices

**DO:**
```rust
// Pre-allocate with capacity
let mut nodes = Vec::with_capacity(1000);

// Use references when possible
fn process_node(node: &JsonNode) { }

// Clone Arc pointers, not data
let shared = Arc::clone(&self.tree);

// Use iterators (lazy evaluation)
nodes.iter().filter(|n| n.depth > 5).count()
```

**DON'T:**
```rust
// Allocate in hot loops
for _ in 0..1000 { let v = Vec::new(); }

// Unnecessary clones
fn process_node(node: JsonNode) { } // Clones!

// Collect when not needed
nodes.iter().collect::<Vec<_>>().len() // Use .count()
```

### Functional Programming Style

1. **Pure Functions**: Same input → same output, no side effects
2. **Immutable Core Data**: Minimize mutation, use message passing
3. **Composition Over Inheritance**: Build features by composing smaller pieces
4. **Higher-Order Functions**: Use map/filter/fold patterns

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum JsonViewerError {
    #[error("Failed to parse JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File too large: {size} bytes (max: {max})")]
    FileTooLarge { size: u64, max: u64 },
}
```

## Testing Strategy

### Unit Tests
- Test each module independently
- Mock file I/O for parser tests
- Property-based testing (proptest) for diff algorithm

### Integration Tests
- Test full file loading pipeline
- Test cross-module communication

### Performance Tests
- Benchmark parsing speed (target: 2GB/s)
- Benchmark search speed (target: 50k results/s)
- Memory usage profiling
- UI responsiveness (60fps)

### Benchmark Template
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse(c: &mut Criterion) {
    let json = include_str!("../tests/fixtures/large.json");
    c.bench_function("parse_100mb", |b| {
        b.iter(|| parse_json(black_box(json)))
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

## Features Explicitly Excluded from v1.0

1. **JSON Editing** - Read-only viewer only (maybe v1.1+)
2. **Cloud Integration** - Local-first approach
3. **Real-time Collaboration** - Too complex, not core use case
4. **Plugin System** - Can add features directly for now (maybe WASM plugins in v2.0)

## Release Configuration

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true          # Remove debug symbols
panic = "abort"       # Smaller binary
```

## Important Principles

1. **Performance First**: Every decision considers impact on speed and memory
2. **Build for JSON, Design for Extensibility**: Write concrete code now, extract abstractions later
3. **No Premature Abstraction**: YAGNI (You Aren't Gonna Need It) - only abstract when you see duplication
4. **MVP Mindset**: Ship v1.0, iterate based on feedback
5. **Privacy-Focused**: Local-first, no telemetry by default

## Essential Documentation

- **PROJECT_SPEC.md**: High-level requirements, features, goals
- **ARCHITECTURE.md**: Technical architecture, data structures, system design
- **ROADMAP.md**: Timeline, phases, milestones, task breakdown
- **IMPLEMENTATION.md**: Code examples, best practices, implementation guide
- **DESIGN_DECISIONS.md**: Key decisions, trade-offs, rationale
- **MODULARITY.md**: Extensibility principles for future format support
- **QUICK_REFERENCE.md**: One-page cheat sheet for rapid development
