// tree trait of mcts

use super::{Heuristic, MCTSAlgo, MCTSGame};

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
    type NodeID: Copy + Eq + std::fmt::Debug;
    type PathNode: Copy + Eq + std::fmt::Debug;

    fn root_id(&self) -> Option<Self::NodeID>;
    fn init_root(&mut self, state: G::State) -> Self::NodeID;
    fn set_root(&mut self, new_root_id: Self::NodeID);
    fn get_node_id(
        &self,
        state: &G::State,
        start_node_id: Self::NodeID,
        max_depth: usize,
    ) -> Option<Self::NodeID>;
    fn get_cached_node_id(&self, state: &G::State) -> Option<Self::NodeID>;
    fn get_node_state(&self, id: Self::NodeID) -> &G::State;
    fn add_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        state: G::State,
    ) -> Self::PathNode;
    fn link_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        child_id: Self::NodeID,
    ) -> Self::PathNode;
    fn count_children(&self, id: Self::NodeID) -> usize;
    fn get_children_path_nodes<'a>(
        &'a self,
        parent_id: Self::NodeID,
    ) -> Box<dyn Iterator<Item = Self::PathNode> + 'a>;
    fn root_path_node(&self) -> Self::PathNode;
    fn target(&self, path_node: Self::PathNode) -> Self::NodeID;
    fn get_target_visits(&self, path_node: Self::PathNode) -> usize;
    fn get_visits_from_parent(&self, path_node: Self::PathNode) -> usize;
    fn calc_uct(&mut self, path_node: Self::PathNode, mcts_config: &A::Config) -> f32;
    fn should_target_expand(
        &self,
        path_node: Self::PathNode,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> bool;
    fn pop_target_expandable_move(
        &mut self,
        path_node: Self::PathNode,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move>;
    fn update_stats(&mut self, path_node: Self::PathNode, result: f32);
    fn get_move(&self, path_node: Self::PathNode) -> Option<&G::Move>;
}
