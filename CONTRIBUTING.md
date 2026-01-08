# Contributing to Unfold

Thank you for your interest in contributing to Unfold!

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/) for automatic versioning and changelog generation.

### Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description | Version Bump |
|------|-------------|--------------|
| `feat` | New feature | Minor (1.0.0 → 1.1.0) |
| `fix` | Bug fix | Patch (1.0.0 → 1.0.1) |
| `docs` | Documentation only | No release |
| `style` | Formatting, no code change | No release |
| `refactor` | Code change, no new feature/fix | No release |
| `perf` | Performance improvement | Patch |
| `test` | Adding tests | No release |
| `chore` | Maintenance tasks | No release |
| `ci` | CI/CD changes | No release |

### Breaking Changes

For breaking changes, either:
- Add `!` after the type: `feat!: remove deprecated API`
- Add `BREAKING CHANGE:` in the footer

Breaking changes trigger a **major** version bump (1.0.0 → 2.0.0).

### Examples

```bash
# New feature (minor bump)
git commit -m "feat: add JSON path copy to clipboard"

# Bug fix (patch bump)
git commit -m "fix: correct search highlighting for special characters"

# Feature with scope
git commit -m "feat(search): add regex flags support"

# Breaking change (major bump)
git commit -m "feat!: change config file format to TOML"

# Documentation (no release)
git commit -m "docs: update keyboard shortcuts in README"

# Multiple paragraphs
git commit -m "feat: add export to CSV

This adds the ability to export JSON arrays as CSV files.
Supports nested objects with dot notation for column headers.

Closes #123"
```

## Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes with conventional commits
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Push and create a Pull Request

## Release Process

Releases are automated using [release-plz](https://release-plz.ieni.dev/):

1. When PRs are merged to `main`, release-plz analyzes commits
2. It creates a "release PR" with version bump and changelog updates
3. When the release PR is merged, it creates a git tag
4. The tag triggers the build and release workflow

You don't need to manually update versions or changelogs!
