# Changelog

All notable changes to Unfold will be documented in this file.

## [1.5.5](https://github.com/maumercado/unfold/compare/v1.5.4...v1.5.5) (2026-04-16)


### Bug Fixes

* **ci:** satisfy macOS clippy ([c900c6b](https://github.com/maumercado/unfold/commit/c900c6b7ce062d8e7fadbfcbe7e5a9ad61d5aca3))

## [1.5.4](https://github.com/maumercado/unfold/compare/v1.5.3...v1.5.4) (2026-04-16)


### Bug Fixes

* **mac:** open json files from Finder ([b3944ae](https://github.com/maumercado/unfold/commit/b3944ae7e78399cb69c9c3105047b3fdae7a9b81))

## [1.5.3](https://github.com/maumercado/unfold/compare/v1.5.2...v1.5.3) (2026-04-15)


### Bug Fixes

* restore tag-based release handoff ([ffda706](https://github.com/maumercado/unfold/commit/ffda706584d49d960e85266244d23958580c67eb))

## [1.5.2](https://github.com/maumercado/unfold/compare/v1.5.1...v1.5.2) (2026-04-15)


### Bug Fixes

* tie release artifacts to tags ([7db717e](https://github.com/maumercado/unfold/commit/7db717e67dc5ce570d610700c67c8de27469c4f5))

## [1.5.1](https://github.com/maumercado/unfold/compare/v1.5.0...v1.5.1) (2026-03-25)


### Bug Fixes

* show [] and {} for empty arrays and objects instead of [...] and {...} ([#11](https://github.com/maumercado/unfold/issues/11)) ([48c95d7](https://github.com/maumercado/unfold/commit/48c95d743ce0bd33b4cb6533a6fc6c42b5f98110))

## [1.5.0](https://github.com/maumercado/unfold/compare/v1.4.0...v1.5.0) (2026-03-10)


### Features

* unify search to always match both keys and values ([4a847f8](https://github.com/maumercado/unfold/commit/4a847f8e439abf258b39cf6072a17f6dd95a6dbc))

## [1.4.0](https://github.com/maumercado/unfold/compare/v1.3.0...v1.4.0) (2026-03-08)


### Features

* add inline search match highlighting within keys and values ([#7](https://github.com/maumercado/unfold/issues/7)) ([816452b](https://github.com/maumercado/unfold/commit/816452bf0b8c3d17b08e16aa2dfd4055ac7cf287))

## [1.3.0](https://github.com/maumercado/unfold/compare/v1.2.4...v1.3.0) (2026-03-08)


### Features

* add search scope filter (keys, values, or both) ([#5](https://github.com/maumercado/unfold/issues/5)) ([8219848](https://github.com/maumercado/unfold/commit/8219848a9764050ac14274b2eb806f716d527124))

## [1.2.4](https://github.com/maumercado/unfold/compare/v1.2.3...v1.2.4) (2026-02-17)


### Bug Fixes

* set window icon at runtime for Windows title bar ([3c86d4f](https://github.com/maumercado/unfold/commit/3c86d4f449bfb21fd83e36f3b7fa79bf660754eb))

## [1.2.3](https://github.com/maumercado/unfold/compare/v1.2.2...v1.2.3) (2026-02-17)


### Bug Fixes

* embed app icon in Windows exe ([ca70f56](https://github.com/maumercado/unfold/commit/ca70f56599f96fa3cb2ecbbb6f080e1e39e41031))

## [1.2.1](https://github.com/maumercado/unfold/compare/v1.2.0...v1.2.1) (2026-02-04)


### Bug Fixes

* prevent Windows from exiting immediately on double-click ([1fe4f64](https://github.com/maumercado/unfold/commit/1fe4f64b717701fc15f442e3f8c81293107869bd))

## [1.1.1](https://github.com/maumercado/unfold/compare/v1.1.0...v1.1.1) (2026-01-19)


### Bug Fixes

* bump version to 1.1.1 for corrected release ([41ef50c](https://github.com/maumercado/unfold/commit/41ef50cfd469834127755644b98d0c72ce06d330))
* update version to 1.1.0 in Cargo.toml ([acf0836](https://github.com/maumercado/unfold/commit/acf0836e226d9560ec853c266524fc165ffab5f3))

## [1.0.0] - 2025-01-08

### Features

- Tree view with expand/collapse
- Virtual scrolling for large files
- Syntax highlighting (keys, strings, numbers, booleans, null)
- Text and RegEx search with case-sensitivity toggle
- Dark/light theme toggle
- JSON path display in status bar on selection
- Native macOS menu bar
- Right-click context menu with submenus
- Copy value, key, or path (keyboard shortcuts + context menu)
- Copy/Export as minified or formatted JSON
- Expand/collapse all children
- Help overlay with keyboard shortcuts (Cmd+/)
- Check for updates from GitHub
- Open in external editor
- Better error messages with line numbers
- CLI argument support
- Multi-window support
