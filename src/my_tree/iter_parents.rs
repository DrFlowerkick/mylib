// iterate over parents of node

use super::TreeNode;
use std::rc::Rc;

pub struct IterParents<N> {
    node: Rc<TreeNode<N>>,
    len_parents: usize,
    parent_index: usize,
    finished: bool, // true if iterator finished
}

impl<N: PartialEq> IterParents<N> {
    pub fn new(node: Rc<TreeNode<N>>) -> Self {
        let len_parents = node.len_parents();
        IterParents {
            node,
            len_parents,
            parent_index: 0,
            finished: false,
        }
    }
}

impl<N: PartialEq> Iterator for IterParents<N> {
    type Item = Rc<TreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        // loop over parents. Since parents are weak, some parents may have gone out of scope
        while self.parent_index < self.len_parents {
            if let Some(parent) = self.node.get_parent(self.parent_index) {
                self.parent_index += 1;
                return Some(parent);
            }
            self.parent_index += 1;
        }
        self.finished = true;
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len_parents))
    }
}

impl<N: PartialEq + Copy + Clone> ExactSizeIterator for IterParents<N> {
    fn len(&self) -> usize {
        self.len_parents
    }
}
