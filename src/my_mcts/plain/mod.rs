// plain implementation of mcts traits

mod node;

pub use node::*;

use super::{BaseTree, BaseMCTS, NoTranspositionTable, TranspositionHashMap};

// plain implementation of MCTSTree using PlainNode
pub type PlainTree<G, UP, UC, EP, H> = BaseTree<G, PlainNode<G, UP, UC, EP, H>, EP, H>;

// PlainMCTS: BaseMCTS + PlainNode + PlainTree
pub type PlainMCTS<G, UP, UC, EP, H, SP> = BaseMCTS<
    G,
    PlainNode<G, UP, UC, EP, H>,
    PlainTree<G, UP, UC, EP, H>,
    UP,
    UC,
    EP,
    H,
    SP,
    NoTranspositionTable,
>;

// PlainMCTSWithTT: BaseMCTS + PlainNode + PlainTree + TranspositionHashMap
pub type PlainMCTSWithTT<G, UP, UC, EP, H, SP> = BaseMCTS<
    G,
    PlainNode<G, UP, UC, EP, H>,
    PlainTree<G, UP, UC, EP, H>,
    UP,
    UC,
    EP,
    H,
    SP,
    TranspositionHashMap<G, PlainNode<G, UP, UC, EP, H>, PlainTree<G, UP, UC, EP, H>, EP, H>,
>;
