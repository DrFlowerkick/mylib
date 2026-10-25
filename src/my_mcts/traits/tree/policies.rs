// policy traits at MCTS tree layer

use super::{Heuristic, MCTSAlgo, MCTSGame, UCTCache};

// A tree consist of nodes and edges. Both may contain parameters required
// to calculate UCT respectively exploitation and exploration scores.
// These parameters may even split between nodes and edges.
// There are two requirements to consider:
// 1.) You always want to calculate the exploitation score based upon all
//     visits o a node.
// 2.) The exploration score expects the fulfillment of N(parent) ≈ Σ N(children).
// Without caching of nodes in a transposition table both requirements may be
// fulfilled with either node-only or edge-only handling of all UCT parameters.
// These are:
// - N: visits of the node
// - acc: accumulated value of simulations, of which the node was part of the
// path of moves, which led to the game state, which was simulated.
// The exploration score requires the visits of the parent node (see above
// N(parent)).
//
// Some games profit from caching of game states with transposition table. If
// multiple path of moves result at the same number of turns in the same
// game state, the expansion of the tree may continue for all the path from the
// same node, which holds the resulting game state.
// In other words, you may reach a node of the tree via different path. This
// has consequences for UCT:
// In case of node-only, the formula N(parent) ≈ Σ N(children) cannot be
// fulfilled anymore, since a child may be reached by other path, increasing
// it visits. This does not happen with edge-only, but with edge-only you
// only see the visits of a node from each respective path (since the
// visits are stored in the edge) and not the sum of all visits, leading
// to a miscalculation of the exploitation score.
//
// You can solve this with a hybrid strategy: save visits in node and edge.
// For exploitation score you use visits from the node (and acc). For
// exploration you use for parent visits the visits of the parent node
// and for child visits the visits of the edge, leading from parent
// to child.
//
// There are several ways to implement this in rust. In this crate we want
// maximum flexibility combined with all options to minimize memory usage.
// Instead of implementing three times the MCTSTree trait we provide the
// UCTStatsPolicy trait. With this trait you define the combination of
// acceptable UCTCache implementations for node and edge. In implementation
// of MCTSTree you use for the UCTCache of a node and edge the types
// from this trait instead of defining them separately.

pub trait UCTStatsPolicy<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node: UCTCache<G, A::UCT, A::Config>;
    type Edge: UCTCache<G, A::UCT, A::Config>;
}
