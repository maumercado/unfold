# Unfold - Project Roadmap

## Timeline Overview

**Total Estimated Duration**: 8-12 weeks (part-time development)

```
Weeks 1-2:  Phase 1 - Core Viewer (MVP)
Weeks 3-4:  Phase 2 - Advanced Features  
Weeks 5-6:  Phase 3 - Comparison Engine
Weeks 7-8:  Phase 4 - Power Features
Weeks 9-10: Phase 5 - Polish & Testing
Weeks 11-12: Documentation & Release
```

## Phase 1: Core Viewer (MVP) - Weeks 1-2

**Goal**: Prove the concept with a working, high-performance viewer

### Week 1: Foundation

#### Day 1-2: Project Setup
- [x] Initialize Rust project with Cargo
- [x] Set up workspace structure
- [x] Configure dependencies (Iced, serde_json, tokio)
- [x] Set up CI/CD pipeline (GitHub Actions)
- [x] Create basic Iced application shell
- [x] Implement file picker UI

**Deliverable**: Empty app that can open file dialog

#### Day 3-4: Parser Core
- [ ] Implement `JsonNode` data structure
- [ ] Implement `JsonTree` flat array structure
- [ ] Create streaming parser with `serde_json`
- [ ] Add progress tracking for large files
- [ ] Write unit tests for parser
- [ ] Benchmark parsing speed

**Deliverable**: Parse JSON file into tree structure

#### Day 5-7: Basic Tree View
- [ ] Implement `TreeState` for tracking expanded nodes
- [ ] Create basic tree rendering
- [ ] Add expand/collapse functionality
- [ ] Implement syntax highlighting (simple colors)
- [ ] Add node selection
- [ ] Display node path on selection

**Deliverable**: View and navigate JSON tree

### Week 2: Virtual Scrolling & Polish

#### Day 1-3: Virtual Scrolling
- [ ] Implement `VirtualScroller` algorithm
- [ ] Calculate visible node range
- [ ] Add scroll event handling
- [ ] Optimize rendering performance
- [ ] Test with large files (100MB+)
- [ ] Profile and optimize

**Deliverable**: Smooth scrolling through millions of nodes

#### Day 4-5: Basic Search
- [ ] Implement text search in tree
- [ ] Add search UI (input + results counter)
- [ ] Highlight search matches
- [ ] Navigate between results (next/prev)
- [ ] Make search async/cancellable

**Deliverable**: Working search functionality

#### Day 6-7: MVP Polish
- [ ] Add file info display (size, node count, depth)
- [ ] Implement copy node value to clipboard
- [ ] Add keyboard shortcuts (expand, collapse, search)
- [ ] Basic error handling and user feedback
- [ ] Create README with usage instructions
- [ ] Record demo video

**Deliverable**: Functional MVP ready for internal testing

**Milestone**: MVP Complete - Can view and search large JSON files efficiently

## Phase 2: Advanced Features - Weeks 3-4

**Goal**: Add professional-grade features

### Week 3: Formatting & Validation

#### Day 1-3: JSON Formatter
- [ ] Implement pretty print with configurable indent
- [ ] Implement minifier
- [ ] Add key sorting option
- [ ] Create format configuration UI
- [ ] Add format preview
- [ ] Save formatted files

**Deliverable**: Format JSON with various options

#### Day 4-5: RegEx Search
- [ ] Integrate `regex` crate
- [ ] Add RegEx toggle in search UI
- [ ] Validate RegEx patterns with user feedback
- [ ] Cache compiled RegEx for performance
- [ ] Add RegEx help/examples

**Deliverable**: Search with RegEx patterns

#### Day 6-7: Validation & Export
- [ ] Implement JSON validation with error reporting
- [ ] Show validation errors inline in tree
- [ ] Add export selected node functionality
- [ ] Export to file with format options
- [ ] Add validation status indicator

**Deliverable**: Validate and export JSON

### Week 4: User Experience

#### Day 1-2: Multiple Files
- [ ] Implement tab system for multiple files
- [ ] Add tab switching UI
- [ ] Manage memory for multiple open files
- [ ] Add "close all" functionality
- [ ] Persist open tabs on restart

**Deliverable**: Open multiple JSON files in tabs

#### Day 3-4: Themes & Preferences
- [ ] Implement dark/light theme toggle
- [ ] Create preferences data structure
- [ ] Add settings panel UI
- [ ] Persist user preferences
- [ ] Add keyboard shortcut customization

**Deliverable**: Customizable appearance and behavior

#### Day 5-7: Polish & Testing
- [ ] Add keyboard navigation (vim bindings)
- [ ] Implement undo/redo for operations
- [ ] Comprehensive error handling
- [ ] Write integration tests
- [ ] Performance testing and optimization
- [ ] Bug fixes from internal testing

**Deliverable**: Polished, stable application

**Milestone**: Feature-complete viewer ready for beta testing

## Phase 3: Comparison Engine - Weeks 5-6

**Goal**: Differentiate with powerful diff capabilities

### Week 5: Diff Engine Core

#### Day 1-3: Structural Diff
- [ ] Implement diff algorithm (LCS for arrays)
- [ ] Create `DiffNode` and `DiffResult` structures
- [ ] Compare two JSON trees semantically
- [ ] Generate diff statistics
- [ ] Write unit tests for diff logic
- [ ] Benchmark diff performance

**Deliverable**: Working diff engine

#### Day 4-5: Comparison Modes
- [ ] Implement strict comparison mode
- [ ] Implement semantic comparison (ignore order)
- [ ] Implement flexible comparison (ignore keys order)
- [ ] Add comparison mode selector UI
- [ ] Test edge cases

**Deliverable**: Multiple comparison modes

#### Day 6-7: Diff Navigation
- [ ] Track difference locations
- [ ] Implement next/previous difference navigation
- [ ] Show current difference position (3/47)
- [ ] Jump to specific difference
- [ ] Filter by diff type (added/removed/modified)

**Deliverable**: Navigate through differences efficiently

### Week 6: Diff Visualization

#### Day 1-3: Side-by-Side View
- [ ] Create split-pane UI layout
- [ ] Synchronize scrolling between panes
- [ ] Align matching nodes visually
- [ ] Add diff color coding (green/red/yellow)
- [ ] Handle different tree depths

**Deliverable**: Visual side-by-side comparison

#### Day 4-5: Inline Diff View
- [ ] Show diffs inline in single tree
- [ ] Use color/icons to indicate diff type
- [ ] Add diff path display
- [ ] Toggle between side-by-side and inline
- [ ] Optimize for large diffs

**Deliverable**: Inline diff visualization

#### Day 6-7: Diff Reports
- [ ] Generate diff summary statistics
- [ ] Create exportable diff report (HTML)
- [ ] Add Markdown export option
- [ ] Include diff context in reports
- [ ] Test with real-world API diffs

**Deliverable**: Export diff reports

**Milestone**: Full comparison capabilities ready

## Phase 4: Power Features - Weeks 7-8

**Goal**: Professional-grade tooling for advanced users

### Week 7: Multi-File & Logs

#### Day 1-3: JSON-Lines Support
- [ ] Detect JSON-Lines / ndjson format
- [ ] Parse line-by-line efficiently
- [ ] Show line numbers in tree
- [ ] Support filtering by line range
- [ ] Test with large log files

**Deliverable**: View JSON log files

#### Day 4-5: File Watching
- [ ] Integrate `notify` crate
- [ ] Monitor file changes
- [ ] Auto-refresh on external modification
- [ ] Add manual refresh option
- [ ] Debounce rapid changes

**Deliverable**: Auto-refresh for live logs

#### Day 6-7: JSON Path Filtering
- [ ] Implement JSON path parser ($.users[*].email)
- [ ] Filter tree based on path
- [ ] Show only matching nodes
- [ ] Add path query UI
- [ ] Save favorite queries

**Deliverable**: Filter JSON by path expressions

### Week 8: Advanced Operations

#### Day 1-3: Multi-File Merge
- [ ] Load multiple files for comparison
- [ ] Show merged view of all files
- [ ] Highlight unique/common elements
- [ ] Export merged result
- [ ] Handle conflicts

**Deliverable**: Merge multiple JSON files

#### Day 4-5: Performance Monitoring
- [ ] Add performance metrics display
- [ ] Show memory usage in real-time
- [ ] Display parse/search/diff times
- [ ] Add profiling mode for optimization
- [ ] Export performance report

**Deliverable**: Built-in performance monitoring

#### Day 6-7: API Integration (Stretch)
- [ ] Add URL input for direct API calls
- [ ] Support basic auth / bearer tokens
- [ ] Fetch and display JSON response
- [ ] Save API endpoints for reuse
- [ ] Handle rate limiting

**Deliverable**: Fetch JSON from APIs directly

**Milestone**: Power features complete

## Phase 5: Polish & Distribution - Weeks 9-10

**Goal**: Production-ready release

### Week 9: Quality Assurance

#### Day 1-2: Comprehensive Testing
- [ ] Write missing unit tests (80% coverage goal)
- [ ] Create integration test suite
- [ ] Perform manual testing scenarios
- [ ] Test on Windows, macOS, Linux
- [ ] Performance regression testing

**Deliverable**: High-quality, tested codebase

#### Day 3-4: Error Handling
- [ ] Review all error paths
- [ ] Add user-friendly error messages
- [ ] Implement graceful degradation
- [ ] Add error reporting (opt-in)
- [ ] Test error scenarios

**Deliverable**: Robust error handling

#### Day 5-7: UI/UX Polish
- [ ] Refine visual design
- [ ] Improve responsive layouts
- [ ] Add loading indicators
- [ ] Smooth animations
- [ ] Accessibility improvements
- [ ] User feedback from beta testers

**Deliverable**: Polished user experience

### Week 10: Documentation & Packaging

#### Day 1-2: Documentation
- [ ] Write comprehensive README
- [ ] Create user guide
- [ ] Document keyboard shortcuts
- [ ] Add architecture documentation
- [ ] Create contributing guidelines

**Deliverable**: Complete documentation

#### Day 3-5: Distribution
- [ ] Set up release automation
- [ ] Create Windows installer (MSI)
- [ ] Create macOS bundle (DMG)
- [ ] Create Linux AppImage
- [ ] Test installers on clean systems
- [ ] Set up auto-update mechanism

**Deliverable**: Distributable packages

#### Day 6-7: Release Preparation
- [ ] Create marketing materials (screenshots, demo)
- [ ] Write blog post / announcement
- [ ] Prepare GitHub release notes
- [ ] Set up project website (GitHub Pages)
- [ ] Create demo video/GIF

**Deliverable**: Release-ready materials

**Milestone**: v1.0.0 Ready for Public Release

## Weeks 11-12: Launch & Iteration

### Week 11: Soft Launch

#### Day 1-3: Beta Release
- [ ] Release to select beta testers
- [ ] Gather feedback
- [ ] Monitor crash reports
- [ ] Quick bug fixes
- [ ] Performance tuning based on feedback

#### Day 4-7: Public Launch
- [ ] Publish v1.0.0 release
- [ ] Post to HackerNews, Reddit (r/rust, r/programming)
- [ ] Share on Twitter/LinkedIn
- [ ] Update portfolio website
- [ ] Respond to community feedback

### Week 12: Post-Launch

#### Day 1-7: Support & Iteration
- [ ] Monitor GitHub issues
- [ ] Fix critical bugs (hotfix releases)
- [ ] Plan v1.1.0 features based on feedback
- [ ] Write "lessons learned" blog post
- [ ] Begin planning next features

**Milestone**: Successful v1.0.0 Launch

## Post-v1.0 Roadmap (Future)

### v1.1.0 - Editing Support
- JSON editing mode
- In-place value modification
- Add/remove nodes
- Validation on edit

### v1.2.0 - Collaboration
- Export snapshots
- Compare snapshots
- Team sharing features
- Cloud sync (optional)

### v1.3.0 - Extensibility
- Plugin system (WASM-based)
- Custom transformers
- Scripting support
- Community plugins

### v2.0.0 - Beyond JSON
- YAML support
- TOML support
- XML support
- Universal tree viewer

## Resource Allocation

**Time Commitment**: 15-20 hours per week

### Weekly Breakdown
- Coding: 10-12 hours
- Testing: 2-3 hours
- Documentation: 1-2 hours
- Planning/Review: 1-2 hours

### Critical Path Items
1. Virtual scrolling performance (Week 2)
2. Diff algorithm correctness (Week 5)
3. Cross-platform builds (Week 10)

## Risk Management

### High-Risk Items
- **Virtual scrolling performance**: May need iteration to get right
  - *Mitigation*: Allocate extra time, consider simpler approach for MVP
- **Memory usage on huge files**: May exceed 1:1 ratio
  - *Mitigation*: Implement memory limits, chunked viewing fallback
- **Cross-platform issues**: UI may look different on each OS
  - *Mitigation*: Test early and often on all platforms

### Medium-Risk Items
- **Diff algorithm complexity**: Edge cases may be tricky
  - *Mitigation*: Extensive unit tests, fuzz testing
- **File format edge cases**: Various JSON encodings
  - *Mitigation*: Use battle-tested `serde_json`, add encoding detection

## Success Criteria

### Technical Success
- [ ] Handles 1GB+ files without crash
- [ ] Sub-2-second load time for 100MB files
- [ ] 60fps scrolling through millions of nodes
- [ ] Memory usage < 1.5x file size
- [ ] Zero critical bugs in production

### User Success
- [ ] 5+ positive beta tester reviews
- [ ] Used daily by creator (dogfooding)
- [ ] 100+ GitHub stars in first month
- [ ] Featured on Rust blog/newsletter

### Portfolio Success
- [ ] Demonstrates systems programming expertise
- [ ] Shows project completion ability
- [ ] Generates consulting interest
- [ ] Referenced in job applications

## Decision Points

### Week 2 Review
- Is virtual scrolling performant enough?
- Should we simplify for MVP?
- Are we on track for timeline?

### Week 6 Review
- Is diff algorithm correct and fast?
- Should we cut any Phase 4 features?
- Do we need more testing time?

### Week 10 Review
- Is quality good enough for release?
- Should we delay for more polish?
- Is documentation sufficient?

## Notes

- Priorities may shift based on feedback
- Be willing to cut features to meet timeline
- Focus on core value proposition: **performance**
- Don't let perfect be enemy of good

---

**Document Version**: 0.1.0  
**Last Updated**: 2025-12-12  
**Status**: Draft  
**Next Review**: Week 2 (Day 14)
