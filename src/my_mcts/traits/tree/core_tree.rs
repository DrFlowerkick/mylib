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
    type Step: Copy + Eq + std::fmt::Debug;
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
    fn add_child(&mut self, parent_id: Self::NodeID, mv: G::Move, state: G::State) -> Self::Step;
    fn get_cached_node_id(&self, state: &G::State) -> Option<Self::NodeID>;
    fn link_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        child_id: Self::NodeID,
    ) -> Self::Step;
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
    fn get_children_steps<'a>(
        &'a self,
        parent_id: Self::NodeID,
    ) -> Box<dyn Iterator<Item = Self::Step> + 'a>;
    fn calc_utc(&mut self, step: Self::Step, mcts_config: &A::Config) -> f32;
    fn target(&self, step: Self::Step) -> Self::NodeID;
    fn get_move(&self, step: Self::Step) -> &G::Move;
    fn update_stats(&mut self, step: Self::Step, result: f32);
}
