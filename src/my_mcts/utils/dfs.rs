// depth first search of MCTS tree

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree};
use std::collections::VecDeque;

// iterative DFS over NodeIDs, Tree is extern.
pub struct DfsWalker<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    stack: VecDeque<A::NodeID>,
    skip: Vec<A::NodeID>,
    _phantom: std::marker::PhantomData<(G, H, A)>,
}

impl<G, H, A> DfsWalker<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    pub fn new(start: A::NodeID, skip: Vec<A::NodeID>) -> Self {
        let mut stack = VecDeque::new();
        stack.push_back(start);
        Self {
            stack,
            skip,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next(&mut self, tree: &A::Tree) -> Option<A::NodeID> {
        let id = self.stack.pop_back()?;
        for (child, _) in tree.get_children(id).iter() {
            if !self.skip.contains(child) {
                self.stack.push_back(*child);
            }
        }
        Some(id)
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
{
    walker: DfsWalker<G, H, A>,
    tree: &'a A::Tree,
}

impl<G, H, A> Iterator for DfsIterator<'_, G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Item = A::NodeID;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.tree)
    }
}
