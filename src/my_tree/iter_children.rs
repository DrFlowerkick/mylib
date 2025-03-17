// iterate over children of node

use super::TreeNode;
use std::rc::Rc;

pub struct IterChildren<N> {
    node: Rc<TreeNode<N>>,
    len_children: usize,
    child_index: usize,
    finished: bool, // true if iterator finished
}

impl<N: PartialEq> IterChildren<N> {
    pub fn new(node: Rc<TreeNode<N>>) -> Self {
        let len_children = node.len_children();
        IterChildren {
            node,
            len_children,
            child_index: 0,
            finished: false,
        }
    }
}

impl<N: PartialEq> Iterator for IterChildren<N> {
    type Item = Rc<TreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        match self.node.get_child(self.child_index) {
            Some(node) => {
                self.child_index += 1;
                Some(node)
            }
            None => {
                self.finished = true;
                None
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len_children))
    }
}

impl<N: PartialEq + Copy + Clone> ExactSizeIterator for IterChildren<N> {
    fn len(&self) -> usize {
        self.len_children
    }
}
