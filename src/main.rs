mod parser;

fn main() {
  println!("Unfold - JSON Viewer");

  // Quick test that our types work
  let node = parser::JsonNode {
      key: None,
      value: parser::JsonValue::Object,
      depth: 0,
      children: vec![],
  };

  println!("Created node: {:?}", node);
}
