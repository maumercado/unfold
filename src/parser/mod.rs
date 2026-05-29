// Declare the node submodule
pub mod builder;
pub mod node;
pub mod tree;

// Re-export for easier access (optional but convenient)
pub use builder::build_tree;
#[allow(unused_imports)] // May be used by tests or future code
pub use node::JsonNode;
pub use node::JsonValue;
pub use tree::JsonTree;
