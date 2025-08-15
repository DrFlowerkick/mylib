// plain implementation of mcts traits
// - a plain implementation of MCTSNode
// - a base implementation of MCTSTree, using Vec to store nodes of type MCTSNode
//   and edges. An edge contains the ID of a child of a node and the corresponding
//   move, which creates this child.
// - a base implementation of MCTSAlgo, using all defined traits of MCTS

mod algo;
mod node;
mod tree;

pub use algo::*;
pub use node::*;
pub use tree::*;

use super::{
    ExpansionPolicy, GameCache, Heuristic, HeuristicCache, MCTSAlgo, MCTSConfig, MCTSGame,
    MCTSNode, MCTSTree, SimulationPolicy, TranspositionHashMap, TranspositionTable, UCTPolicy,
    UTCCache,
};
