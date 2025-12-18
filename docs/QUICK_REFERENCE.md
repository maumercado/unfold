# Unfold - Quick Reference Guide

> One-page cheat sheet for rapid development

## 🚀 Quick Commands

```bash
# Create new Rust project
cargo new unfold
cd unfold

# Add dependencies with latest versions (December 2025)
cargo add iced@0.14 --features tokio
cargo add serde@1.0 --features derive
cargo add serde_json@1.0
cargo add tokio@1.48 --features full
cargo add anyhow@1.0 
cargo add thiserror@2.0
cargo add rayon@1.10
cargo add memmap2@0.9
cargo add regex@1.11

# Add these later as needed
# cargo add similar@2.6    # For diff functionality
# cargo add notify@7.0     # For file watching

# Run development build
cargo run

# Run with optimizations (faster)
cargo run --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check
```

## 📁 File Templates

### Minimal main.rs

```rust
use iced::{Application, Settings};

mod app;
use app::JsonViewerApp;

fn main() -> iced::Result {
    JsonViewerApp::run(Settings::default())
}
```

### Error Type Template

```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, MyError>;
```

### Iced Message Enum Template

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // File operations
    OpenFile,
    FileOpened(Result<PathBuf, String>),
    
    // UI interactions
    NodeClicked(usize),
    SearchChanged(String),
    
    // Async results
    TaskCompleted(Result<Data, String>),
}
```

### Async Command Template

```rust
Command::perform(
    async move {
        // Your async code here
        let result = load_file(path).await;
        result
    },
    |result| Message::FileOpened(result),
)
```

## 🎨 Iced Widget Cheatsheet

```rust
use iced::widget::{
    button, column, container, row, scrollable,
    text, text_input, Space,
};

// Column (vertical stack)
column![
    text("Title"),
    button("Click me"),
    Space::with_height(20),
]
.spacing(10)

// Row (horizontal stack)
row![
    text("Label:"),
    text_input("placeholder", &value),
]
.spacing(5)

// Scrollable area
scrollable(column![/* content */])
    .height(Length::Fill)

// Container (wrapping)
container(text("Centered"))
    .center_x()
    .center_y()
    .padding(20)
```

## 🔍 Common Patterns

### Safe Index Access

```rust
// DON'T: Panic on invalid index
let node = &self.nodes[index];

// DO: Handle gracefully
if let Some(node) = self.nodes.get(index) {
    // Use node
} else {
    // Handle error
}
```

### Cloning Arc

```rust
// DON'T: Clone inner data
let tree_copy = (*self.tree).clone();

// DO: Clone Arc pointer (cheap)
let tree_ref = Arc::clone(&self.tree);
```

### String Interning

```rust
// For duplicate strings
let interned: Arc<str> = Arc::from(string.as_str());

// Later, check equality by pointer
if Arc::ptr_eq(&str1, &str2) {
    // Same string!
}
```

### Async in Iced

```rust
// In update() method
Command::perform(
    async_function(),
    Message::AsyncComplete,
)

// async_function must return Send + 'static
async fn async_function() -> Result<Data> {
    // ...
}
```

## 📊 Performance Tips

### DO

```rust
// ✅ Pre-allocate with capacity
let mut nodes = Vec::with_capacity(1000);

// ✅ Use references when possible
fn process_node(node: &JsonNode) { }

// ✅ Clone only Arc, not data
let shared = Arc::clone(&self.tree);

// ✅ Use iterators (lazy)
nodes.iter().filter(|n| n.depth > 5)
```

### DON'T

```rust
// ❌ Allocate in hot loops
for _ in 0..1000 {
    let v = Vec::new(); // Bad!
}

// ❌ Unnecessary clones
fn process_node(node: JsonNode) { } // Clones!

// ❌ Collect when not needed
let count = nodes.iter()
    .filter(|n| n.depth > 5)
    .collect::<Vec<_>>() // Unnecessary!
    .len();

// ✅ Better
let count = nodes.iter()
    .filter(|n| n.depth > 5)
    .count();
```

## 🧪 Testing Shortcuts

```rust
// Unit test template
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() {
        let result = my_function();
        assert_eq!(result, expected);
    }
    
    #[tokio::test]
    async fn test_async() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}

// Property-based testing
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_with_random_input(s in "\\PC*") {
        // Test with random strings
    }
}
```

## 🐛 Debug Helpers

```rust
// Print type of variable
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

// Debug print with location
dbg!(&my_variable);

// Conditional compilation
#[cfg(debug_assertions)]
println!("Debug mode!");

// Pretty-print JSON
let json = serde_json::to_string_pretty(&value)?;
```

## 📐 Math Helpers

```rust
// Virtual scrolling
let visible_start = (scroll_offset / node_height) as usize;
let visible_count = (viewport_height / node_height).ceil() as usize;
let visible_end = (visible_start + visible_count).min(total);

// Add buffer
let buffer = 10;
let start = visible_start.saturating_sub(buffer);
let end = (visible_end + buffer).min(total);

// Clamp value
let clamped = value.max(min).min(max);
// Or use standard library
use std::cmp::{min, max};
```

## 🎯 Focus Areas by Week

### Week 1: Foundation
- Set up project structure ✓
- Basic Iced app ✓
- File picker UI ✓
- JSON parsing ✓

### Week 2: Tree View
- Virtual scrolling
- Expand/collapse
- Syntax highlighting
- Node selection

### Week 3-4: Features
- Formatting
- RegEx search
- Multiple files
- Themes

### Week 5-6: Diff
- Diff algorithm
- Side-by-side view
- Diff navigation
- Export reports

## 💡 When You Get Stuck

1. **Compiler errors**: Read carefully, Rust errors are helpful
2. **Borrow checker**: Try `Arc<T>` or `Rc<T>` for shared ownership
3. **Performance**: Profile first, optimize later
4. **Iced confusion**: Check examples in Iced repo
5. **Design decisions**: Re-read DESIGN_DECISIONS.md

## 🔗 Essential Links

- **Iced Examples**: https://github.com/iced-rs/iced/tree/master/examples
- **Rust Book**: https://doc.rust-lang.org/book/
- **serde_json docs**: https://docs.rs/serde_json/
- **Tokio tutorial**: https://tokio.rs/tokio/tutorial

## 🎓 Learning Resources

- Rust by Example: https://doc.rust-lang.org/rust-by-example/
- Rust Async Book: https://rust-lang.github.io/async-book/
- Jon Gjengset's videos: https://www.youtube.com/c/JonGjengset

## ⚡ Performance Benchmarks to Track

```rust
// In benches/parser_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse(c: &mut Criterion) {
    let json = include_str!("../tests/fixtures/large.json");
    
    c.bench_function("parse_100mb", |b| {
        b.iter(|| {
            parse_json(black_box(json))
        });
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

## 🏁 Release Checklist

- [ ] All tests passing
- [ ] Benchmarks run (no regressions)
- [ ] Cross-platform testing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped
- [ ] Git tag created
- [ ] Binaries built for all platforms
- [ ] Release notes written

## 💾 Useful Cargo.toml Snippets

```toml
# Faster compile times in dev
[profile.dev.package."*"]
opt-level = 3

# Small binary size
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"

# Faster linking (macOS/Linux)
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

---

**Keep this file open while coding!** 📌
