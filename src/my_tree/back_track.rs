// backtracks from node to root of tree
// Since multiple parents are possible, multiple backtrack paths are possible, too.
// Since all parents of one node are always on the same level, all backtrack
// paths have the same length. Therefore backtrack returns on each iteration all
// nodes of one level of backtracking.

use super::TreeNode;

use std::collections::HashSet;
use std::rc::Rc;

pub struct BackTrack<N> {
    current_nodes: Vec<Rc<TreeNode<N>>>,
    finished: bool, // true if iterator finished
}

impl<N: PartialEq> BackTrack<N> {
    pub fn new(node: Rc<TreeNode<N>>) -> Self {
        BackTrack {
            current_nodes: vec![node],
            finished: false,
        }
    }
}

impl<N: PartialEq> Iterator for BackTrack<N> {
    type Item = Vec<Rc<TreeNode<N>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        // backtrack iterator always starts with initial node as return value (collected in a vec).
        let result = Some(self.current_nodes.clone());
        // When backtrack splits into multiple nodes, which happens if a node does have multiple parents,
        // at some level of the tree the different backtrack paths will meet again at a node.
        // Therefore backtrack has to filter for duplicate parents, which is done with seen HashSet.
        let mut seen = HashSet::new();
        self.current_nodes = self
            .current_nodes
            .iter()
            .map(|c| c.iter_parents())
            .flatten()
            .filter(|n| seen.insert(n.get_id()))
            .collect();
        self.finished = self.current_nodes.is_empty();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tree_node::tests::setup_test_tree, *};

    #[test]
    fn test_backtrack() {
        let test_tree = setup_test_tree();

        let child_h = test_tree.get_node(&'H').unwrap();
        let mut backtrack_iterator = BackTrack::new(child_h).map(|p| p[0]);
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'H');
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'I');
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'G');
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'F');
        assert!(backtrack_iterator.next().is_none());
    }
}
