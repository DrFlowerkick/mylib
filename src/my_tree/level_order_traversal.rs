// level order traversal of tree

use super::TreeNode;
use std::rc::Rc;

pub struct LevelOrderTraversal<N> {
    current_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    parent_ids: Vec<usize>,    // vector of parent ids while traveling through tree
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
            parent_ids: vec![],
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
            if self.vertical {
                // in direction of parent
                match self.parent_ids.pop() {
                    Some(parent_id) => {
                        self.child_indices.pop();
                        let last_child_index = self.child_indices.len() - 1;
                        self.child_indices[last_child_index] += 1;
                        self.current_node = self.current_node.get_parent_by_id(parent_id).unwrap();
                    }
                    None => {
                        // root of sub tree
                        if self.node_on_target_level {
                            if self.increment_target_level() {
                                return None;
                            }
                            self.node_on_target_level = false;
                            assert_eq!(self.child_indices.len(), 1);
                            self.child_indices[0] = 0; // reset index
                        } else {
                            // no more children of root to search for target_level
                            self.finished = true;
                            return None;
                        }
                    }
                }
                self.vertical = false;
            } else {
                // in direction of child
                // reached target level?
                if self.child_indices.len() - 1 == self.target_level {
                    self.node_on_target_level = true;
                    self.vertical = true;
                    return self.current_node.get_self().map(|n| (n, self.target_level));
                }
                let child_index = self.child_indices[self.child_indices.len() - 1];
                match self.current_node.get_child(child_index) {
                    Some(node) => {
                        self.parent_ids.push(self.current_node.get_id());
                        self.current_node = node;
                        self.child_indices.push(0);
                    }
                    None => self.vertical = true,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tree_node::tests::setup_test_tree, *};

    #[test]
    fn test_level_order_traversal() {
        let test_tree = setup_test_tree();
        
        assert_eq!(
            LevelOrderTraversal::new(test_tree.clone(), 0, None)
                .filter(|(n, _)| n.is_leave())
                .count(),
            4
        );
        
        let level_order_vector: Vec<char> = LevelOrderTraversal::new(test_tree.clone(), 0, None)
            .map(|(n, _)| *n.get_value())
            .collect();
        assert_eq!(
            level_order_vector,
            ['F', 'B', 'G', 'A', 'D', 'I', 'C', 'E', 'H']
        );
        
        let child_b = test_tree.get_node(&'B').unwrap();
        let level_order_vector: Vec<char> = LevelOrderTraversal::new(child_b, 0, None)
            .map(|(n, _)| *n.get_value())
            .collect();
        assert_eq!(level_order_vector, ['B', 'A', 'D', 'C', 'E']);
        
        let level_order_vector: Vec<char> = LevelOrderTraversal::new(test_tree.clone(), 2, None)
            .map(|(n, _)| *n.get_value())
            .collect();
        assert_eq!(level_order_vector, ['A', 'D', 'I', 'C', 'E', 'H']);
        
        let level_order_vector: Vec<char> = LevelOrderTraversal::new(test_tree.clone(), 1, Some(2))
            .map(|(n, _)| *n.get_value())
            .collect();
        assert_eq!(level_order_vector, ['B', 'G', 'A', 'D', 'I']);
    }
}
