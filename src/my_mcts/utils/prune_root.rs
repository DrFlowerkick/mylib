// if root of tree is set to new root, this util removes all nodes before new root

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree, PlainTree, TranspositionTable, UCTCache};

pub trait PruneToRoot {
    fn prune_to_root(&mut self);
}

// pruning to root is only allowed for PlainTree, if no transposition table is used,
// since the table would contain old (and therefore wrong) indices after pruning.
impl<G, H, A, TT, UC> PruneToRoot for PlainTree<G, H, A, UC, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    TT: TranspositionTable<G::State, usize>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    fn prune_to_root(&mut self) {
        let Some(root_id) = self.root_id() else {
            // uninitialized tree
            return;
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
