# GPUI Migration (Phased)

Status: **In progress**: Phases 0 to 2 built, Phase 3 mostly built, waiting on hand checks and the go or no-go call.

## Progress

| Phase | State | Where |
|---|---|---|
| 0. Baseline | Done (headless numbers; window numbers still open) | `migration/phase-0-baseline`, `gpui-baseline.md` |
| 1. Extract the core | Done | `migration/phase-1-core` |
| 2. GPUI spike | Built and tested headless. See `gpui-spike-results.md` | `migration/phase-2-gpui-spike` |
| 3. Feature parity | Mostly built in the spike, see the results doc | `migration/phase-2-gpui-spike` |
| 4. Platform behavior | Partly: menu bar, open events, drops, multi window in one process | `migration/phase-2-gpui-spike` |
| 5. Packaging | Not started | |

Changes from the plan, found during the spike:
* gpui-kit is 0.10.0 now, pinned as `=0.10.0`.
* The tree uses GPUI's `uniform_list` instead of gpui-kit's `VirtualList` (fixed row height, no per-row size vector).
* The core crate keeps JSON key order from the file (`serde_json/preserve_order`), because GPUI turns that on for the whole build anyway.
* The Iced app stayed at the repository root instead of moving to `crates/unfold-iced`. The GPUI app is `crates/unfold-gpui`.
* Windows open in one process, so `LaunchOptions`, `--spawn-handoff` and `macos_open.rs` are not needed in the GPUI build.

## Goal

Find out whether moving Unfold's UI from Iced to GPUI, using gpui-kit components, is worth it, without putting the working app at risk.

The work is split into phases that each end in a decision point. The Iced app stays shippable the whole time and nothing is deleted until the GPUI build matches it.

## Architecture

### Where things stand

About 1,600 lines have no UI dependency: `parser/*`, `search.rs`, `json_export.rs`, `config.rs`, `parse_error.rs`, `update_check.rs`. They carry over unchanged.

A few files are lightly tied to Iced:
* `flat_row.rs` and `theme.rs` use Iced's `Color`.
* `message.rs` uses Iced's key and modifier types.

The heavy part is `main.rs` (2,769 lines), which holds app state, `update`, and every `view` function. `menu.rs` depends on muda, and `macos_open.rs` hooks into winit's app delegate, which GPUI does not use.

### Target layout

A Cargo workspace with three crates:
1. `unfold-core`: parser, row flattening (`flatten_visible_nodes` and `flatten_node`, currently methods on `App`), search, export, config, parse errors. No UI dependency. It returns a plain `ValueType` and each UI maps that to a color.
2. `unfold-iced`: the current app, same behavior.
3. `unfold-gpui`: the new app on gpui-kit, pinned to an exact version (0.7.3 when this was written, 0.10.0 now).

### One design choice up front

Keep the flat `Vec<FlatRow>` model and render it with gpui-kit's `VirtualList`. Do not use gpui-kit's `Tree` component. Its docs do not say it virtualizes, and it expects a pre-built `TreeItem` hierarchy, which does not suit multi gigabyte files. Keeping `FlatRow` also keeps the tree line prefixes and syntax colors under our control.

## Ground Rules

1. The Iced app keeps releasing normally through every phase.
2. One small PR per step. No giant branch.
3. Nothing from Iced is deleted until Phase 5 is done.
4. Each phase has an exit check. If it fails, stop and decide before going on.
5. Follow CLAUDE.md: explain the idea before writing the code.

## Phases

Sizes are rough: S is a sitting, M is a few sittings, L is a few weeks of part time work.

### Phase 0: Baseline (S)

1. Add a script in `scripts/` that generates test JSON files of about 10MB, 100MB, and 500MB. Do not commit the generated files.
2. Measure the current Iced build on those files: time to open, memory after load, scroll smoothness, and the cost of opening every node under the root.
3. Save the numbers in `docs/gpui-baseline.md`.

Exit check: the numbers are written down. If Iced already meets the targets in DESIGN_DECISIONS.md and scrolls smoothly, the case for moving rests on developer experience, not speed. That is a fine reason, but it should be a conscious one.

### Phase 1: Extract the core (S to M)

1. Convert the repo to a workspace and move the UI free code into `unfold-core`.
2. Move row flattening out of `App` into the core crate, with the 7 tests that live in `main.rs`.
3. Replace `Color` in `ValueType` and `FlatRow` with plain types. The Iced crate converts to colors.
4. Update `rust.yml`, `release.yml`, and the `[package.metadata.bundle]` section for the new crate paths. Run `cargo bundle` in a PR first, since the bundle and document type steps are the most likely to break.

Exit check: all tests pass on macOS, Windows, and Linux CI, and a test release produces the same `.app`. No behavior change. This phase is worth merging even if the GPUI work never happens, since it also makes the CLI tool from DESIGN_DECISIONS.md possible.

### Phase 2: GPUI spike (M)

Work on a `gpui-spike` branch in a new `unfold-gpui` crate. Throwaway code is allowed. Colors can be hardcoded.

In scope: a window, open a file from a CLI argument and from `rfd`, parse with the core crate, `VirtualList` over `FlatRow`, click a row to toggle it, scroll, key and value colors, tree line prefixes.

Out of scope: search, menus, themes, dialogs, packaging.

Questions the spike must answer:
1. Does `VirtualList` stay smooth at 1M rows, and at 10M, with a fixed 16px row height?
2. Can rows scroll horizontally for long values (Iced does this today)?
3. Can we scroll to a given row from code? Search needs this.
4. How does clipboard access work, and can text in rows be selected?
5. How do keyboard actions work, and do they clash with a focused `Input`? This is the same kind of bug fixed in PR 19 and PR 20.
6. Does it build cleanly on macOS and Linux? Which system packages does Linux need, and how do compile time and binary size compare to Iced?
7. How are Finder open events and file drops delivered?
8. Does the pinned snapshot of gpui under gpui-kit cause any friction?

Exit check (go, no go, or adjust): run the Phase 0 files again and write the results in `docs/gpui-spike-results.md`. Go only if load time and scrolling match or beat Iced, and the answers to questions 3, 5, and 7 are workable. The decision is Mau's.

### Phase 3: Feature parity, one PR each (L)

Order is chosen so the riskiest items come first.
1. Selection, keyboard navigation, and copy of value, key, and JSON path.
2. Toolbar and search: input, case and regex toggles, previous and next, opening the path to a result, scrolling to it, inline match highlighting.
3. Themes: dark and light with syntax colors, plus the Cmd+T toggle.
4. Context menu: copy options, export, open or close all children, always acting on the node under the cursor.
5. Status bar, welcome screen, and parse error screen with line numbers.
6. Dialogs: shortcut help, update check, CLI install result.

Exit check: every item in the README feature list works, core tests still pass, and a manual pass on macOS finds no regressions.

### Phase 4: Platform behavior (M, riskiest on macOS)

1. Native menu bar, macOS first. Decide between GPUI's own menus and muda. Pick an approach for Windows and Linux, since gpui-kit lists no menu bar component.
2. Finder open events, file drop, and CLI argument handling. Drop `macos_open.rs` if GPUI makes it unnecessary.
3. New windows: stay with one process per window, or move to several windows in one process. Keep `LaunchOptions` only if processes stay.
4. Window title, icon, and default size.
5. Smoke test on Windows and Linux.

Exit check: the two completed plans act as the acceptance tests. Paste works with Cmd+V, right click targets the right node, opening a file never spawns processes in a loop, and new windows cascade.

### Phase 5: Packaging and release (M)

1. Bundle, document types script, codesign, and notarize for the GPUI binary.
2. Windows icon and resource, and Linux packages in CI.
3. Ship as a pre release (2.0.0 beta) next to the Iced build and use it daily.
4. After a stable stretch, make GPUI the default, remove `unfold-iced`, and update README, CLAUDE.md, and DESIGN_DECISIONS.md (the Iced decision becomes superseded).

## Decisions Needed

1. Use full gpui-kit, or bare gpui plus gpui-base primitives? This plan assumes full gpui-kit for the spike, since it already has the toolbar, dialog, menu, and theme pieces.
2. Three crate workspace, or one crate with a library and two binaries? This plan assumes the workspace.
3. Keep shipping the Iced build as a fallback during the beta?
4. Is selectable text in rows a goal? It affects how rows are built in Phase 2.

## Risks

* GPUI is tied to Zed and gpui-kit is at 0.7.x, so the API can change between releases. Pin exact versions and upgrade on purpose.
* Linux builds need extra system packages that are not yet verified.
* macOS native behavior (menus, open events) is the most likely place to lose time.
* GPUI uses entities, contexts, and actions in place of a `Message` enum and one `update` function. The spike keeps that learning small before anything big is committed.
* The `Tree` versus `FlatRow` choice above is untested until the spike runs.

## What Does NOT Change

* JSON parsing and tree storage.
* Search matching and highlighting rules.
* Copy and export formats.
* Config file format.
* The feature set and the performance goals.

## Future Improvements

* A CLI wrapper on `unfold-core`.
* Single process multiple windows, if Phase 4 shows it is simpler.
* UI automation tests using gpui-kit's headless test support.
