// pre order traversal of tree

use super::TreeNode;
use std::rc::Rc;

pub struct PreOrderTraversal<N> {
    next_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    vertical: bool,            // false: children, true: parent
    iter_finished: bool,
}

impl<N: PartialEq> PreOrderTraversal<N> {
    pub fn new(root: Rc<TreeNode<N>>) -> Self {
        PreOrderTraversal {
            next_node: root,
            child_indices: vec![],
            vertical: false,
            iter_finished: false,
        }
    }
}

impl<N: PartialEq> Iterator for PreOrderTraversal<N> {
    type Item = Rc<TreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_finished {
            return None;
        }
        loop {
            if self.vertical {
                // in direction of parent
                match self.next_node.get_parent() {
                    Some(node) => {
                        self.child_indices.pop();
                        if self.child_indices.is_empty() {
                            break; // end of subtree, which started at given "root" node
                        }
                        let last_index = self.child_indices.len() - 1;
                        self.child_indices[last_index] += 1;
                        self.next_node = node;
                        self.vertical = false;
                    }
                    None => break, // end of tree
                }
            } else {
                // in direction of children
                if self.child_indices.is_empty() {
                    self.child_indices.push(0);
                    return Some(self.next_node.clone());
                }
                let child_index = self.child_indices[self.child_indices.len() - 1];
                match self.next_node.get_child(child_index) {
                    Some(node) => {
                        self.next_node = node;
                        self.child_indices.push(0);
                        return Some(self.next_node.clone());
                    }
                    None => self.vertical = true,
                }
            }
        }
        self.iter_finished = true;
        None
    }
}
