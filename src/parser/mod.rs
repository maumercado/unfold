// Declare the node submodule
pub mod node;
pub mod tree;
pub mod builder;

// Re-export for easier access (optional but convenient)
#[allow(unused_imports)]  // May be used by tests or future code
pub use node::JsonNode;
pub use node::JsonValue;
pub use tree::JsonTree;
pub use builder::build_tree;
