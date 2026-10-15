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
    queue: VecDeque<T::NodeID>,
    skip: HashSet<T::NodeID>,
    visited: HashSet<T::NodeID>, // required because of transposition table
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
    pub fn new(start: T::NodeID, skip: impl IntoIterator<Item = T::NodeID>) -> Self {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(start);
        visited.insert(start);

        Self {
            queue,
            skip: skip.into_iter().collect(),
            visited,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next(&mut self, tree: &T) -> Option<T::NodeID> {
        let node_id = self.queue.pop_front()?;

        for (child_id, _) in tree.get_children(node_id) {
            if !self.skip.contains(child_id) && self.visited.insert(*child_id) {
                self.queue.push_back(*child_id);
            }
        }

        Some(node_id)
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
    type Item = T::NodeID;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.tree)
    }
}
