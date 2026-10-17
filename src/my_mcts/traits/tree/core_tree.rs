// tree trait of mcts

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSNode, TranspositionTable};

pub trait MCTSTree<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node: MCTSNode<G, H, A::Config, A::UTC, A::Expansion>;
    type NodeID: Copy + Eq + std::fmt::Debug;
    type TranspositionTable: TranspositionTable<G::State, Self::NodeID>;

    fn init_root(&mut self, state: G::State, expansion_policy: A::Expansion) -> Self::NodeID;
    fn set_root(&mut self, new_root_id: Self::NodeID);
    fn root_id(&self) -> Option<Self::NodeID>;
    fn get_node_id(
        &self,
        state: &G::State,
        start_node_id: Self::NodeID,
        max_depth: usize,
    ) -> Option<Self::NodeID>;
    fn get_node(&self, id: Self::NodeID) -> &Self::Node;
    fn get_node_mut(&mut self, id: Self::NodeID) -> &mut Self::Node;
    fn add_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        state: G::State,
        expansion_policy: A::Expansion,
    ) -> Self::NodeID;
    fn get_cached_node_id(&self, state: &G::State) -> Option<Self::NodeID>;
    fn link_child(&mut self, parent_id: Self::NodeID, mv: G::Move, child_id: Self::NodeID);
    fn get_children(&self, id: Self::NodeID) -> &[(Self::NodeID, G::Move)];
}
