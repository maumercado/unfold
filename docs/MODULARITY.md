# Unfold - Modularity & Extensibility Principles

## Overview

While Unfold v1.0 focuses exclusively on JSON, the architecture is designed with future format support in mind (TOML, YAML, XML, etc.). This document outlines the principles and patterns that enable this extensibility without overengineering the initial implementation.

## Core Principle: Build for JSON, Design for Extensibility

**What this means:**
- Write concrete JSON code now
- Extract abstractions when patterns emerge (not before)
- Keep interfaces clean and minimal
- Don't over-abstract prematurely

**What this doesn't mean:**
- Don't build a generic "format framework" upfront
- Don't create complex plugin systems yet
- Don't write code you don't need today

## Architectural Modularity

### 1. Separate Concerns into Crates (Workspace)

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "unfold-core",      # Core abstractions (minimal)
    "unfold-json",      # JSON-specific implementation
    "unfold-ui",        # Iced UI layer
    "unfold-cli",       # CLI binary
]

# Future (when needed):
# "unfold-toml",
# "unfold-yaml",
```

**Why**: Physical separation enforces clean boundaries. JSON code can't accidentally depend on future TOML code.

### 2. Core Abstractions (unfold-core)

Define **minimal** traits that represent universal concepts:

```rust
// unfold-core/src/tree.rs

/// Universal tree node - works for JSON, TOML, YAML, etc.
pub trait TreeNode: Send + Sync {
    /// Get the type of this node
    fn node_type(&self) -> NodeType;
    
    /// Get the display key (if any)
    fn key(&self) -> Option<&str>;
    
    /// Get the value as a string for display
    fn value_display(&self) -> String;
    
    /// Get children indices
    fn children(&self) -> &[usize];
    
    /// Get depth in tree
    fn depth(&self) -> usize;
    
    /// Is this node currently expanded?
    fn is_expanded(&self) -> bool;
    
    /// Toggle expansion state
    fn toggle_expand(&mut self);
}

/// Universal node types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
    // Future: Table, Inline, etc. for TOML
}

/// Universal tree interface
pub trait Tree: Send + Sync {
    type Node: TreeNode;
    
    /// Get root node index
    fn root_index(&self) -> usize;
    
    /// Get node by index
    fn get_node(&self, index: usize) -> Option<&Self::Node>;
    
    /// Get mutable node by index
    fn get_node_mut(&mut self, index: usize) -> Option<&mut Self::Node>;
    
    /// Get all visible nodes (respecting expand/collapse)
    fn visible_nodes(&self) -> Vec<usize>;
    
    /// Get tree statistics
    fn stats(&self) -> TreeStats;
    
    /// Get path to a node
    fn get_path(&self, index: usize) -> String;
}

/// Format-agnostic parser trait
pub trait Parser: Send + Sync {
    type Tree: Tree;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Parse content into a tree
    fn parse(&self, content: &str) -> Result<Self::Tree, Self::Error>;
    
    /// Format identifier (e.g., "json", "toml", "yaml")
    fn format_name(&self) -> &'static str;
}
```

**Key Points:**
- These traits are **minimal** - only what's truly universal
- JSON implementation provides the concrete types
- Future formats implement the same traits
- UI layer works with traits, not concrete types

### 3. JSON Implementation (unfold-json)

Concrete implementation for v1.0:

```rust
// unfold-json/src/lib.rs

use unfold_core::{Tree, TreeNode, Parser, NodeType};

/// JSON-specific tree
pub struct JsonTree {
    nodes: Vec<JsonNode>,
    root_index: usize,
    // ... JSON-specific fields
}

/// JSON-specific node
pub struct JsonNode {
    key: Option<Arc<str>>,
    value: JsonValue,
    node_type: NodeType,
    // ... JSON-specific fields
}

// Implement core traits
impl TreeNode for JsonNode {
    fn node_type(&self) -> NodeType {
        self.node_type
    }
    
    fn key(&self) -> Option<&str> {
        self.key.as_ref().map(|k| k.as_ref())
    }
    
    // ... implement other trait methods
}

impl Tree for JsonTree {
    type Node = JsonNode;
    
    // ... implement trait methods
}

/// JSON parser
pub struct JsonParser;

impl Parser for JsonParser {
    type Tree = JsonTree;
    type Error = JsonError;
    
    fn parse(&self, content: &str) -> Result<JsonTree, JsonError> {
        // Concrete JSON parsing logic
    }
    
    fn format_name(&self) -> &'static str {
        "json"
    }
}
```

**Key Points:**
- Full JSON implementation lives here
- All the code from IMPLEMENTATION.md goes here
- Uses serde_json, has JSON-specific optimizations
- Implements core traits for UI interoperability

### 4. UI Layer (unfold-ui)

Works with traits, not concrete types:

```rust
// unfold-ui/src/tree_view.rs

use unfold_core::{Tree, TreeNode};

/// Generic tree view widget
pub struct TreeView<T: Tree> {
    tree: Arc<T>,
    viewport_height: f32,
    // ... UI state
}

impl<T: Tree> TreeView<T> {
    pub fn new(tree: Arc<T>) -> Self {
        Self {
            tree,
            viewport_height: 600.0,
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        // Render using trait methods only
        let visible = self.tree.visible_nodes();
        
        for &index in &visible {
            if let Some(node) = self.tree.get_node(index) {
                // Use TreeNode trait methods
                let key = node.key().unwrap_or("");
                let value = node.value_display();
                // ... render
            }
        }
    }
}
```

**Key Points:**
- UI is generic over `Tree` trait
- Can render any format that implements the trait
- No JSON-specific code in UI layer

### 5. CLI Binary (unfold-cli)

Detects format and dispatches:

```rust
// unfold-cli/src/main.rs

use unfold_json::JsonParser;
use unfold_core::Parser;

fn main() {
    let path = parse_args();
    
    // v1.0: Just JSON
    let parser = JsonParser;
    let tree = parser.parse(&content)?;
    
    // Launch UI with parsed tree
    JsonViewerApp::run(tree);
}

// Future (v2.0):
// fn detect_format(path: &Path) -> Box<dyn Parser> {
//     match path.extension() {
//         "json" => Box::new(JsonParser),
//         "toml" => Box::new(TomlParser),
//         "yaml" | "yml" => Box::new(YamlParser),
//     }
// }
```

## Functional Programming Principles

### 1. Immutable Core Data

```rust
// DON'T: Mutate tree everywhere
impl TreeView {
    fn expand_node(&mut self, index: usize) {
        self.tree.get_node_mut(index).unwrap().is_expanded = true;
    }
}

// DO: Return new state, or use interior mutability carefully
impl TreeView {
    fn expand_node(&self, index: usize) -> Message {
        Message::ExpandNode(index)
    }
}

// Application handles state changes
impl Application {
    fn update(&mut self, message: Message) {
        match message {
            Message::ExpandNode(index) => {
                // Controlled mutation in one place
                if let Some(node) = self.tree.get_node_mut(index) {
                    node.toggle_expand();
                }
            }
        }
    }
}
```

### 2. Pure Functions

```rust
// Pure: Same input → same output, no side effects
pub fn calculate_visible_range(
    scroll_offset: f32,
    viewport_height: f32,
    node_height: f32,
    total_nodes: usize,
) -> Range<usize> {
    let start = (scroll_offset / node_height) as usize;
    let count = (viewport_height / node_height).ceil() as usize;
    let end = (start + count).min(total_nodes);
    start..end
}

// Use pure functions in methods
impl TreeView {
    fn visible_range(&self) -> Range<usize> {
        calculate_visible_range(
            self.scroll_offset,
            self.viewport_height,
            self.node_height,
            self.tree.visible_nodes().len(),
        )
    }
}
```

### 3. Composition Over Inheritance

```rust
// DON'T: Inherit behavior
// (Rust doesn't have inheritance anyway, but the principle applies)

// DO: Compose smaller pieces
pub struct SearchableTree<T: Tree> {
    tree: T,
    search_index: SearchIndex,
}

pub struct FormattableTree<T: Tree> {
    tree: T,
    formatter: Formatter,
}

// Compose features as needed
pub struct RichJsonTree {
    tree: JsonTree,
    search: SearchIndex,
    formatter: Formatter,
    diff: Option<DiffEngine>,
}
```

### 4. Higher-Order Functions

```rust
// Filter nodes by predicate
pub fn filter_nodes<T, F>(tree: &T, predicate: F) -> Vec<usize>
where
    T: Tree,
    F: Fn(&T::Node) -> bool,
{
    tree.visible_nodes()
        .into_iter()
        .filter(|&index| {
            tree.get_node(index)
                .map(|node| predicate(node))
                .unwrap_or(false)
        })
        .collect()
}

// Usage
let objects = filter_nodes(&tree, |node| {
    node.node_type() == NodeType::Object
});
```

### 5. Map/Reduce Patterns

```rust
// Transform tree nodes
pub fn map_nodes<T, F, R>(tree: &T, f: F) -> Vec<R>
where
    T: Tree,
    F: Fn(&T::Node) -> R,
{
    tree.visible_nodes()
        .iter()
        .filter_map(|&index| tree.get_node(index).map(&f))
        .collect()
}

// Aggregate tree information
pub fn fold_tree<T, F, Acc>(tree: &T, init: Acc, f: F) -> Acc
where
    T: Tree,
    F: Fn(Acc, &T::Node) -> Acc,
{
    tree.visible_nodes()
        .iter()
        .filter_map(|&index| tree.get_node(index))
        .fold(init, f)
}
```

## When to Add Abstractions

### ✅ Add Abstraction When:
1. You're implementing the **second format** (TOML/YAML)
2. You notice **exact duplication** between JSON code and new format
3. The abstraction is **simple and obvious**
4. It **reduces** complexity, not increases it

### ❌ Don't Add Abstraction When:
1. You're still on v1.0 (JSON only)
2. You're "planning ahead" for hypothetical future
3. The abstraction is complex or unclear
4. It makes the current code harder to understand

## Refactoring Path

### v1.0 - JSON Only (Now)
```
unfold/
├── src/
│   ├── main.rs           # CLI entry
│   ├── app.rs            # Iced app
│   ├── parser/           # JSON parsing
│   ├── ui/               # UI widgets
│   └── ...
└── Cargo.toml
```

**Focus**: Build excellent JSON viewer. Use concrete types everywhere.

### v1.5 - Prepare for Modularity (After v1.0 shipped)
```
unfold/
├── unfold-core/          # Extract minimal traits
│   └── src/
│       ├── tree.rs       # Tree trait
│       └── parser.rs     # Parser trait
├── unfold-json/          # Move JSON code here
│   └── src/
│       ├── tree.rs       # JsonTree
│       └── parser.rs     # JsonParser
├── unfold-ui/            # Make UI generic
└── unfold-cli/           # Main binary
```

**Focus**: Extract patterns that emerged. Make UI generic over traits.

### v2.0 - Add TOML Support
```
unfold/
├── unfold-core/
├── unfold-json/
├── unfold-toml/          # NEW: TOML implementation
│   └── src/
│       ├── tree.rs       # TomlTree
│       └── parser.rs     # TomlParser
├── unfold-ui/            # Already generic!
└── unfold-cli/           # Dispatch by format
```

**Focus**: Implement TOML using existing traits. UI Just Works™.

## Practical Example: Diff Module

### v1.0 - Concrete JSON Diff
```rust
// src/diff/json_diff.rs

pub struct JsonDiff {
    left: JsonTree,
    right: JsonTree,
}

impl JsonDiff {
    pub fn compare(&self) -> DiffResult {
        // JSON-specific comparison logic
    }
}
```

### v2.0 - When Adding TOML
```rust
// unfold-core/src/diff.rs

pub trait Differ {
    type Tree: Tree;
    
    fn compare(&self, left: &Self::Tree, right: &Self::Tree) -> DiffResult;
}

// unfold-json/src/diff.rs
pub struct JsonDiffer;

impl Differ for JsonDiffer {
    type Tree = JsonTree;
    
    fn compare(&self, left: &JsonTree, right: &JsonTree) -> DiffResult {
        // Same logic as before
    }
}

// unfold-toml/src/diff.rs
pub struct TomlDiffer;

impl Differ for TomlDiffer {
    type Tree = TomlTree;
    
    fn compare(&self, left: &TomlTree, right: &TomlTree) -> DiffResult {
        // TOML-specific logic (if needed)
        // Or use default implementation
    }
}
```

## Testing Strategy for Modularity

### Test Each Layer Independently

```rust
// unfold-json/tests/json_tree_tests.rs
#[test]
fn test_json_tree_creation() {
    let json = r#"{"key": "value"}"#;
    let parser = JsonParser;
    let tree = parser.parse(json).unwrap();
    // Test JSON-specific behavior
}

// unfold-core/tests/trait_tests.rs
fn test_tree_trait<T: Tree>(tree: T) {
    // Test that any Tree implementation works correctly
    assert!(tree.visible_nodes().len() > 0);
    let root = tree.get_node(tree.root_index()).unwrap();
    assert_eq!(root.depth(), 0);
}

#[test]
fn test_json_implements_tree_trait() {
    let tree = create_test_json_tree();
    test_tree_trait(tree);
}
```

## Documentation Strategy

### v1.0 Documentation
- Focus on JSON use cases
- Concrete examples
- Performance characteristics
- No mention of future formats (avoid vaporware)

### v2.0 Documentation
- Add "Supported Formats" section
- Show format-agnostic examples
- Document trait system for contributors
- Migration guide from v1.0

## Key Takeaways

1. **Build Concrete First**: Write JSON-specific code for v1.0
2. **Extract Patterns Later**: Only abstract when you see duplication
3. **Minimal Traits**: Keep trait surface area small
4. **Physical Separation**: Use workspace crates to enforce boundaries
5. **Functional Style**: Pure functions, composition, immutability
6. **Test Independence**: Each module tests independently
7. **No Premature Abstraction**: YAGNI (You Aren't Gonna Need It)

## Decision Framework

When writing code, ask:

1. **Does this need to be generic?** (Usually no for v1.0)
2. **Can I make this a pure function?** (Usually yes)
3. **Am I mutating shared state?** (Try to avoid)
4. **Is this composable?** (Small, focused functions)
5. **Would this be easy to test?** (If no, refactor)

---

**Remember**: The goal is to write excellent JSON code that *happens* to be structured in a way that makes future extension natural. Not to build a framework upfront.

**Document Version**: 0.1.0  
**Last Updated**: 2025-12-12  
**Status**: Draft
