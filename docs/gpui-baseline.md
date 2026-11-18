# Baseline Measurements (Phase 0)

Numbers for the current Iced build, taken before any GPUI work, so later changes can be compared against them. See `gpui-migration-plan.md` for the plan these belong to.

## How to reproduce

```bash
python3 scripts/gen_test_json.py                      # writes bench-data/ (ignored by git)
cargo run --release -p unfold-core --example bench -- bench-data/bench-100mb.json
```

The generator is deterministic, so the same file comes out on every machine. The benchmark runs the same steps the app runs when it opens a file (read, parse, build tree, flatten), plus a search and an expand-all. It runs without a window, so it measures the core crate only.

## Headless results (core crate)

Machine: Linux container, 2 vCPU, 7 GB RAM. Release build. Run once per file, so treat small differences as noise.

| Step | 10 MB | 100 MB |
|---|---|---|
| Read file | 0.010 s | 0.074 s |
| `serde_json` parse | 0.150 s | 1.284 s |
| Build tree | 0.160 s | 1.647 s |
| Flatten, root collapsed | 0.006 s | 0.089 s |
| Search "alpha" (keys and values) | 0.103 s | 0.913 s |
| Expand all | 0.035 s | 0.285 s |
| Flatten, all expanded | 0.410 s | 4.290 s |
| Nodes | 744,266 | 7,415,685 |
| Rows, all expanded | 744,265 | 7,415,684 |
| Peak memory at end of load | 278 MB | 2,742 MB |
| Peak memory / file size | 27.8x | 27.4x |

The 500 MB file was not run. At this memory ratio it needs about 14 GB.

## What the numbers say

* **Load time misses the target.** DESIGN_DECISIONS.md aims for under 2 seconds for 100 MB. Parse plus tree build is about 2.9 seconds here, before the window draws anything.
* **Memory misses the target by a lot.** The goal is 1.2x to 1.5x the file size. It is about 27x. `build_tree` takes a complete `serde_json::Value` (about 1.5 GB for 100 MB), then builds a second copy as `JsonNode`s, each with its own `String` key, `String` value, and `Vec` of children. The streaming parser described in DESIGN_DECISIONS.md was never built.
* **Dropping the `Value` did not lower resident memory** in this run (2,742 MB before, 2,737 MB after). The allocator probably keeps the freed pages. This is a measurement caveat, not proof the memory is still in use.
* **Flattening everything is slow** (4.3 s for 7.4 million rows), but the app only flattens expanded nodes, and a fresh file starts collapsed, so the common path is the 0.089 s case.

These costs live in `unfold-core`, so a GPUI port would not fix them and would not make them worse. If large files matter, a core level fix (a leaner node layout, interned keys, or a streaming parse) pays off in either UI and can be done before or after the port.

## Still to measure by hand (needs a window)

The container has no display, so these were not measured. Fill them in on a real machine with the 10 MB and 100 MB files, using `cargo run --release`.

| Check | Iced build |
|---|---|
| Time from file open to first rows visible | |
| Memory shown by Activity Monitor after open | |
| Scroll through 100 MB file, expanded at the first level: smooth or visible stutter | |
| Expand a node with about 150,000 children | |
| Search "alpha", time to first result, then Next | |
| Machine used | |

These are the numbers the GPUI spike (Phase 2) has to match or beat.
