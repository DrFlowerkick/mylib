// backtracks from node to root of tree

use super::TreeNode;
use std::rc::Rc;

pub struct BackTrack<N> {
    current_node: Rc<TreeNode<N>>,
    finished: bool, // true if iterator finished
}

impl<N: PartialEq> BackTrack<N> {
    pub fn new(node: Rc<TreeNode<N>>) -> Self {
        BackTrack {
            current_node: node,
            finished: false,
        }
    }
}

impl<N: PartialEq> Iterator for BackTrack<N> {
    type Item = Rc<TreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        let result = self.current_node.get_self();
        match self.current_node.get_parent() {
            Some(node) => self.current_node = node,
            None => self.finished = true,
        }
        result
    }
}
