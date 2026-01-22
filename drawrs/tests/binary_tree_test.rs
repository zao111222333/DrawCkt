use drawrs::diagram_types::binary_tree::{BinaryNodeObject, BinaryTreeDiagram};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_binary_node_basic() {
    let parent = Rc::new(RefCell::new(BinaryNodeObject::new("P".to_string())));
    let child = Rc::new(RefCell::new(BinaryNodeObject::new("C".to_string())));

    assert_eq!(parent.borrow().value(), "P");
    assert_eq!(child.borrow().value(), "C");
}

#[test]
fn test_binary_tree_diagram_new() {
    let diagram = BinaryTreeDiagram::new();
    assert_eq!(diagram.objects().len(), 0);
}

#[test]
fn test_add_left() {
    let mut diagram = BinaryTreeDiagram::new();
    let parent = Rc::new(RefCell::new(BinaryNodeObject::new("P".to_string())));
    let child = Rc::new(RefCell::new(BinaryNodeObject::new("C".to_string())));

    let result = diagram.add_left(&parent, child.clone());
    assert!(result.is_ok());
    assert!(parent.borrow().left().is_some());
    assert_eq!(child.borrow().tree_parent().is_some(), true);
}

#[test]
fn test_left_set_and_remove() {
    let parent = Rc::new(RefCell::new(BinaryNodeObject::new("P".to_string())));
    let child = Rc::new(RefCell::new(BinaryNodeObject::new("C".to_string())));

    BinaryNodeObject::set_left(&parent, Some(child.clone())).unwrap();
    assert!(parent.borrow().left().is_some());
    assert_eq!(child.borrow().tree_parent().is_some(), true);

    // Remove left
    BinaryNodeObject::set_left(&parent, None).unwrap();
    assert!(parent.borrow().left().is_none());
    assert_eq!(child.borrow().tree_parent().is_none(), true);
}
