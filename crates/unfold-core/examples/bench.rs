//! Headless benchmark of the work Unfold does when it opens a file.
//!
//! Run it in release mode, since debug builds are far slower:
//!
//!     python3 scripts/gen_test_json.py
//!     cargo run --release -p unfold-core --example bench -- bench-data/bench-100mb.json
//!
//! It measures the same steps the app does on load (read, parse, build tree,
//! flatten) plus a search and an expand-all. It cannot measure rendering or
//! scroll smoothness; those need a window and are measured by hand.
//!
//! Memory numbers come from /proc/self/status, so they are only printed on Linux.

use std::env;
use std::fs;
use std::time::{Duration, Instant};

use unfold_core::flat_row::flatten_visible_nodes;
use unfold_core::parser::builder::build_tree;
use unfold_core::search::search_nodes;

/// Read a field such as "VmRSS" or "VmHWM" from /proc/self/status, in MB.
fn proc_status_mb(field: &str) -> Option<f64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with(field))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024.0)
}

fn mem_mb(mb: Option<f64>) -> String {
    match mb {
        Some(mb) => format!("{:>9.0} MB", mb),
        None => "      n/a".to_string(),
    }
}

fn mem(field: &str) -> String {
    mem_mb(proc_status_mb(field))
}

fn secs(d: Duration) -> String {
    format!("{:>8.3} s", d.as_secs_f64())
}

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: bench <file.json> [search term]");
        std::process::exit(2);
    };
    let term = env::args().nth(2).unwrap_or_else(|| "alpha".to_string());

    println!("file: {}", path);

    let t = Instant::now();
    let contents = fs::read_to_string(&path).expect("could not read file");
    let read = t.elapsed();
    let file_mb = contents.len() as f64 / (1024.0 * 1024.0);
    println!("size: {:.1} MB", file_mb);

    let t = Instant::now();
    let value: serde_json::Value = serde_json::from_str(&contents).expect("invalid JSON");
    let parse = t.elapsed();
    drop(contents);
    let rss_value = mem("VmRSS");

    let t = Instant::now();
    let mut tree = build_tree(&value);
    let build = t.elapsed();
    let rss_both = mem("VmRSS");
    let peak_load_mb = proc_status_mb("VmHWM");
    drop(value);
    let rss_tree = mem("VmRSS");

    let t = Instant::now();
    let rows = flatten_visible_nodes(&tree);
    let flatten_collapsed = t.elapsed();

    let t = Instant::now();
    let (results, _) = search_nodes(&tree, &term, false, false);
    let search = t.elapsed();

    // Expand-all can be huge on a big file, so it is last and uses its own rows.
    let root = tree.root_index();
    let t = Instant::now();
    tree.set_expanded_recursive(root, true);
    let expand = t.elapsed();
    let t = Instant::now();
    let all_rows = flatten_visible_nodes(&tree);
    let flatten_expanded = t.elapsed();

    println!();
    println!("{:<28}{}", "read file", secs(read));
    println!("{:<28}{}", "serde_json parse", secs(parse));
    println!("{:<28}{}", "build tree", secs(build));
    println!(
        "{:<28}{}",
        "flatten (root collapsed)",
        secs(flatten_collapsed)
    );
    println!("{:<28}{}", format!("search \"{}\"", term), secs(search));
    println!("{:<28}{}", "expand all", secs(expand));
    println!("{:<28}{}", "flatten (all expanded)", secs(flatten_expanded));
    println!();
    println!("{:<28}{}", "nodes", tree.node_count());
    println!("{:<28}{}", "rows (collapsed)", rows.len());
    println!("{:<28}{}", "rows (all expanded)", all_rows.len());
    println!("{:<28}{}", "search results", results.len());
    println!();
    println!("{:<28}{}", "RSS after parse (Value)", rss_value);
    println!("{:<28}{}", "RSS tree + Value", rss_both);
    println!("{:<28}{}", "RSS tree only", rss_tree);
    println!("{:<28}{}", "peak RSS (end of load)", mem_mb(peak_load_mb));
    println!("{:<28}{}", "peak RSS (whole run)", mem("VmHWM"));
    if let Some(peak) = peak_load_mb {
        println!("{:<28}{:>9.2}x", "load peak / file size", peak / file_mb);
    }
}
