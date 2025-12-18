# Testing in Rust - A Beginner's Guide

> Learn Rust testing from day one

## Why Test-Driven Development?

1. **Tests document behavior** - They show how your code should work
2. **Catch bugs early** - Before they become problems
3. **Enable refactoring** - Change code confidently knowing tests will catch breakage
4. **Learn by example** - Tests are executable documentation
5. **Rust makes testing easy** - Built-in test framework, no external tools needed

## Rust's Built-in Testing

Rust has testing built into the language and `cargo`. No need for external frameworks!

### Basic Test Structure

```rust
// In any .rs file (usually in the same file as the code being tested)

#[cfg(test)]
mod tests {
    use super::*;  // Import items from parent module

    #[test]
    fn test_something() {
        // Arrange - Set up test data
        let input = 42;

        // Act - Call the function
        let result = my_function(input);

        // Assert - Check the result
        assert_eq!(result, expected_value);
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output (see println! statements)
cargo test -- --nocapture

# Run a specific test
cargo test test_something

# Run tests matching a pattern
cargo test json_parser
```

## Common Test Assertions

```rust
#[test]
fn test_assertions() {
    // Equality
    assert_eq!(2 + 2, 4);
    assert_ne!(2 + 2, 5);  // Not equal

    // Boolean conditions
    assert!(true);
    assert!(!false);

    // With custom error messages
    assert_eq!(
        2 + 2,
        4,
        "Math is broken! 2+2 should be 4 but got {}",
        2 + 2
    );
}
```

## Testing Error Cases

```rust
#[test]
fn test_error_handling() {
    let result = parse_json("invalid json");

    // Check that it returns an error
    assert!(result.is_err());

    // Or use pattern matching
    match result {
        Err(e) => {
            // Can check error message
            assert!(e.to_string().contains("invalid"));
        }
        Ok(_) => panic!("Expected error but got Ok"),
    }
}

#[test]
#[should_panic]
fn test_panic_case() {
    // This test passes if the code panics
    my_function_that_should_panic();
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_specific_panic() {
    // Passes only if panic message contains this text
    vec![1, 2][99];  // This will panic with index out of bounds
}
```

## Testing with Result

```rust
#[test]
fn test_with_result() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_json(r#"{"key": "value"}"#)?;
    assert_eq!(parsed.get("key"), Some("value"));
    Ok(())  // Test passes if we return Ok
}
```

## Test Organization

### Option 1: Tests in Same File (Small modules)
```rust
// src/parser.rs
pub fn parse(input: &str) -> Result<Data, Error> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // Tests here
    }
}
```

### Option 2: Separate Test File (Larger modules)
```rust
// src/parser.rs
pub fn parse(input: &str) -> Result<Data, Error> {
    // Implementation
}

// tests/parser_tests.rs
use unfold::parser::parse;

#[test]
fn test_parse() {
    // Tests here
}
```

## Our Testing Strategy for Unfold

### Phase 1: Core Viewer
**What we'll test:**
```rust
// JSON parsing
#[test]
fn test_parse_simple_json() {
    let json = r#"{"name": "test"}"#;
    let tree = parse_json(json).unwrap();
    assert_eq!(tree.node_count(), 2); // Root + name field
}

// Tree building
#[test]
fn test_tree_structure() {
    let json = r#"{"nested": {"key": "value"}}"#;
    let tree = parse_json(json).unwrap();
    let root = tree.get_node(0).unwrap();
    assert_eq!(root.node_type, NodeType::Object);
}

// Node access
#[test]
fn test_get_node() {
    let tree = create_test_tree();
    let node = tree.get_node(0);
    assert!(node.is_some());
}
```

### Phase 2: Search & Format
**What we'll test:**
```rust
// Search
#[test]
fn test_search_finds_matches() {
    let tree = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
    let results = search(&tree, "Alice");
    assert_eq!(results.len(), 1);
}

// Formatting
#[test]
fn test_minify() {
    let input = r#"{
        "key": "value"
    }"#;
    let minified = minify(input).unwrap();
    assert_eq!(minified, r#"{"key":"value"}"#);
}
```

### Phase 3: Diff Engine
**What we'll test:**
```rust
// Diff comparison
#[test]
fn test_diff_detects_changes() {
    let left = parse_json(r#"{"a": 1}"#).unwrap();
    let right = parse_json(r#"{"a": 2}"#).unwrap();

    let diff = compare_trees(&left, &right);
    assert_eq!(diff.modifications, 1);
}
```

## Test-Driven Development Flow

### Example: Building a JSON Parser

**Step 1: Write a failing test**
```rust
#[test]
fn test_parse_empty_object() {
    let result = parse_json("{}");
    assert!(result.is_ok());
}
```

**Step 2: Run test (it will fail)**
```bash
cargo test
# Error: parse_json not found
```

**Step 3: Write minimal code to compile**
```rust
pub fn parse_json(input: &str) -> Result<JsonTree, ParseError> {
    todo!()  // Rust's placeholder
}
```

**Step 4: Run test (still fails)**
```bash
cargo test
# Panics: not yet implemented
```

**Step 5: Implement the function**
```rust
pub fn parse_json(input: &str) -> Result<JsonTree, ParseError> {
    // Actual implementation
    Ok(JsonTree::new())
}
```

**Step 6: Run test (passes!)**
```bash
cargo test
# ✓ test_parse_empty_object
```

**Step 7: Add more tests for edge cases**
```rust
#[test]
fn test_parse_nested_objects() { /* ... */ }

#[test]
fn test_parse_arrays() { /* ... */ }

#[test]
fn test_parse_invalid_json() { /* ... */ }
```

## Testing Best Practices

### ✅ DO:
- Write descriptive test names (`test_parse_handles_empty_array`)
- Test one thing per test
- Test edge cases (empty input, very large input, invalid input)
- Use `Result` in tests instead of `unwrap()` when possible
- Add comments explaining complex test setup

### ❌ DON'T:
- Don't test implementation details, test behavior
- Don't write tests that depend on each other
- Don't skip error cases
- Don't use `unwrap()` without thinking (use `?` or proper error handling)

## Useful Testing Patterns

### Test Helpers
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function used by multiple tests
    fn create_test_tree() -> JsonTree {
        parse_json(r#"{"test": "data"}"#).unwrap()
    }

    #[test]
    fn test_one() {
        let tree = create_test_tree();
        // Use tree...
    }

    #[test]
    fn test_two() {
        let tree = create_test_tree();
        // Use tree...
    }
}
```

### Parametrized Tests (using a loop)
```rust
#[test]
fn test_multiple_inputs() {
    let test_cases = vec![
        (r#"{"a":1}"#, 2),   // (input, expected_node_count)
        (r#"{"a":1,"b":2}"#, 3),
        (r#"{}"#, 1),
    ];

    for (input, expected) in test_cases {
        let tree = parse_json(input).unwrap();
        assert_eq!(tree.node_count(), expected);
    }
}
```

## Learning Progression

### Week 1-2: Basic Testing
- Write simple unit tests
- Use `assert_eq!` and `assert!`
- Test happy paths
- Test error cases with `is_err()`

### Week 3-4: Intermediate Testing
- Test async functions
- Use test helpers
- Test with different inputs
- Integration tests

### Week 5+: Advanced Testing
- Property-based testing with `proptest`
- Benchmarking with `criterion`
- Test coverage analysis
- Performance regression tests

## Next Steps

When you start coding:
1. I'll show you how to write your first test
2. We'll run it and see it fail
3. We'll implement code to make it pass
4. We'll add more tests for edge cases
5. You'll learn by doing!

**Remember**: Tests are your safety net. Write them early, run them often! 🧪

---

## Quick Reference

```bash
# Common cargo test commands
cargo test                    # Run all tests
cargo test -- --nocapture     # Show println! output
cargo test test_name          # Run specific test
cargo test --lib              # Run library tests only
cargo test --test integration # Run specific integration test
cargo test -- --test-threads=1 # Run tests serially (not parallel)
```
