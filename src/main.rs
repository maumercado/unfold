mod parser;

use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Unfold - JSON Viewer\n");

    // Get filename from command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: unfold <filename.json>");
        println!("\nExample: cargo run -- test.json");
        return Ok(());
    }

    let filename = &args[1];
    println!("📄 Reading: {}\n", filename);

    // Read the file
    let contents = fs::read_to_string(filename)?;

    // Parse JSON with serde_json
    let json_value: serde_json::Value = serde_json::from_str(&contents)?;

    // Build our tree
    let tree = parser::build_tree(&json_value);

    // Print stats
    println!("✓ Parsed successfully!");
    println!("  Nodes: {}", tree.node_count());
    println!();

    // Print the tree
    println!("📊 Tree structure:\n");
    println!("{}", tree.print_tree());

    Ok(())
}
