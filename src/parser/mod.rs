// Declare the node submodule
pub mod node;
pub mod tree;

// Re-export for easier access (optional but convenient)
pub use node::JsonNode;
pub use node::JsonValue;
pub use tree::JsonTree;
