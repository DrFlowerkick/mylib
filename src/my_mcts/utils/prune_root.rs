// if root of tree is set to new root, this util removes all nodes before new root

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree, NoTranspositionTable, PlainTree, UTCCache};

pub trait PruneToRoot<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H, Tree = Self>,
{
    fn prune_to_root(&mut self);
}

// pruning to root is only allowed for PlainTree, if no transposition table is used,
// since the table would contain old (and therefore wrong) indices after pruning.
impl<G, H, A, UC> PruneToRoot<G, H, A> for PlainTree<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H, Tree = Self, NodeID = usize, TranspositionTable = NoTranspositionTable>,
    UC: UTCCache<G, A::UTC, A::Config>,
{
    fn prune_to_root(&mut self) {
        let Some(root_id) = self.root_id() else {
            // uninitialized tree
            return
        };
        if root_id == 0 {
            // nothing to prune
            return;
        }

        // remove all nodes and edges before root_id
        self.nodes.drain(..root_id);
        self.edges.drain(..root_id);

        // reindex all remaining children
        for edge in self.edges.iter_mut() {
            for (child_id, _move) in edge.iter_mut() {
                *child_id -= root_id;
            }
        }

        // reset root id
        self.root_id = 0;
    }
}
