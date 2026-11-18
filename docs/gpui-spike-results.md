# GPUI Spike Results (Phase 2)

Answers to the Phase 2 questions in `gpui-migration-plan.md`, from building `crates/unfold-gpui` on gpui-kit 0.10.0. That crate has since become the root `unfold` app, and the Iced app was removed. Iced numbers below were taken before the removal.

Status: **automated checks done, hand checks still open.** Everything below marked *measured* or *tested* was run on this branch. Anything that needs eyes on a real window is listed at the end. Mau makes the go or no-go call.

Machine for the measurements: macOS on Apple Silicon. Release builds.

## Versions

* gpui-kit is now **0.10.0**, not the 0.7.3 the plan was written against. It pulls in the `gpui-pre-*` 0.3.12 crate family (GPUI published by the gpui-kit authors) and gpui-component 0.10.0.
* Pinned exactly: `gpui-kit = "=0.10.0"`.

## Answers

### 1. Does the list stay smooth at 1M and 10M rows?

The spike uses GPUI's own `uniform_list`, not gpui-kit's `VirtualList`. Both render only the rows in view. `VirtualList` needs a `Vec` with the size of every row (about 80 MB extra at 10M rows), which exists to support rows of different heights. Ours are all 16 px, so `uniform_list` is the better fit. That is a small change from the plan.

*Measured* (headless, `frame_time_with_a_million_rows` test): 1,000,000 rows, 200 frames mixing wheel scrolls and jumps of 100,000 rows. Average CPU time per frame was **0.9 ms**, worst **1.2 ms**. This covers layout and prepaint only. GPU drawing is not included because the test has no display.

The work per frame depends only on how many rows are visible, so 10M rows costs the same per frame. What grows with row count is flattening after expand or collapse. That cost is in `unfold-core` and is the same in both UIs.

### 2. Horizontal scrolling for long values

Yes. `uniform_list` with `ListHorizontalSizingBehavior::Unconstrained`, with its width measured from the widest row (`row::widest_row`, which counts chars, so `│` counts as one). One scrollbar element (gpui-component `Scrollbar`) covers both axes. *Needs a visual check.*

### 3. Scroll to a row from code

Yes: `UniformListScrollHandle::scroll_to_item(row, ScrollStrategy::Center)`. Search uses it for next and previous. Keyboard navigation uses `Nearest`. *Tested* indirectly by the search UI test.

### 4. Clipboard and selectable text

* Clipboard: `cx.write_to_clipboard` and `cx.read_from_clipboard`. *Tested*: Cmd+C on a selected row puts the value on the clipboard.
* Selectable text in rows is **not built**. gpui-base has `selectable_text`, so it is possible. Each row is a single `StyledText`, which is where selection would go. This is still open decision 4 in the plan.

### 5. Keyboard actions and a focused input (the PR 19 and PR 20 bug class)

This works by design. GPUI matches key bindings by *key context* (a tag on an element), and the deepest context wins. The search box has the `Input` context and binds its own `cmd-c` and `cmd-v`. The viewer has the `Viewer` context and binds `CopyValue` and `PasteIntoSearch` to the same keys. So:

* Search box focused: Cmd+C and Cmd+V edit the search text.
* Tree focused: Cmd+C copies the node, and Cmd+V pastes into search.

There is no "was the event captured?" check like the Iced app has. *Tested* by `search_box_keeps_its_own_copy_and_paste` and `cmd_c_copies_the_selected_value_when_the_tree_has_focus`.

One finding: the input handles Enter and then lets it continue to our `enter` binding. Handling the input's `PressEnter` event as well made search step twice, so the app only listens to its own binding. The test caught this.

### 6. Builds, packages, compile time, binary size

* macOS: builds and passes clippy with `-D warnings`. The first debug build of the GPUI crates took about 32 s on this machine.
* Release binary size: **28.3 MB** for GPUI, against **20.9 MB** for Iced.
* Linux: **not verified** (no Linux machine was available). The CI step installs the packages GPUI is documented to need: `libxkbcommon-x11-dev libwayland-dev libfontconfig-dev libx11-xcb-dev libvulkan-dev`. The first CI run on a PR is the real test.
* Windows: not verified locally. CI covers it.

### 7. Finder open events and file drops

* Finder, Dock and "Open With" deliver `file://` URLs through `Application::on_open_urls`. They are queued in a channel and handled once the app has started. The app reuses an empty window if there is one, otherwise it opens a new window. This replaces `macos_open.rs` (264 lines of AppKit delegate code), the `--spawn-handoff` flag, and the loop guard. Only the URL decoding is *tested*. *The Finder flow needs a hand check with a bundled `.app`.*
* File drop: `on_drop::<ExternalPaths>`. Several dropped files open in several windows. *Needs a hand check.*

### 8. Friction from gpui-kit's pinned GPUI

Each of these came up once and is handled:

* **Cargo feature unification changed JSON key order.** GPUI turns on `serde_json/preserve_order` for the whole build, so objects keep file order instead of being sorted. That broke a core test under `cargo test --workspace`. Fix: `unfold-core` now turns the feature on itself, so **both apps show keys in file order**. This is a visible change for the Iced app (it used to sort keys alphabetically). To go back, sort keys in `build_tree`.
* **wgpu lockfile conflict.** Turning on `gpui-kit/test-support` made Cargo also resolve the optional `gpui-fast` backend, which needs `wgpu =30.0.1`. Fixed with `cargo update -p wgpu --precise 30.0.1`. Watch for this when upgrading gpui-kit.
* `KeyBinding::new` does not understand the `secondary-` modifier, and panics at startup. Bindings now go through `KeyBinding::load`, which resolves it for the current OS. A unit test parses every binding for macOS, Linux and Windows.
* Test modules must not `use super::*` when the parent has `use gpui_kit::*`. The glob brings in GPUI's `test` macro, which shadows `#[test]`.

## Load and memory (*measured*, release, same files as Phase 0)

| | 10 MB | 100 MB |
|---|---|---|
| GPUI: read + parse + build + flatten | 76 ms | 737 ms |
| GPUI: resident memory after load | 341 MB | 2,517 MB |
| Iced: resident memory after load | 349 MB | 2,616 MB |

The GPUI build parses on a background thread and shows "Loading…" in the meantime. The Iced build blocks the window while it parses. Memory is about the same, as expected: it is dominated by the core tree (see `gpui-baseline.md`).

## What was built beyond the spike

The spike went further than its scope, into most of Phase 3, because the GPUI APIs made each feature small:

* Selection, click to expand, arrow key navigation (new: ↑ ↓ select, → ← expand and collapse)
* Copy value, key and path, copy minified or formatted, export as JSON, minified or formatted
* Toolbar search: case and regex toggles, previous and next, result count, inline match highlighting, auto expand and scroll
* Dark and light theme with the same palette, Cmd+T, saved to the same config file
* Native right click menu (gpui-component `NativeMenu`), acting on the row under the cursor (*tested*)
* Status bar, welcome screen, parse error screen with line, column and caret
* Dialogs: keyboard shortcuts (Cmd+/), About, Check for Updates, Install CLI result
* Native menu bar built from the same actions, so shortcut labels come from the key bindings
* Several windows in one process, cascaded by 32 px, with Cmd+N, Cmd+W and quit after the last window closes

## Still to check by hand

These need a person and a real window:

1. Scrolling the 100 MB file with first level open: smooth, or visible stutter? Compare with Iced.
2. Horizontal scroll on a row with a very long string.
3. Right click menu: shows natively, Copy Value As and Export Value As submenus work.
4. Menu bar: shortcut labels appear, and Cmd+C in the search box does not trigger Copy Value.
5. Drag and drop one file and several files.
6. Bundle with `cargo bundle` and open a file from Finder (cold start and while running).
7. Theme toggle: toolbar buttons, input and dialogs switch together.
