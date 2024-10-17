// pre order traversal of tree

use super::TreeNode;
use std::rc::Rc;

pub struct PreOrderTraversal<N> {
    current_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    parent_ids: Vec<usize>,    // vector of parent ids while traveling through tree
    vertical: bool,            // false: children, true: parent
    iter_finished: bool,
}

impl<N: PartialEq> PreOrderTraversal<N> {
    pub fn new(root: Rc<TreeNode<N>>) -> Self {
        PreOrderTraversal {
            current_node: root,
            child_indices: vec![0],
            parent_ids: vec![],
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
        let result = Some(self.current_node.clone());
        // prepare for next iteration
        loop {
            if self.vertical {
                // in direction of parent
                match self.parent_ids.pop() {
                    Some(parent_id) => {
                        self.child_indices.pop();
                        let last_child_index = self.child_indices.len() - 1;
                        self.child_indices[last_child_index] += 1;
                        self.current_node = self.current_node.get_parent_by_id(parent_id).unwrap();
                        self.vertical = false;
                    }
                    None => {
                        // end of tree (or subtree, which started at initial given root)
                        self.iter_finished = true;
                        break;
                    }
                }
            } else {
                // in direction of children
                let child_index = self.child_indices[self.child_indices.len() - 1];
                match self.current_node.get_child(child_index) {
                    Some(node) => {
                        self.parent_ids.push(self.current_node.get_id());
                        self.current_node = node;
                        self.child_indices.push(0);
                        break;
                    }
                    None => self.vertical = true,
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tree_node::tests::setup_test_tree, *};

    #[test]
    fn test_pre_order_traversal() {
        let test_tree = setup_test_tree();

        assert_eq!(
            PreOrderTraversal::new(test_tree.clone())
                .filter(|n| n.is_leave())
                .count(),
            4
        );

        let pre_order_vector: Vec<char> = PreOrderTraversal::new(test_tree.clone())
            .map(|n| *n.get_value())
            .collect();
        assert_eq!(
            pre_order_vector,
            ['F', 'B', 'A', 'D', 'C', 'E', 'G', 'I', 'H']
        );

        let child_b = test_tree.get_node(&'B').unwrap();
        let pre_order_vector: Vec<char> = PreOrderTraversal::new(child_b)
            .map(|n| *n.get_value())
            .collect();
        assert_eq!(pre_order_vector, ['B', 'A', 'D', 'C', 'E']);
    }
}
