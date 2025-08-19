// depth first search of MCTS tree

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree};
use std::collections::HashSet;

// iterative DFS over NodeIDs, Tree is extern.
pub struct DfsWalker<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    A::NodeID: std::hash::Hash,
{
    stack: Vec<A::NodeID>,
    skip: HashSet<A::NodeID>,
    visited: HashSet<A::NodeID>, // required because of transposition table
    _phantom: std::marker::PhantomData<(G, H, A)>,
}

impl<G, H, A> DfsWalker<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    A::NodeID: std::hash::Hash,
{
    pub fn new(start: A::NodeID, skip: impl IntoIterator<Item = A::NodeID>) -> Self {
        let stack = vec![start];
        Self {
            stack,
            skip: skip.into_iter().collect(),
            visited: HashSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next(&mut self, tree: &A::Tree) -> Option<A::NodeID> {
        let node_id = self.stack.pop()?;
        for (child_id, _) in tree.get_children(node_id).iter().rev() {
            if !self.skip.contains(child_id) && self.visited.insert(*child_id) {
                self.stack.push(*child_id);
            }
        }
        Some(node_id)
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn into_iter(self, tree: &A::Tree) -> DfsIterator<'_, G, H, A> {
        DfsIterator { walker: self, tree }
    }
}

pub struct DfsIterator<'a, G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    A::NodeID: std::hash::Hash,
{
    walker: DfsWalker<G, H, A>,
    tree: &'a A::Tree,
}

impl<G, H, A> Iterator for DfsIterator<'_, G, H, A>
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
