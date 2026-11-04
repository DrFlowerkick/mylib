// breadth first search

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree};
use std::collections::HashSet;
use std::collections::VecDeque;

pub struct BfsWalker<G, H, A, T>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
    T::NodeID: std::hash::Hash,
{
    queue: VecDeque<(T::PathNode, usize)>,
    skip: HashSet<T::NodeID>,
    visited: HashSet<T::NodeID>, // required because of transposition table
    max_depth: usize, // limit search depth to prevent long searches if tree does not contain state
    _phantom: std::marker::PhantomData<(G, H, A, T)>,
}

impl<G, H, A, T> BfsWalker<G, H, A, T>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
    T::NodeID: std::hash::Hash,
{
    pub fn new(tree: &T, skip: impl IntoIterator<Item = T::NodeID>, max_depth: usize) -> Self {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let path_root = tree.root_path_node();
        queue.push_back((path_root, 0));
        visited.insert(tree.target(path_root));

        Self {
            queue,
            skip: skip.into_iter().collect(),
            visited,
            max_depth,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next(&mut self, tree: &T) -> Option<T::PathNode> {
        let (path_node, depth) = self.queue.pop_front()?;
        if depth + 1 > self.max_depth {
            return Some(path_node);
        }

        for child_path_node in tree.get_children_path_nodes(tree.target(path_node)) {
            let child_id = tree.target(child_path_node);
            if !self.skip.contains(&child_id) && self.visited.insert(child_id) {
                self.queue.push_back((child_path_node, depth + 1));
            }
        }

        Some(path_node)
    }

    pub fn into_iter(self, tree: &T) -> BfsIterator<'_, G, H, A, T> {
        BfsIterator { walker: self, tree }
    }
}

pub struct BfsIterator<'a, G, H, A, T>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
    T::NodeID: std::hash::Hash,
{
    walker: BfsWalker<G, H, A, T>,
    tree: &'a T,
}

impl<'a, G, H, A, T> Iterator for BfsIterator<'a, G, H, A, T>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
    T::NodeID: std::hash::Hash,
{
    type Item = T::PathNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.tree)
    }
}
