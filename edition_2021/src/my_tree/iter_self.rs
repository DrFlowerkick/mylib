// returns reference to node
// not sure, if this is really needed anywhere

use super::TreeNode;
use std::rc::Rc;

pub struct IterSelf<N> {
    node: Rc<TreeNode<N>>,
    finished: bool, // true if iterator finished
}

impl<N: PartialEq> IterSelf<N> {
    pub fn new(node: Rc<TreeNode<N>>) -> Self {
        IterSelf {
            node,
            finished: false,
        }
    }
}

impl<N: PartialEq> Iterator for IterSelf<N> {
    type Item = Rc<TreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        self.finished = true;
        Some(self.node.clone())
    }
}
