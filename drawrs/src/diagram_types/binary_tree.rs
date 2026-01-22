use crate::diagram::Object;
use crate::error::{DrawrsError, DrawrsResult};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub struct BinaryNodeObject {
    value: String,
    tree_children: Vec<Option<Rc<RefCell<BinaryNodeObject>>>>,
    tree_parent: Weak<RefCell<BinaryNodeObject>>,
}

impl BinaryNodeObject {
    pub fn new(value: String) -> Self {
        Self {
            value,
            tree_children: vec![None, None], // Always exactly 2 slots for left and right
            tree_parent: Weak::new(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    fn ensure_two_slots(&mut self) {
        while self.tree_children.len() < 2 {
            self.tree_children.push(None);
        }
        if self.tree_children.len() > 2 {
            self.tree_children.truncate(2);
        }
    }

    fn detach_from_old_parent(node: &Rc<RefCell<BinaryNodeObject>>) {
        let parent = node.borrow().tree_parent.upgrade();
        if let Some(parent_rc) = parent {
            let mut parent_borrow = parent_rc.borrow_mut();
            for child in &mut parent_borrow.tree_children {
                if let Some(child_rc) = child {
                    if Rc::ptr_eq(child_rc, node) {
                        *child = None;
                        break;
                    }
                }
            }
        }
        node.borrow_mut().tree_parent = Weak::new();
    }

    fn assign_child(
        &mut self,
        index: usize,
        node: Option<Rc<RefCell<BinaryNodeObject>>>,
        self_rc: &Rc<RefCell<BinaryNodeObject>>,
    ) -> DrawrsResult<()> {
        self.ensure_two_slots();

        if let Some(ref new_node) = node {
            // Check if we already have 2 distinct children
            let existing_count = self
                .tree_children
                .iter()
                .filter(|child| child.is_some())
                .count();

            if existing_count >= 2 {
                // Check if the new node is already a child
                let is_existing = self.tree_children.iter().any(|child| {
                    if let Some(child_rc) = child {
                        Rc::ptr_eq(child_rc, new_node)
                    } else {
                        false
                    }
                });

                if !is_existing {
                    return Err(DrawrsError::TooManyChildren);
                }
            }

            // Detach from old parent
            Self::detach_from_old_parent(new_node);

            // Clear existing slot if node was in the other slot
            let other_index = 1 - index;
            if let Some(ref other_child) = self.tree_children[other_index] {
                if Rc::ptr_eq(other_child, new_node) {
                    self.tree_children[other_index] = None;
                }
            }

            // Set parent reference
            new_node.borrow_mut().tree_parent = Rc::downgrade(self_rc);
        } else {
            // Setting to None - clear the slot
            if let Some(ref old_child) = self.tree_children[index] {
                old_child.borrow_mut().tree_parent = Weak::new();
            }
        }

        self.tree_children[index] = node;
        Ok(())
    }

    pub fn left(&self) -> Option<Rc<RefCell<BinaryNodeObject>>> {
        self.tree_children
            .get(0)
            .and_then(|opt| opt.as_ref().map(|rc| Rc::clone(rc)))
    }

    pub fn set_left(
        self_rc: &Rc<RefCell<BinaryNodeObject>>,
        node: Option<Rc<RefCell<BinaryNodeObject>>>,
    ) -> DrawrsResult<()> {
        self_rc.borrow_mut().assign_child(0, node, self_rc)
    }

    pub fn right(&self) -> Option<Rc<RefCell<BinaryNodeObject>>> {
        self.tree_children
            .get(1)
            .and_then(|opt| opt.as_ref().map(|rc| Rc::clone(rc)))
    }

    pub fn set_right(
        self_rc: &Rc<RefCell<BinaryNodeObject>>,
        node: Option<Rc<RefCell<BinaryNodeObject>>>,
    ) -> DrawrsResult<()> {
        self_rc.borrow_mut().assign_child(1, node, self_rc)
    }

    pub fn tree_parent(&self) -> Option<Rc<RefCell<BinaryNodeObject>>> {
        self.tree_parent.upgrade()
    }
}

pub struct BinaryTreeDiagram {
    objects: Vec<Object>,
    root: Option<Rc<RefCell<BinaryNodeObject>>>,
}

impl BinaryTreeDiagram {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            root: None,
        }
    }

    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn add_left(
        &mut self,
        parent: &Rc<RefCell<BinaryNodeObject>>,
        child: Rc<RefCell<BinaryNodeObject>>,
    ) -> DrawrsResult<()> {
        BinaryNodeObject::set_left(parent, Some(child))
    }

    pub fn add_right(
        &mut self,
        parent: &Rc<RefCell<BinaryNodeObject>>,
        child: Rc<RefCell<BinaryNodeObject>>,
    ) -> DrawrsResult<()> {
        BinaryNodeObject::set_right(parent, Some(child))
    }

    pub fn from_dict(data: &HashMap<String, Vec<Option<String>>>) -> DrawrsResult<Self> {
        if data.len() != 1 {
            return Err(DrawrsError::InvalidRootDict);
        }

        let mut diagram = Self::new();
        // Simplified implementation - would need recursive parsing
        Ok(diagram)
    }
}

impl Default for BinaryTreeDiagram {
    fn default() -> Self {
        Self::new()
    }
}
