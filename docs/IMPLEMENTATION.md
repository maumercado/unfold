# Unfold - Implementation Guide

## Getting Started

### Project Structure

```
unfold/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── src/
│   ├── main.rs
│   ├── app.rs                 # Main application state
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── json_tree.rs       # Tree data structure
│   │   ├── streaming_parser.rs
│   │   └── validator.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── tree_view.rs       # Tree widget
│   │   ├── diff_view.rs       # Diff widget
│   │   ├── search_panel.rs    # Search UI
│   │   └── components/        # Reusable UI components
│   ├── diff/
│   │   ├── mod.rs
│   │   ├── structural_diff.rs
│   │   └── algorithms.rs
│   ├── search/
│   │   ├── mod.rs
│   │   ├── text_search.rs
│   │   └── regex_search.rs
│   ├── format/
│   │   ├── mod.rs
│   │   ├── pretty_printer.rs
│   │   └── minifier.rs
│   ├── file/
│   │   ├── mod.rs
│   │   ├── loader.rs
│   │   └── watcher.rs
│   └── utils/
│       ├── mod.rs
│       ├── error.rs
│       └── config.rs
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/              # Test JSON files
└── benches/
    ├── parser_bench.rs
    └── search_bench.rs
```

### Initial Cargo.toml

```toml
[package]
name = "unfold"
version = "0.1.0"
edition = "2021"
authors = ["Mau <your-email@example.com>"]
description = "High-performance JSON viewer with diff and formatting capabilities"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/unfold"

[dependencies]
# GUI Framework
iced = { version = "0.14", features = ["tokio"] }

# Async runtime
tokio = { version = "1.48", features = ["full"] }

# JSON processing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Performance
rayon = "1.10"              # Parallel processing
memmap2 = "0.9"             # Memory-mapped files

# Search
regex = "1.11"              # RegEx support

# Diff (when implementing comparison)
similar = "2.6"             # Text/structural diffing

# File watching (Phase 4)
# notify = "7.0"            # Uncomment when needed

# Utilities
chrono = "0.4"              # Timestamp handling

[dev-dependencies]
criterion = "0.5"
proptest = "1.6"
pretty_assertions = "1.4"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[[bench]]
name = "parser_bench"
harness = false
```

**Note**: As of December 12, 2025, these are the latest stable versions. Always check crates.io for the most current releases.

## Core Implementation Examples

### 1. JSON Tree Structure (src/parser/json_tree.rs)

```rust
use serde_json::Value;
use std::sync::Arc;

/// Represents the type of a JSON node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

/// A single node in the JSON tree
#[derive(Debug, Clone)]
pub struct JsonNode {
    /// Optional key (for object properties)
    pub key: Option<Arc<str>>,
    
    /// The value of this node
    pub value: JsonValue,
    
    /// Type of this node
    pub node_type: JsonType,
    
    /// Depth in the tree (0 for root)
    pub depth: usize,
    
    /// Byte offset in the original file
    pub offset: usize,
    
    /// Indices of child nodes in the tree's node vector
    pub children: Vec<usize>,
    
    /// Whether this node is currently expanded in the UI
    pub is_expanded: bool,
}

/// Efficient value representation
#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(Arc<str>),
    Array(usize),     // Length only
    Object(usize),    // Size only
}

/// The complete JSON tree
#[derive(Debug)]
pub struct JsonTree {
    /// All nodes in a flat array for cache efficiency
    nodes: Vec<JsonNode>,
    
    /// Index of the root node
    root_index: usize,
    
    /// Original file size
    file_size: u64,
    
    /// Total number of nodes
    node_count: usize,
    
    /// Maximum depth of the tree
    max_depth: usize,
    
    /// String interner for deduplication
    strings: StringInterner,
}

/// Simple string interning for memory efficiency
#[derive(Debug, Default)]
struct StringInterner {
    strings: std::collections::HashMap<String, Arc<str>>,
}

impl StringInterner {
    fn intern(&mut self, s: String) -> Arc<str> {
        if let Some(interned) = self.strings.get(&s) {
            Arc::clone(interned)
        } else {
            let arc: Arc<str> = Arc::from(s.as_str());
            self.strings.insert(s, Arc::clone(&arc));
            arc
        }
    }
}

impl JsonTree {
    /// Create a new empty tree
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_index: 0,
            file_size: 0,
            node_count: 0,
            max_depth: 0,
            strings: StringInterner::default(),
        }
    }
    
    /// Build tree from serde_json Value
    pub fn from_value(value: Value, file_size: u64) -> Self {
        let mut tree = Self::new();
        tree.file_size = file_size;
        
        let root_index = tree.build_node(None, &value, 0, 0);
        tree.root_index = root_index;
        tree.node_count = tree.nodes.len();
        
        tree
    }
    
    /// Recursively build nodes
    fn build_node(
        &mut self,
        key: Option<String>,
        value: &Value,
        depth: usize,
        offset: usize,
    ) -> usize {
        self.max_depth = self.max_depth.max(depth);
        
        let node_index = self.nodes.len();
        let key = key.map(|k| self.strings.intern(k));
        
        let (node_type, json_value, children) = match value {
            Value::Null => (JsonType::Null, JsonValue::Null, vec![]),
            
            Value::Bool(b) => (JsonType::Bool, JsonValue::Bool(*b), vec![]),
            
            Value::Number(n) => {
                let f = n.as_f64().unwrap_or(0.0);
                (JsonType::Number, JsonValue::Number(f), vec![])
            }
            
            Value::String(s) => {
                let interned = self.strings.intern(s.clone());
                (JsonType::String, JsonValue::String(interned), vec![])
            }
            
            Value::Array(arr) => {
                let mut child_indices = Vec::with_capacity(arr.len());
                for (i, item) in arr.iter().enumerate() {
                    let child_index = self.build_node(
                        Some(format!("[{}]", i)),
                        item,
                        depth + 1,
                        offset, // TODO: track actual offsets
                    );
                    child_indices.push(child_index);
                }
                (JsonType::Array, JsonValue::Array(arr.len()), child_indices)
            }
            
            Value::Object(obj) => {
                let mut child_indices = Vec::with_capacity(obj.len());
                for (k, v) in obj.iter() {
                    let child_index = self.build_node(
                        Some(k.clone()),
                        v,
                        depth + 1,
                        offset,
                    );
                    child_indices.push(child_index);
                }
                (JsonType::Object, JsonValue::Object(obj.len()), child_indices)
            }
        };
        
        self.nodes.push(JsonNode {
            key,
            value: json_value,
            node_type,
            depth,
            offset,
            children,
            is_expanded: false,
        });
        
        node_index
    }
    
    /// Get a node by index
    pub fn get_node(&self, index: usize) -> Option<&JsonNode> {
        self.nodes.get(index)
    }
    
    /// Get a mutable node by index
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut JsonNode> {
        self.nodes.get_mut(index)
    }
    
    /// Get the root node
    pub fn root(&self) -> &JsonNode {
        &self.nodes[self.root_index]
    }
    
    /// Get all nodes (for iteration)
    pub fn nodes(&self) -> &[JsonNode] {
        &self.nodes
    }
    
    /// Get tree statistics
    pub fn stats(&self) -> TreeStats {
        TreeStats {
            file_size: self.file_size,
            node_count: self.node_count,
            max_depth: self.max_depth,
        }
    }
    
    /// Get flattened visible nodes (for rendering)
    pub fn visible_nodes(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        self.collect_visible_nodes(self.root_index, &mut visible);
        visible
    }
    
    fn collect_visible_nodes(&self, index: usize, visible: &mut Vec<usize>) {
        visible.push(index);
        
        if let Some(node) = self.get_node(index) {
            if node.is_expanded {
                for &child_index in &node.children {
                    self.collect_visible_nodes(child_index, visible);
                }
            }
        }
    }
    
    /// Toggle expand/collapse for a node
    pub fn toggle_expand(&mut self, index: usize) {
        if let Some(node) = self.get_node_mut(index) {
            node.is_expanded = !node.is_expanded;
        }
    }
    
    /// Get the JSON path to a node
    pub fn get_path(&self, index: usize) -> String {
        let mut path_parts = Vec::new();
        let mut current = index;
        
        while let Some(node) = self.get_node(current) {
            if let Some(key) = &node.key {
                path_parts.push(key.to_string());
            }
            
            // Find parent (inefficient, should cache parent indices)
            if let Some((parent_idx, _)) = self.nodes.iter().enumerate()
                .find(|(_, n)| n.children.contains(&current)) {
                current = parent_idx;
            } else {
                break; // Reached root
            }
        }
        
        path_parts.reverse();
        format!("root.{}", path_parts.join("."))
    }
}

#[derive(Debug, Clone)]
pub struct TreeStats {
    pub file_size: u64,
    pub node_count: usize,
    pub max_depth: usize,
}
```

### 2. Virtual Scroller (src/ui/tree_view.rs - excerpt)

```rust
use iced::widget::{Column, Container, Scrollable};
use iced::{Element, Length, Theme};

pub struct TreeView {
    tree: Arc<JsonTree>,
    viewport_height: f32,
    node_height: f32,
    scroll_offset: f32,
    selected_node: Option<usize>,
}

impl TreeView {
    pub fn new(tree: Arc<JsonTree>) -> Self {
        Self {
            tree,
            viewport_height: 600.0,
            node_height: 24.0,
            scroll_offset: 0.0,
            selected_node: None,
        }
    }
    
    /// Calculate which nodes are visible in the viewport
    fn visible_range(&self, visible_nodes: &[usize]) -> std::ops::Range<usize> {
        let start_index = (self.scroll_offset / self.node_height) as usize;
        let visible_count = (self.viewport_height / self.node_height).ceil() as usize;
        let end_index = (start_index + visible_count).min(visible_nodes.len());
        
        // Add buffer for smooth scrolling
        let buffer = 10;
        let buffered_start = start_index.saturating_sub(buffer);
        let buffered_end = (end_index + buffer).min(visible_nodes.len());
        
        buffered_start..buffered_end
    }
    
    /// Render the tree view
    pub fn view(&self) -> Element<Message> {
        let visible_nodes = self.tree.visible_nodes();
        let range = self.visible_range(&visible_nodes);
        
        let mut column = Column::new().spacing(0);
        
        // Add spacer for nodes above viewport
        if range.start > 0 {
            let spacer_height = range.start as f32 * self.node_height;
            column = column.push(
                Container::new(iced::widget::Space::new(Length::Fill, spacer_height))
            );
        }
        
        // Render visible nodes
        for &node_index in &visible_nodes[range.clone()] {
            if let Some(node) = self.tree.get_node(node_index) {
                column = column.push(self.render_node(node_index, node));
            }
        }
        
        // Add spacer for nodes below viewport
        if range.end < visible_nodes.len() {
            let spacer_height = (visible_nodes.len() - range.end) as f32 * self.node_height;
            column = column.push(
                Container::new(iced::widget::Space::new(Length::Fill, spacer_height))
            );
        }
        
        Scrollable::new(column)
            .height(Length::Fill)
            .into()
    }
    
    fn render_node(&self, index: usize, node: &JsonNode) -> Element<Message> {
        // Indentation based on depth
        let indent = node.depth * 20;
        
        // Render node content (simplified)
        let content = format!(
            "{}{}: {}",
            " ".repeat(indent),
            node.key.as_ref().map(|k| k.as_ref()).unwrap_or(""),
            self.value_display(&node.value)
        );
        
        // Create clickable node element
        // (Actual implementation would use proper Iced widgets)
        iced::widget::text(content).into()
    }
    
    fn value_display(&self, value: &JsonValue) -> String {
        match value {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Array(len) => format!("[{} items]", len),
            JsonValue::Object(size) => format!("{{{} properties}}", size),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    NodeClicked(usize),
    NodeExpanded(usize),
    Scrolled(f32),
}
```

### 3. Async File Loading (src/file/loader.rs)

```rust
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

pub struct FileLoader;

impl FileLoader {
    /// Load a JSON file asynchronously
    pub async fn load(path: PathBuf) -> Result<(String, u64)> {
        // Check file exists
        let metadata = fs::metadata(&path)
            .await
            .context("Failed to read file metadata")?;
        
        let file_size = metadata.len();
        
        // For small files, read directly
        if file_size < 10 * 1024 * 1024 {
            // < 10MB
            let content = fs::read_to_string(&path)
                .await
                .context("Failed to read file contents")?;
            
            Ok((content, file_size))
        } else {
            // For large files, use memory mapping
            // (Simplified - actual implementation would use memmap2)
            let content = fs::read_to_string(&path)
                .await
                .context("Failed to read large file")?;
            
            Ok((content, file_size))
        }
    }
    
    /// Parse JSON string into tree
    pub fn parse(json_str: &str, file_size: u64) -> Result<JsonTree> {
        let value: serde_json::Value = serde_json::from_str(json_str)
            .context("Failed to parse JSON")?;
        
        Ok(JsonTree::from_value(value, file_size))
    }
    
    /// Load and parse in one step
    pub async fn load_and_parse(path: PathBuf) -> Result<JsonTree> {
        let (content, file_size) = Self::load(path).await?;
        Self::parse(&content, file_size)
    }
}
```

### 4. Main Application (src/app.rs - basic structure)

```rust
use iced::{Application, Command, Element, Settings, Theme};
use std::sync::Arc;

pub struct JsonViewerApp {
    current_file: Option<LoadedFile>,
    view_mode: ViewMode,
    tree_view: Option<TreeView>,
}

pub struct LoadedFile {
    path: PathBuf,
    tree: Arc<JsonTree>,
}

pub enum ViewMode {
    TreeView,
    DiffView,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFilePressed,
    FileOpened(Result<LoadedFile, String>),
    TreeViewMessage(tree_view::Message),
}

impl Application for JsonViewerApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                current_file: None,
                view_mode: ViewMode::TreeView,
                tree_view: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("JSON Viewer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OpenFilePressed => {
                // Open file dialog
                Command::perform(
                    async { Self::open_file_dialog().await },
                    Message::FileOpened,
                )
            }
            
            Message::FileOpened(result) => {
                match result {
                    Ok(loaded_file) => {
                        self.tree_view = Some(TreeView::new(Arc::clone(&loaded_file.tree)));
                        self.current_file = Some(loaded_file);
                    }
                    Err(e) => {
                        eprintln!("Failed to open file: {}", e);
                    }
                }
                Command::none()
            }
            
            Message::TreeViewMessage(_msg) => {
                // Handle tree view messages
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if let Some(tree_view) = &self.tree_view {
            tree_view.view().map(Message::TreeViewMessage)
        } else {
            // Show welcome screen
            iced::widget::text("Open a JSON file to get started").into()
        }
    }
}

impl JsonViewerApp {
    async fn open_file_dialog() -> Result<LoadedFile, String> {
        // Use rfd crate for file dialog
        // (Simplified - actual implementation would use rfd)
        let path = PathBuf::from("example.json");
        
        let tree = FileLoader::load_and_parse(path.clone())
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(LoadedFile {
            path,
            tree: Arc::new(tree),
        })
    }
}
```

## Best Practices

### 1. Error Handling

```rust
// Use thiserror for custom errors
#[derive(Debug, thiserror::Error)]
pub enum JsonViewerError {
    #[error("Failed to parse JSON: {0}")]
    ParseError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: u64, max: u64 },
}

// Use Result type alias
pub type Result<T> = std::result::Result<T, JsonViewerError>;
```

### 2. Performance Profiling

```rust
// Add criterion benchmarks
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_large_json(c: &mut Criterion) {
    let json_str = include_str!("../tests/fixtures/large.json");
    
    c.bench_function("parse_large_json", |b| {
        b.iter(|| {
            let value: serde_json::Value = serde_json::from_str(black_box(json_str)).unwrap();
            JsonTree::from_value(value, json_str.len() as u64)
        });
    });
}

criterion_group!(benches, bench_parse_large_json);
criterion_main!(benches);
```

### 3. Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"name": "test", "value": 42}"#;
        let tree = JsonTree::from_value(
            serde_json::from_str(json).unwrap(),
            json.len() as u64,
        );
        
        assert_eq!(tree.node_count, 3); // root + 2 properties
        assert_eq!(tree.max_depth, 1);
    }
    
    #[tokio::test]
    async fn test_load_file() {
        let result = FileLoader::load_and_parse(
            PathBuf::from("tests/fixtures/sample.json")
        ).await;
        
        assert!(result.is_ok());
    }
}
```

## Next Steps

1. Set up the project structure
2. Implement core `JsonTree` data structure
3. Create basic Iced application shell
4. Implement file loading
5. Build simple tree view (no virtual scrolling yet)
6. Test with small JSON files
7. Iterate and add features incrementally

---

**Document Version**: 0.1.0  
**Last Updated**: 2025-12-12  
**Status**: Draft
