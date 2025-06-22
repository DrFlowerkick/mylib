// breadth first search

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree};
use std::collections::HashSet;
use std::collections::VecDeque;

pub struct BfsWalker<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    A::NodeID: std::hash::Hash,
{
    queue: VecDeque<A::NodeID>,
    skip: HashSet<A::NodeID>,
    visited: HashSet<A::NodeID>, // required because of transposition table
    _phantom: std::marker::PhantomData<(G, H, A)>,
}

impl<G, H, A> BfsWalker<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    A::NodeID: std::hash::Hash,
{
    pub fn new(start: A::NodeID, skip: impl IntoIterator<Item = A::NodeID>) -> Self {
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

    pub fn next(&mut self, tree: &A::Tree) -> Option<A::NodeID> {
        let node_id = self.queue.pop_front()?;

        for (child_id, _) in tree.get_children(node_id) {
            if !self.skip.contains(child_id) && self.visited.insert(*child_id) {
                self.queue.push_back(*child_id);
            }
        }

        Some(node_id)
    }

    pub fn into_iter(self, tree: &A::Tree) -> BfsIterator<G, H, A> {
        BfsIterator { walker: self, tree }
    }
}

pub struct BfsIterator<'a, G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    A::NodeID: std::hash::Hash,
{
    walker: BfsWalker<G, H, A>,
    tree: &'a A::Tree,
}

impl<'a, G, H, A> Iterator for BfsIterator<'a, G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    A::NodeID: std::hash::Hash,
{
    type Item = A::NodeID;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.tree)
    }
}
