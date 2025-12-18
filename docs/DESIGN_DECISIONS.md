# Unfold - Design Decisions & Trade-offs

## Overview

This document captures the key design decisions made for the JSON viewer project, along with the rationale and trade-offs for each decision.

## Major Design Decisions

### 1. UI Framework: Iced vs Tauri

**Decision**: Use Iced for native GUI

**Rationale**:
- Native performance crucial for rendering millions of nodes
- Direct access to parsed data structures without serialization
- Lower memory overhead (no web engine)
- Simpler architecture (single language)

**Trade-offs**:
- **Pro**: Maximum performance, smaller binary, true native feel
- **Pro**: No JavaScript bridge overhead
- **Pro**: Better memory management (no GC)
- **Con**: Less familiar UI paradigm (vs web technologies)
- **Con**: Smaller component ecosystem compared to React/Vue
- **Con**: UI polish may take longer

**Alternative Considered**: Tauri with Svelte
- Would leverage your existing Svelte expertise
- Faster UI development with web tech
- But performance bottleneck for large files
- **Why rejected**: Performance is our core value proposition

---

### 2. Data Structure: Flat Array vs Tree Pointers

**Decision**: Store nodes in flat `Vec<JsonNode>` with index-based references

**Rationale**:
- Better cache locality (nodes stored contiguously)
- Simpler memory management (no lifetime issues)
- Easier serialization for caching
- Predictable memory usage

**Trade-offs**:
- **Pro**: Cache-friendly, fast iteration
- **Pro**: No lifetime annotations needed
- **Pro**: Can easily swap nodes in/out for memory management
- **Con**: Slight indirection cost (index lookup)
- **Con**: Must validate indices before access
- **Con**: Can't use standard tree traversal patterns

**Alternative Considered**: Tree with `Box<Node>` pointers
- More "traditional" tree structure
- Easier to understand
- **Why rejected**: Worse cache performance, harder memory management

---

### 3. Parsing Strategy: Streaming vs Full Load

**Decision**: Use streaming parser with lazy-loading for large files

**Rationale**:
- Handle arbitrarily large files
- Faster time-to-first-render
- Lower memory footprint
- Progressive loading UX

**Trade-offs**:
- **Pro**: Can open multi-GB files
- **Pro**: Better user experience (immediate feedback)
- **Pro**: Memory scales with visible nodes, not file size
- **Con**: More complex implementation
- **Con**: Some operations require full parse (e.g., search all)
- **Con**: Need to cache parsed nodes intelligently

**Alternative Considered**: Parse entire file upfront
- Simpler implementation
- All data available immediately
- **Why rejected**: Cannot handle 1GB+ files in reasonable memory

---

### 4. Virtual Scrolling: Custom vs Library

**Decision**: Implement custom virtual scrolling

**Rationale**:
- Full control over rendering performance
- Optimized for tree structure (not just list)
- Can add tree-specific optimizations
- Learning opportunity

**Trade-offs**:
- **Pro**: Maximum performance optimization
- **Pro**: Tree-aware (handle expand/collapse efficiently)
- **Pro**: No external dependencies for core feature
- **Con**: More initial development time
- **Con**: Need to handle edge cases ourselves
- **Con**: Potential bugs to work through

**Alternative Considered**: Use existing Iced scrollable
- Faster to implement
- Battle-tested
- **Why rejected**: May not handle millions of nodes efficiently

---

### 5. String Interning: Yes vs No

**Decision**: Use string interning for duplicate keys/values

**Rationale**:
- JSON often has many duplicate keys (e.g., "id", "name")
- Large memory savings for repetitive data
- Small performance cost, big memory win

**Trade-offs**:
- **Pro**: 30-50% memory reduction for typical JSON
- **Pro**: Faster comparisons (pointer equality)
- **Con**: Slight overhead for interning lookups
- **Con**: Memory not freed until tree dropped
- **Con**: More complex implementation

**Alternative Considered**: Store strings directly
- Simpler implementation
- **Why rejected**: Memory waste for large files

---

### 6. Diff Algorithm: Structural vs Text-based

**Decision**: Implement structural (semantic) diff

**Rationale**:
- Understands JSON structure, not just text
- Meaningful diffs (e.g., array reordering)
- Can offer multiple comparison modes
- Better for API response comparison

**Trade-offs**:
- **Pro**: Smarter diffs for JSON data
- **Pro**: Can ignore cosmetic changes (whitespace, order)
- **Pro**: Path-based diff reporting
- **Con**: More complex than text diff
- **Con**: Slower for very large files
- **Con**: Must handle edge cases carefully

**Alternative Considered**: Text-based line diff
- Much simpler (use existing library)
- Faster
- **Why rejected**: Not useful for JSON (whitespace changes create noise)

---

### 7. Search: Index vs Linear Scan

**Decision**: Start with linear scan, add indexing later (Phase 2)

**Rationale**:
- MVP doesn't require index
- Index adds complexity and memory
- Can add incrementally if needed
- Linear scan fast enough for most cases

**Trade-offs**:
- **Pro**: Simpler initial implementation
- **Pro**: No memory overhead for index
- **Pro**: Works well for one-off searches
- **Con**: Slower for repeated searches
- **Con**: No instant-search as-you-type

**Future Enhancement**: Add optional inverted index
- Build index on first search
- Update incrementally
- Toggle in settings

---

### 8. File Watching: Auto-refresh vs Manual

**Decision**: Support both with auto-refresh as opt-in

**Rationale**:
- Log file use case needs auto-refresh
- But auto-refresh can be disruptive
- User should control behavior

**Trade-offs**:
- **Pro**: Flexible for different use cases
- **Pro**: No surprise UI changes
- **Con**: Slightly more complex settings
- **Con**: Need to handle file modifications gracefully

**Implementation**: Use `notify` crate with debouncing

---

### 9. Themes: Built-in vs Customizable

**Decision**: Start with 2 built-in themes (dark/light), custom later

**Rationale**:
- Most users satisfied with dark/light
- Custom themes add UI complexity
- Can add in future if requested
- Focus on core functionality first

**Trade-offs**:
- **Pro**: Simpler settings UI
- **Pro**: Less maintenance burden
- **Con**: Power users may want customization
- **Con**: May not suit all accessibility needs

**Future Enhancement**: Theme customization with presets

---

### 10. Error Handling: Strict vs Lenient

**Decision**: Lenient parsing with clear error reporting

**Rationale**:
- Real-world JSON often has issues
- Users need tool to diagnose problems
- Better to show partial data than fail completely

**Trade-offs**:
- **Pro**: More useful for debugging bad JSON
- **Pro**: Handles edge cases gracefully
- **Pro**: Better user experience
- **Con**: May accept technically invalid JSON
- **Con**: More complex error recovery logic

**Approach**: 
- Validate first, show errors
- Allow viewing valid portions
- Highlight problematic areas

---

## Performance Targets & Rationale

### Target: 1:1 Memory Ratio (File Size to RAM)

**Rationale**: 
- Dadroit claims this as competitive advantage
- Users want to know tool won't consume all RAM
- Predictable memory usage builds trust

**How We'll Achieve It**:
- String interning (reduce string duplication)
- Lazy loading (don't load invisible nodes)
- Efficient data structures (flat array, Arc for sharing)
- Optional streaming mode for huge files

**Reality Check**: 
- Likely 1.2x-1.5x ratio due to:
  - Index overhead
  - UI state
  - Iced framework overhead
- Still excellent compared to alternatives (VSCode: 3-5x)

---

### Target: Sub-2-Second Load for 100MB Files

**Rationale**:
- 163x improvement over Notepad++ (per Dadroit)
- Users expect near-instant for common file sizes
- Psychological threshold (~2 sec feels fast)

**How We'll Achieve It**:
- Efficient parsing (serde_json is very fast)
- Streaming/progressive loading
- Defer non-critical work (syntax highlighting, etc.)
- Show partial UI while parsing continues

**Reality Check**:
- May take 3-4 seconds on slower machines
- Still significantly faster than alternatives

---

### Target: 60 FPS Scrolling

**Rationale**:
- Smooth interaction is critical UX
- Jank is immediately noticeable
- Competitive advantage over web-based tools

**How We'll Achieve It**:
- Virtual scrolling (render only visible)
- Pre-render buffer (smooth scrolling)
- Optimize rendering path (minimize allocations)
- GPU-accelerated rendering (Iced handles this)

**Reality Check**:
- May drop to 30 FPS on older hardware
- But still better than web alternatives

---

## Technology Choices

### Rust

**Why**:
- Memory safety without GC overhead
- Native performance
- Excellent tooling (cargo, clippy)
- Growing ecosystem
- Portfolio value (demonstrates systems programming)

**Trade-offs**:
- Steeper learning curve than JS/Python
- Slower initial development
- But: Better final performance and reliability

---

### Iced

**Why**:
- Pure Rust (no FFI overhead)
- Good performance characteristics
- Cross-platform out of box
- Elm-inspired architecture (predictable state)

**Trade-offs**:
- Smaller ecosystem than Qt/GTK
- Less mature (still evolving)
- But: Growing fast, good community

---

### serde_json

**Why**:
- De facto standard for JSON in Rust
- Battle-tested and optimized
- Supports streaming
- Great error messages

**Trade-offs**:
- Could use simd-json for 2-3x speedup
- But: serde_json easier to use, "fast enough"

---

## Features Explicitly Excluded from v1.0

### 1. JSON Editing

**Why Excluded**:
- Significantly increases scope
- Different UX paradigm (read vs write)
- Validation becomes much more complex
- Save/autosave adds complexity

**Future Consideration**: 
- Maybe v1.1 or v2.0
- Would need undo/redo system
- Validation on every edit
- Backup/recovery system

---

### 2. Cloud Integration

**Why Excluded**:
- Local-first is simpler and more secure
- No auth/sync complexity
- No server costs
- Privacy by default

**Future Consideration**:
- Optional cloud sync for settings/bookmarks
- But core functionality stays local

---

### 3. Real-time Collaboration

**Why Excluded**:
- Massive scope increase
- Requires server infrastructure
- Operational transform is complex
- Not core use case

**Future Consideration**: 
- Unlikely unless there's significant demand
- Would be separate product/tier

---

### 4. Plugin System

**Why Excluded**:
- Complex to design well
- Security concerns (arbitrary code)
- Maintenance burden
- Can add features directly for now

**Future Consideration**:
- WASM-based plugins in v2.0
- Safe, sandboxed
- Community-driven extensions

---

## Open Design Questions

### 1. Should we support YAML/TOML?

**Considerations**:
- Broadens appeal (more file types)
- Relatively easy (just add parsers)
- But: dilutes focus on JSON excellence
- May confuse positioning/marketing

**Decision**: Defer to post-v1.0
- Focus on being best JSON tool first
- Can add format support later if demand exists

---

### 2. CLI Tool vs GUI Only?

**Considerations**:
- CLI version useful for automation
- Can share core parsing/diff logic
- But: adds maintenance burden
- Different UX paradigm

**Decision**: GUI first, optional CLI later
- Core functionality in library crate
- GUI in separate binary
- Easy to add CLI wrapper later
- Some users might prefer CLI for scripting

---

### 3. Telemetry/Analytics?

**Considerations**:
- Helps understand usage patterns
- Can prioritize features better
- But: privacy concerns
- Setup/maintenance overhead

**Decision**: No telemetry in v1.0
- Privacy-first approach
- Can add opt-in crash reporting later
- Rely on GitHub issues for feedback
- Focus on product quality over metrics

---

### 4. Paid vs Free?

**Considerations**:
- Time investment deserves compensation
- Paid incentivizes quality/support
- But: free gets more adoption/stars
- Portfolio value vs revenue

**Decision**: Free and open source (MIT/Apache-2.0)
- Best for portfolio showcase
- Community building
- Can monetize later with:
  - Premium support/consulting
  - Team/enterprise features
  - Cloud-hosted version
- Open source demonstrates confidence in code quality

---

## Risk Mitigation Strategies

### Risk: Iced Immaturity

**Mitigation**:
- Keep UI layer separate (can swap if needed)
- Contribute back to Iced (build goodwill)
- Have fallback plan (egui) if Iced blocks progress
- Start simple, validate approach early

---

### Risk: Performance Targets Unreachable

**Mitigation**:
- Profile early and often
- Set realistic MVP targets first
- Can compromise on edge cases
- Focus on common use cases (10-500MB files)
- Defer huge file support (1GB+) if needed

---

### Risk: Scope Creep

**Mitigation**:
- Strict phase gates (don't start Phase 2 until Phase 1 complete)
- Focus on MVP (Minimum **Viable** Product, not Minimum Feature Product)
- Track feature requests separately
- Be willing to cut features to hit timeline
- Remember: Can always add features later

---

### Risk: Time Availability

**Mitigation**:
- Realistic timeline (12 weeks part-time)
- Built-in buffer (weeks 11-12)
- Can extend timeline if needed
- Focus on learning over perfect execution
- OK to take breaks between phases

---

## Success Criteria (Repeated for Emphasis)

### Minimum Success (Must Have)
- [ ] Opens and displays 100MB JSON file
- [ ] Smooth scrolling through tree
- [ ] Basic search works
- [ ] Works on your machine (dogfooding)
- [ ] No crashes on valid JSON

### Good Success (Should Have)
- [ ] Handles 500MB+ files
- [ ] Diff two JSON files
- [ ] Format/minify JSON
- [ ] Works on Windows/Mac/Linux
- [ ] 5 happy beta testers

### Great Success (Nice to Have)
- [ ] Handles 1GB+ files
- [ ] All Phase 4 features
- [ ] 100+ GitHub stars
- [ ] Featured in Rust newsletter
- [ ] Used in production somewhere

---

**Document Version**: 0.1.0  
**Last Updated**: 2025-12-12  
**Status**: Draft
