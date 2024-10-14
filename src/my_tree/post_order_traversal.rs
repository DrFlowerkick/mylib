// post order traversal of tree

use super::TreeNode;
use std::rc::Rc;

pub struct PostOrderTraversal<N> {
    current_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    vertical: bool,            // false: children, true: parent
    finished: bool,            // true if iterator finished
}

impl<N: PartialEq> PostOrderTraversal<N> {
    pub fn new(root: Rc<TreeNode<N>>) -> Self {
        PostOrderTraversal {
            current_node: root,
            child_indices: vec![0],
            vertical: false,
            finished: false,
        }
    }
}

impl<N: PartialEq> Iterator for PostOrderTraversal<N> {
    type Item = Rc<TreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        loop {
            if self.vertical {
                // in direction of parent
                let last_index = self.child_indices.len() - 1;
                self.child_indices[last_index] += 1;
                self.vertical = false;
            } else {
                // in direction of child
                let child_index = self.child_indices[self.child_indices.len() - 1];
                match self.current_node.get_child(child_index) {
                    Some(node) => {
                        self.current_node = node;
                        self.child_indices.push(0);
                    }
                    None => {
                        let result = self.current_node.get_self();
                        match self.current_node.get_parent() {
                            Some(node) => {
                                self.vertical = true;
                                self.child_indices.pop();
                                self.finished = self.child_indices.is_empty(); // root of subtree, which started at given "root" node
                                self.current_node = node;
                            }
                            None => self.finished = true,
                        }
                        return result;
                    }
                }
            }
        }
    }
}
