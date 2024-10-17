// post order traversal of tree

use super::TreeNode;
use std::rc::Rc;

pub struct PostOrderTraversal<N> {
    current_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    parent_ids: Vec<usize>,    // vector of parent ids while traveling through tree
    vertical: bool,            // false: children, true: parent
    finished: bool,            // true if iterator finished
}

impl<N: PartialEq> PostOrderTraversal<N> {
    pub fn new(root: Rc<TreeNode<N>>) -> Self {
        PostOrderTraversal {
            current_node: root,
            child_indices: vec![0],
            parent_ids: vec![],
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
                // in direction of parent: return current node and prepare for next iteration
                let result = self.current_node.get_self();
                match self.parent_ids.pop() {
                    Some(parent_id) => {
                        self.current_node = self.current_node.get_parent_by_id(parent_id).unwrap();
                        self.child_indices.pop();
                        let last_child_index = self.child_indices.len() - 1;
                        self.child_indices[last_child_index] += 1;
                        self.vertical = false;
                    }
                    // end of tree (or subtree, which started at initial given root)
                    None => self.finished = true,
                }
                return result;
            } else {
                // in direction of child
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
    fn test_post_order_traversal() {
        let test_tree = setup_test_tree();

        assert_eq!(
            PostOrderTraversal::new(test_tree.clone())
                .filter(|n| n.is_leave())
                .count(),
            4
        );

        let post_order_vector: Vec<char> = PostOrderTraversal::new(test_tree.clone())
            .map(|n| *n.get_value())
            .collect();
        assert_eq!(
            post_order_vector,
            ['A', 'C', 'E', 'D', 'B', 'H', 'I', 'G', 'F']
        );

        let child_b = test_tree.get_node(&'B').unwrap();
        let post_order_vector: Vec<char> = PostOrderTraversal::new(child_b)
            .map(|n| *n.get_value())
            .collect();
        assert_eq!(post_order_vector, ['A', 'C', 'E', 'D', 'B']);
    }
}
