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
    stack: Vec<T::PathNode>,
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
    pub fn new(tree: &T, skip: impl IntoIterator<Item = T::NodeID>) -> Self {
        let stack = vec![tree.root_path_node()];
        Self {
            stack,
            skip: skip.into_iter().collect(),
            visited: HashSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next(&mut self, tree: &T) -> Option<T::NodeID> {
        let path_node: <T as MCTSTree<G, H, A>>::PathNode = self.stack.pop()?;
        let node_id = tree.target(path_node);
        let children_path_nodes: Vec<_> = tree.get_children_path_nodes(node_id).collect();
        for child_path_node in children_path_nodes.into_iter().rev() {
            let child_id = tree.target(path_node);
            if !self.skip.contains(&child_id) && self.visited.insert(child_id) {
                self.stack.push(child_path_node);
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
