// depth first search of MCTS tree

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree};
use std::collections::HashSet;

// iterative DFS over NodeIDs, Tree is extern.
pub struct DfsWalker<G, H, A, T>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
    T::NodeID: std::hash::Hash,
{
    stack: Vec<T::NodeID>,
    skip: HashSet<T::NodeID>,
    visited: HashSet<T::NodeID>, // required because of transposition table
    _phantom: std::marker::PhantomData<(G, H, A, T)>,
}

impl<G, H, A, T> DfsWalker<G, H, A, T>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
    T::NodeID: std::hash::Hash,
{
    pub fn new(start: T::NodeID, skip: impl IntoIterator<Item = T::NodeID>) -> Self {
        let stack = vec![start];
        Self {
            stack,
            skip: skip.into_iter().collect(),
            visited: HashSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next(&mut self, tree: &T) -> Option<T::NodeID> {
        let node_id = self.stack.pop()?;
        let children_ids: Vec<_> = tree
            .get_children_steps(node_id)
            .map(|s| tree.target(s))
            .collect();
        for child_id in children_ids.into_iter().rev() {
            if !self.skip.contains(&child_id) && self.visited.insert(child_id) {
                self.stack.push(child_id);
            }
        }
        Some(node_id)
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn into_iter(self, tree: &T) -> DfsIterator<'_, G, H, A, T> {
        DfsIterator { walker: self, tree }
    }
}

pub struct DfsIterator<'a, G, H, A, T>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
    T::NodeID: std::hash::Hash,
{
    walker: DfsWalker<G, H, A, T>,
    tree: &'a T,
}

impl<G, H, A, T> Iterator for DfsIterator<'_, G, H, A, T>
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
