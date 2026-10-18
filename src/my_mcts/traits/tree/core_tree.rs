// tree trait of mcts

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSNode, TranspositionTable};

// ToDo: This API is kinda "unsafe". We expect proper usage, but it is easily broken, e.g.:
// - call set_root() with a new_root_id, which does not exist in tree.
// - call add_child() with a parent_id, which does not exist in tree.
// These error cases will result in panics. What about error handling to prevent panics?
pub trait MCTSTree<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node: MCTSNode<G, H, A::Config, A::UTC, A::Expansion>;
    type NodeID: Copy + Eq + std::fmt::Debug;
    type TranspositionTable: TranspositionTable<G::State, Self::NodeID>;

    fn init_root(&mut self, state: G::State) -> Self::NodeID;
    fn set_root(&mut self, new_root_id: Self::NodeID);
    fn root_id(&self) -> Option<Self::NodeID>;
    fn get_node_id(
        &self,
        state: &G::State,
        start_node_id: Self::NodeID,
        max_depth: usize,
    ) -> Option<Self::NodeID>;
    fn add_child(&mut self, parent_id: Self::NodeID, mv: G::Move, state: G::State) -> Self::NodeID;
    fn get_cached_node_id(&self, state: &G::State) -> Option<Self::NodeID>;
    fn link_child(&mut self, parent_id: Self::NodeID, mv: G::Move, child_id: Self::NodeID);
    fn get_node_visits(&self, id: Self::NodeID) -> usize;
    fn get_node_state(&self, id: Self::NodeID) -> &G::State;
    fn should_node_expand(
        &self,
        id: Self::NodeID,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> bool;
    fn pop_node_expandable_move(
        &mut self,
        id: Self::NodeID,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move>;
    fn count_children(&self, id: Self::NodeID) -> usize;
    fn get_children_ids<'a>(
        &'a self,
        id: Self::NodeID,
    ) -> Box<dyn Iterator<Item = Self::NodeID> + 'a>;
    fn get_move(&self, parent_id: Self::NodeID, id: Self::NodeID) -> &G::Move;

    // ToDo: remove direct access to a node or edge. Replace these with any interaction fn, which
    // is required for MCTS
    fn get_node_mut(&mut self, id: Self::NodeID) -> &mut Self::Node;
}
