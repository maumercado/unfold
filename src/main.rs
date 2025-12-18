// Declare the parser module
mod parser;

fn main() {
  println!("Unfold - JSON Viewer");

  // Quick test that our types work
  let node = parser::JsonNode {
      key: None,
      node_type: parser::JsonType::Object,
      depth: 0,
      children: vec![],
  };

  println!("Created node: {:?}", node);
}
