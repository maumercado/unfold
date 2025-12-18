# Unfold - Project Planning & Specifications

> High-performance JSON viewer built in Rust with Iced GUI framework and optional CLI

**Project Name**: Unfold  
**Tagline**: "Unfold your JSON"

## 📋 Project Overview

This repository contains the complete planning and specification documents for building a professional JSON viewer application that competes with tools like Dadroit. The tool focuses on **native performance**, **efficient memory usage**, and **developer-focused features** like diffing and formatting.

**Target Users**: Backend developers, DevOps engineers, data engineers, QA testers

**Core Value Propositions**:
- Native performance (handle multi-gigabyte files)
- Smart memory management (1:1 file-to-RAM ratio target)
- Developer-focused features (diff, format, search)
- Cross-platform (Windows, macOS, Linux)

## 📚 Documentation Structure

### Quick Start: [CLAUDE.md](./CLAUDE.md)
**AI Assistant Quick Reference** - Essential guidance for Claude Code when working in this repository

**What it covers**:
- Development commands and setup
- Core architecture and data structures
- Design decisions and trade-offs
- Performance optimization strategies
- Coding guidelines and best practices

**Read this first if**: You're an AI assistant or want a quick overview

---

### Detailed Planning Docs (in `/docs` folder)

### 1. [docs/PROJECT_SPEC.md](./docs/PROJECT_SPEC.md)
**What it covers**: High-level project requirements, features, and goals

**Read this first if**: You want to understand what we're building and why

---

### 2. [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
**What it covers**: Technical architecture, data structures, and system design

**Read this first if**: You want to understand how we'll build it

---

### 3. [docs/ROADMAP.md](./docs/ROADMAP.md)
**What it covers**: Timeline, phases, milestones, and task breakdown

**Read this first if**: You want to know when things will be done

---

### 4. [docs/IMPLEMENTATION.md](./docs/IMPLEMENTATION.md)
**What it covers**: Code examples, best practices, and implementation guide

**Read this first if**: You're ready to start coding

---

### 5. [docs/DESIGN_DECISIONS.md](./docs/DESIGN_DECISIONS.md)
**What it covers**: Key decisions, trade-offs, and rationale

**Read this first if**: You want to understand the "why" behind choices

---

### 6. [docs/MODULARITY.md](./docs/MODULARITY.md)
**What it covers**: Extensibility principles for future format support (TOML, YAML, etc.)

**Read this first if**: You're planning post-v1.0 multi-format support

---

### 7. [docs/QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md)
**What it covers**: One-page cheat sheet for rapid development

**Read this first if**: You need quick code templates and command references

---

## 🚀 Quick Start

### Phase 1: Core Viewer (Weeks 1-2)
**Goal**: Prove the concept with high-performance viewing

**Deliverables**:
- Parse and display JSON files
- Tree view with expand/collapse
- Virtual scrolling
- Basic search

**Success**: Can view and navigate 100MB+ files smoothly

---

### Phase 2: Advanced Features (Weeks 3-4)
**Goal**: Add professional-grade features

**Deliverables**:
- JSON formatting (pretty print, minify)
- RegEx search
- Multiple file tabs
- Themes and preferences

**Success**: Feature-complete viewer ready for beta

---

### Phase 3: Comparison Engine (Weeks 5-6)
**Goal**: Differentiate with powerful diff capabilities

**Deliverables**:
- Structural diff algorithm
- Side-by-side comparison view
- Diff navigation
- Export diff reports

**Success**: Full comparison capabilities

---

### Phase 4: Power Features (Weeks 7-8)
**Goal**: Professional-grade tooling

**Deliverables**:
- JSON-Lines / ndjson support
- File watching and auto-refresh
- JSON path filtering
- Multi-file merge

**Success**: Power features complete

---

### Phase 5: Polish & Release (Weeks 9-12)
**Goal**: Production-ready release

**Deliverables**:
- Comprehensive testing
- Documentation
- Cross-platform installers
- Public v1.0.0 release

**Success**: Launched and getting positive feedback

---

## 🎯 Key Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Load time (100MB) | < 2 seconds | Psychological threshold for "fast" |
| Memory usage | ~1.2x file size | Predictable, competitive with Dadroit |
| Scrolling | 60 FPS | Smooth interaction is critical UX |
| Search speed | > 50k results/sec | Fast enough for instant results |
| Startup time | < 500ms | Tool should feel responsive |

---

## 🛠️ Technology Stack

| Component | Choice | Version | Rationale |
|-----------|--------|---------|-----------|
| **Language** | Rust | Latest stable (2021 edition) | Native performance, memory safety, portfolio value |
| **Primary Interface** | Iced GUI | 0.14.0 | Pure Rust, native performance, cross-platform, Dec 2025 release |
| **CLI Support** | Optional CLI | - | For automation and scripting use cases (post-v1.0) |
| **JSON Parsing** | serde_json | 1.0+ | De facto standard, battle-tested, supports streaming |
| **Async Runtime** | Tokio | 1.48+ | Industry standard, excellent ecosystem |
| **Diff Algorithm** | similar | 2.6+ | Proven diff implementation |
| **Search** | regex | 1.11+ | Fast, well-maintained |
| **File Mapping** | memmap2 | 0.9+ | Efficient large file handling |
| **Parallel Processing** | rayon | 1.10+ | Data parallelism when needed |

**Note**: Versions current as of December 12, 2025

---

## 📦 Project Structure

### Current (Planning Phase)
```
unfold/
├── CLAUDE.md                      # AI assistant quick reference
├── README.md                      # This file
├── docs/                          # Planning documentation
│   ├── PROJECT_SPEC.md            # Requirements and features
│   ├── ARCHITECTURE.md            # Technical architecture
│   ├── ROADMAP.md                 # Timeline and phases
│   ├── IMPLEMENTATION.md          # Code examples and guides
│   ├── DESIGN_DECISIONS.md        # Decisions and trade-offs
│   ├── MODULARITY.md              # Extensibility principles
│   └── QUICK_REFERENCE.md         # Developer cheat sheet
└── LICENSE
```

### Planned (v1.0 Implementation)
```
unfold/
├── CLAUDE.md
├── README.md
├── Cargo.toml                     # Single crate for v1.0
├── src/
│   ├── main.rs                    # Entry point
│   ├── app.rs                     # Iced application
│   ├── parser/                    # JSON parsing & tree building
│   ├── ui/                        # UI widgets (tree view, search, etc.)
│   ├── diff/                      # Comparison engine
│   ├── search/                    # Search functionality
│   └── format/                    # Formatting operations
├── tests/                         # Integration tests
│   └── fixtures/                  # Test JSON files
├── benches/                       # Performance benchmarks
├── docs/                          # Planning docs
└── LICENSE
```

### Future (v2.0+ Multi-format Support)
```
unfold/
├── Cargo.toml                     # Workspace root
├── unfold-core/                   # Core abstractions/traits
├── unfold-json/                   # JSON implementation
├── unfold-toml/                   # TOML implementation
├── unfold-ui/                     # Iced GUI (format-agnostic)
├── unfold-cli/                    # CLI binary
└── docs/
```

**Note**: Start simple (single crate), refactor to workspace when adding multi-format support (see docs/MODULARITY.md).

---

## ✅ Features by Phase

### ✅ Phase 1: Core Viewer (MVP)
- [ ] Open and parse JSON files (up to 2GB)
- [ ] Tree view with expand/collapse
- [ ] Virtual scrolling
- [ ] Syntax highlighting
- [ ] Basic text search
- [ ] Copy node value
- [ ] Show node path
- [ ] File info display

### ✅ Phase 2: Advanced Features
- [ ] JSON formatting (pretty print, minify, custom indent)
- [ ] RegEx search
- [ ] Search result navigation
- [ ] Export selected node
- [ ] JSON validation
- [ ] Multiple file tabs
- [ ] Dark/light theme toggle
- [ ] Preferences system

### ✅ Phase 3: Comparison
- [ ] Side-by-side JSON diff
- [ ] Structural comparison
- [ ] Navigate between differences
- [ ] Highlight additions, deletions, modifications
- [ ] Show diff path/location
- [ ] Export diff reports

### ✅ Phase 4: Power Features
- [ ] JSON-Lines / ndjson support
- [ ] Auto-refresh for log files
- [ ] JSON path filtering
- [ ] Multi-file merge view
- [ ] Performance monitoring

### ✅ Phase 5: Polish
- [ ] Comprehensive error handling
- [ ] User documentation
- [ ] Installers (Windows MSI, macOS DMG, Linux AppImage)
- [ ] Auto-update mechanism
- [ ] Crash reporting (opt-in)

---

## 🎓 Learning Outcomes

Building this project will demonstrate:

1. **Systems Programming**: Efficient data structures, memory management
2. **Performance Optimization**: Profiling, benchmarking, algorithmic optimization
3. **Native GUI Development**: Modern UI patterns in Rust
4. **Cross-platform Development**: Building and distributing for multiple OSes
5. **Project Management**: Phased development, scope management
6. **Software Architecture**: Modular design, separation of concerns

---

## 💡 Design Principles

1. **Performance First**: Every decision considers impact on speed and memory
2. **User-Centric**: Build for actual use cases (API debugging, log analysis)
3. **Native Experience**: No web view compromises, true native feel
4. **Privacy-Focused**: Local-first, no telemetry by default
5. **Developer-Friendly**: Built by developers, for developers
6. **Production Quality**: No toy project - build something you'd use daily

---

## 🚧 Known Challenges

### Technical Challenges
1. **Virtual scrolling performance**: May need iteration to optimize
2. **Memory management for huge files**: May need chunked viewing fallback
3. **Cross-platform UI consistency**: Test early on all platforms
4. **Diff algorithm correctness**: Extensive testing required

### Scope Challenges
1. **Feature creep risk**: Strict phase gates to prevent
2. **Time availability**: Built-in buffer weeks
3. **Perfectionism trap**: MVP mindset, iterate later

---

## 📊 Success Metrics

### Technical Metrics (Must Achieve)
- Load 100MB files in < 2 seconds
- Memory usage within 50% of file size
- Zero crashes on valid JSON in 100 hours of testing
- Works on Windows, macOS, Linux

### User Metrics (Should Achieve)
- 5 positive beta tester reviews
- Daily use by creator (dogfooding)
- Primary use case in < 3 clicks

### Portfolio Metrics (Nice to Have)
- 100+ GitHub stars
- Featured in Rust newsletter
- Referenced in job applications
- Consulting inquiries

---

## 🔮 Future Possibilities (Post-v1.0)

- **v1.1**: JSON editing mode
- **v1.2**: Team collaboration features
- **v1.3**: Plugin system (WASM-based)
- **v2.0**: Support for YAML, TOML, XML (universal tree viewer)

---

## 📝 Notes

- All specifications are living documents - expect iterations
- Focus on learning and portfolio value over perfection
- Be willing to cut features to ship on time
- Community feedback will guide post-v1.0 direction

---

## 🤝 Contributing

Once the project is public, contributions will be welcome! For now, this is a solo project focused on learning and portfolio building.

---

## 📄 License

To be decided (likely MIT or Apache-2.0 for maximum openness)

---

## 📮 Contact

- **Email**: [Your Email]
- **GitHub**: [Your GitHub]
- **LinkedIn**: [Your LinkedIn]

---

**Status**: Planning Phase  
**Version**: 0.1.0  
**Last Updated**: December 12, 2025  
**Next Milestone**: Start Phase 1 (Week 1)

---

## 🎯 Next Steps

1. Review all specification documents
2. Set up development environment
3. Initialize Rust project
4. Start Phase 1: Core Viewer (MVP)
5. Begin daily development log

**Let's build something great! 🚀**
