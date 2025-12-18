# Rust Learning Path for Unfold

> Learning Rust by building a real-world, high-performance application

## Learning Philosophy

**Learn by doing** - We'll learn Rust concepts as we need them, building progressively more complex features. Each phase introduces new Rust concepts naturally.

**Understand the "why"** - We won't just write code; we'll understand why Rust works the way it does, and why we're making specific choices.

**Embrace the compiler** - Rust's compiler is your teacher. When it gives errors, we'll learn what they mean and how to fix them.

## What You'll Learn (In Order)

### Phase 0: Rust Foundations (Week 0 - Before Phase 1)

**Goal**: Get comfortable with basic Rust syntax and tools

#### Rust Concepts:
- **Cargo** - Rust's build tool and package manager
  - Creating projects with `cargo new`
  - Building and running with `cargo run`
  - Managing dependencies in `Cargo.toml`
- **Basic syntax** - Variables, functions, println!, basic types
- **Ownership basics** - The most important Rust concept!
  - What is ownership?
  - Move semantics vs. Copy
  - Why Rust doesn't have a garbage collector
- **Borrowing** - How to use data without owning it
  - References (`&` and `&mut`)
  - The borrow checker
  - Why Rust prevents data races at compile time

#### Practical Exercise:
Build a tiny "Hello JSON" program:
```rust
// Read a JSON file
// Parse it with serde_json
// Print some values
```

**Learning Resources**:
- Rust Book Chapters 1-4: https://doc.rust-lang.org/book/
- Rustlings exercises: https://github.com/rust-lang/rustlings

---

### Phase 1: Core Viewer (Weeks 1-2)

**Goal**: Build the MVP JSON viewer

#### Rust Concepts You'll Learn:

**1. Structs and Enums**
- Defining custom data types
- Pattern matching with `match`
- The `Option<T>` and `Result<T, E>` types (Rust's way of handling nulls and errors)

**2. Error Handling**
- The `Result` type in depth
- The `?` operator for propagating errors
- Creating custom error types with `thiserror`
- When to use `unwrap()` (spoiler: rarely!)

**3. Memory Management**
- Stack vs. Heap
- `Box<T>` - Smart pointer for heap allocation
- `Arc<T>` - Atomic Reference Counting for shared ownership
- Why we use `Vec<T>` for dynamic arrays

**4. Working with External Crates**
- **serde_json** - JSON parsing
  - How serde works (serialization/deserialization)
  - The Deserialize trait
  - Streaming parser for large files
- **iced** - GUI framework
  - The Elm architecture (Model-View-Update)
  - Widgets and layouts
  - Message passing

**5. Modules and Project Structure**
- `mod` keyword and module system
- File organization
- Public vs. private (`pub`)
- `use` for importing

#### What We'll Build:
```
Week 1:
- Set up Cargo project
- Create basic Iced window
- Add file picker
- Parse small JSON file
- Display raw text

Week 2:
- Create JsonNode struct
- Build tree structure
- Render tree in UI
- Add expand/collapse
```

#### Key Learning Moments:
- **Ownership puzzle**: "Why can't I use `tree` after I've passed it to the UI?"
  - Learn about `Arc<T>` for shared ownership
- **Borrow checker errors**: "Why won't this compile?"
  - Understand borrowing rules
- **Pattern matching**: "How do I handle different JSON types?"
  - Learn Rust's powerful `match` expressions

---

### Phase 2: Advanced Features (Weeks 3-4)

**Goal**: Add search, formatting, and polish

#### Rust Concepts You'll Learn:

**1. Traits** - Rust's version of interfaces
- What traits are and why they're powerful
- Implementing traits for your types
- Generic functions with trait bounds
- Common traits: `Clone`, `Debug`, `Display`

**2. Generic Programming**
- Generic functions and structs
- Type parameters
- Why generics have zero runtime cost

**3. Iterators** - Functional programming in Rust
- The `Iterator` trait
- `map`, `filter`, `fold`, `collect`
- Iterator chains
- Why iterators are fast (lazy evaluation)

**4. String Handling**
- `String` vs. `&str` - owned vs. borrowed strings
- `Arc<str>` for shared, immutable strings
- UTF-8 and why Rust strings are different

**5. Regular Expressions**
- The `regex` crate
- Pattern matching
- Performance considerations

**6. Async Rust (Introduction)**
- What is async/await?
- The `tokio` runtime
- Why async is important for I/O
- `async fn` and `Future`

#### What We'll Build:
```
Week 3:
- Text search with iterators
- RegEx search with regex crate
- Format/minify functions
- Export functionality

Week 4:
- Multiple file tabs (state management)
- Theme system
- Preferences/settings
- Keyboard shortcuts
```

#### Key Learning Moments:
- **Trait bounds**: "How do I write a generic function?"
- **Iterator chains**: "This is like JavaScript map/filter but faster!"
- **Async puzzle**: "Why do I need .await?"

---

### Phase 3: Comparison Engine (Weeks 5-6)

**Goal**: Build the diff algorithm

#### Rust Concepts You'll Learn:

**1. Lifetimes** - Rust's most confusing feature (we'll make it clear!)
- What lifetimes are
- Why Rust needs them
- Lifetime annotations (`'a`)
- Common lifetime patterns
- When you can elide (skip) them

**2. Advanced Enums**
- Enums with data
- Recursive data structures
- `Box<T>` for recursion

**3. Smart Pointers Deep Dive**
- `Box<T>`, `Rc<T>`, `Arc<T>` comparison
- When to use each
- `RefCell<T>` for interior mutability

**4. Algorithms**
- Implementing LCS (Longest Common Subsequence)
- Recursive algorithms in Rust
- Performance optimization techniques

**5. Using External Diff Libraries**
- The `similar` crate
- When to use libraries vs. implement yourself

#### What We'll Build:
```
Week 5:
- DiffNode enum
- Tree comparison algorithm
- Diff result structure
- Basic diff view

Week 6:
- Side-by-side comparison UI
- Diff navigation
- Color coding
- Export diff reports
```

#### Key Learning Moments:
- **Lifetime errors**: "What does 'borrowed value does not live long enough' mean?"
- **Recursive structures**: "Why do I need Box<T>?"
- **Algorithm translation**: "How do I convert this algorithm to Rust?"

---

### Phase 4: Power Features (Weeks 7-8)

**Goal**: Add advanced capabilities

#### Rust Concepts You'll Learn:

**1. Async/Await Deep Dive**
- Futures and the async runtime
- `tokio::spawn` for concurrent tasks
- Async file I/O
- Cancellation tokens

**2. Memory-Mapped Files**
- The `memmap2` crate
- Unsafe Rust (brief introduction)
- When and why to use `unsafe`

**3. Multi-threading**
- `std::thread`
- `rayon` for data parallelism
- Thread safety: `Send` and `Sync` traits
- The difference between async and threads

**4. File System Watching**
- The `notify` crate
- Event-driven programming
- Debouncing

**5. Performance Optimization**
- Profiling with `cargo flamegraph`
- Benchmarking with `criterion`
- Common optimization patterns
- When to optimize (and when not to)

#### What We'll Build:
```
Week 7:
- JSON-Lines parser
- File watcher
- Auto-refresh
- Background parsing

Week 8:
- JSON path filtering
- Multi-file operations
- Performance profiling
- Optimization pass
```

#### Key Learning Moments:
- **Send vs Sync**: "What do these auto-traits mean?"
- **Async vs threads**: "When should I use which?"
- **Performance profiling**: "Let's measure before optimizing!"

---

### Phase 5: Polish & Release (Weeks 9-12)

**Goal**: Production-ready release

#### Rust Concepts You'll Learn:

**1. Testing**
- Unit tests with `#[test]`
- Integration tests
- `#[cfg(test)]` modules
- Property-based testing with `proptest`
- Mocking and test utilities

**2. Documentation**
- Doc comments (`///`)
- `cargo doc`
- Examples in docs
- Module-level documentation

**3. Release Builds**
- Profile optimization
- LTO (Link-Time Optimization)
- Binary size reduction
- Cross-compilation

**4. Error Handling Patterns**
- `anyhow` vs. `thiserror`
- Error context
- Error reporting to users

**5. Macros (Brief Introduction)**
- Declarative macros
- Common macro patterns
- When to use macros

#### What We'll Build:
```
Week 9:
- Comprehensive test suite
- Integration tests
- Benchmark suite
- CI/CD setup

Week 10:
- Documentation
- User guide
- API documentation
- Examples

Week 11-12:
- Cross-platform testing
- Installers
- Release automation
- Public release!
```

---

## Recommended Learning Resources

### Essential Reading:
1. **The Rust Programming Language** (The Book)
   - https://doc.rust-lang.org/book/
   - Read chapters as we need them

2. **Rust by Example**
   - https://doc.rust-lang.org/rust-by-example/
   - Great for seeing code examples

3. **Rustlings** - Interactive exercises
   - https://github.com/rust-lang/rustlings
   - Do exercises in spare time

### When You Get Stuck:
1. **Rust Compiler Error Index**
   - https://doc.rust-lang.org/error-index.html
   - Explains common errors

2. **Rust Users Forum**
   - https://users.rust-lang.org/
   - Friendly community

3. **r/rust Subreddit**
   - Great for questions and learning

### Video Learning:
1. **Jon Gjengset's YouTube**
   - https://www.youtube.com/c/JonGjengset
   - Advanced topics explained clearly

2. **Rust Official YouTube**
   - Conference talks and tutorials

### Reference:
1. **std docs** - https://doc.rust-lang.org/std/
2. **docs.rs** - All crate documentation

---

## How We'll Work Together

### Each New Feature (Test-Driven Approach):
1. **Discussion**: What are we building and why?
2. **Concept Introduction**: What Rust concepts do we need?
3. **Write Test First**: Define expected behavior with a test
4. **Guided Implementation**: I'll guide, you'll write code to pass the test
5. **Explanation**: Why did we write it this way?
6. **Experimentation**: Try variations, see what breaks
7. **Refinement**: Improve based on compiler feedback
8. **More Tests**: Add edge cases and additional tests

### When You're Stuck:
- Read compiler errors together
- Explain what the error means
- Show how to fix it
- Explain why it happened

### Building Understanding:
- I'll ask you questions to check understanding
- You ask questions anytime!
- We'll explore "what if" scenarios
- We'll look at alternatives

---

## Key Rust Concepts Timeline

**Weeks 1-2**: Ownership, Borrowing, Structs, Enums, Result/Option
**Weeks 3-4**: Traits, Generics, Iterators, Async basics
**Weeks 5-6**: Lifetimes, Smart Pointers, Recursive types
**Weeks 7-8**: Advanced Async, Concurrency, Unsafe, Performance
**Weeks 9-12**: Testing, Documentation, Release engineering

---

## Progress Tracking

As you learn concepts, mark them here:

### Fundamentals:
- [ ] Ownership and moves
- [ ] Borrowing (`&` and `&mut`)
- [ ] Structs and enums
- [ ] Pattern matching
- [ ] Error handling (Result/Option)

### Intermediate:
- [ ] Traits
- [ ] Generics
- [ ] Iterators
- [ ] String types (String vs &str)
- [ ] Basic lifetimes

### Advanced:
- [ ] Complex lifetimes
- [ ] Smart pointers (Box, Rc, Arc)
- [ ] Async/await
- [ ] Multi-threading
- [ ] Unsafe Rust

### Mastery:
- [ ] Performance optimization
- [ ] Advanced trait patterns
- [ ] Macros
- [ ] FFI (Foreign Function Interface)
- [ ] No-std programming

---

**Remember**: Learning Rust is challenging but incredibly rewarding. The compiler is strict because it's helping you write correct, safe code. Every error is a learning opportunity!

**Let's build something amazing while learning one of the most powerful programming languages!** 🦀
