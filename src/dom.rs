use std::collections::HashMap;

// The DOM is a tree of nodes
#[derive(Debug)]
pub struct Node {
    children: Vec<Node>,
    node_type: NodeType,
}

// For simplicity just text or element nodes
#[derive(Debug)]
enum NodeType {
    Text(String),
    Element(ElementData),
    Comment()
}

#[derive(Debug)]
struct ElementData {
    tag_name: String,
    attrs: AttrMap,
}

pub type AttrMap = HashMap<String, String>;

pub fn text(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Text(data),
    }
}

pub fn elem(tag_name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData { tag_name, attrs }),
    }
}

pub fn comment() -> Node {
    Node {
        children: vec!{},
        node_type: NodeType::Comment(),
    }
}
