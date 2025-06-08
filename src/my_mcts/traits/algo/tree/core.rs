// tree trait of mcts

use super::{MCTSGame, Heuristic, MCTSAlgo, MCTSNode};

pub trait MCTSTree<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node: MCTSNode<G, H, A::Config, A::UTC, A::Expansion>;

    fn new() -> Self;
    fn init_root(&mut self, root_value: Self::Node) -> A::NodeID;
    fn set_root(&mut self, new_root_id: A::NodeID);
    fn root_id(&self) -> Option<A::NodeID>;
    fn get_node(&self, id: A::NodeID) -> &Self::Node;
    fn get_node_mut(&mut self, id: A::NodeID) -> &mut Self::Node;
    fn add_child(
        &mut self,
        parent_id: A::NodeID,
        mv: G::Move,
        child_value: Self::Node,
    ) -> A::NodeID;
    fn link_child(&mut self, parent_id: A::NodeID, mv: G::Move, child_id: A::NodeID);
    fn get_children(&self, id: A::NodeID) -> &[(A::NodeID, G::Move)];
}
