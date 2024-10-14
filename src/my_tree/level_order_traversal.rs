// level order traversal of tree

use super::TreeNode;
use std::rc::Rc;

pub struct LevelOrderTraversal<N> {
    current_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    vertical: bool,            // false: children, true: parent
    finished: bool,            // true if iterator finished
    target_level: usize,
    end_level: Option<usize>,
    node_on_target_level: bool,
}

impl<N: PartialEq> LevelOrderTraversal<N> {
    pub fn new(root: Rc<TreeNode<N>>, start_level: usize, end_level: Option<usize>) -> Self {
        let ci_capacity = match end_level {
            Some(level) => {
                if start_level > level {
                    panic!("end_level must be >= start_level.");
                }
                level + 1
            }
            None => 1,
        };
        let mut child_indices: Vec<usize> = Vec::with_capacity(ci_capacity);
        child_indices.push(0);
        LevelOrderTraversal {
            current_node: root,
            child_indices,
            vertical: false,
            finished: false,
            target_level: start_level,
            end_level,
            node_on_target_level: false,
        }
    }
    fn increment_target_level(&mut self) -> bool {
        if let Some(level) = self.end_level {
            if self.target_level == level {
                self.finished = true;
                return true;
            }
        }
        self.target_level += 1;
        false
    }
}

impl<N: PartialEq> Iterator for LevelOrderTraversal<N> {
    type Item = (Rc<TreeNode<N>>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        loop {
            if self.child_indices.len() - 1 == self.target_level {
                let result = self.current_node.get_self().map(|n| (n, self.target_level));
                if self.target_level == 0 {
                    if self.increment_target_level() {
                        return None;
                    }
                } else if let Some(node) = self.current_node.get_parent() {
                    self.node_on_target_level = true;
                    self.vertical = true;
                    self.child_indices.pop();
                    self.current_node = node;
                }
                return result;
            }
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
                        if self.child_indices.len() == 1 {
                            // root of sub tree
                            if self.node_on_target_level {
                                if self.increment_target_level() {
                                    return None;
                                }
                                self.node_on_target_level = false;
                                self.child_indices[0] = 0; // reset index
                            } else {
                                // no more children of root to search for target_level
                                self.finished = true;
                                return None;
                            }
                        } else if let Some(node) = self.current_node.get_parent() {
                            self.vertical = true;
                            self.child_indices.pop();
                            self.current_node = node;
                        }
                    }
                }
            }
        }
    }
}
