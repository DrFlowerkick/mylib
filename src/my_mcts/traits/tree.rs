// tree trait of mcts

use super::*;

pub trait MCTSTree<G, N, EP, H>
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    type ID: Copy + Eq + std::fmt::Debug;

    fn new() -> Self;
    fn init_root(&mut self, root_value: N) -> Self::ID;
    fn set_root(&mut self, new_root_id: Self::ID);
    fn root_id(&self) -> Option<Self::ID>;
    fn get_node(&self, id: Self::ID) -> &N;
    fn get_node_mut(&mut self, id: Self::ID) -> &mut N;
    fn add_child(&mut self, parent_id: Self::ID, mv: G::Move, child_value: N) -> Self::ID;
    fn link_child(&mut self, parent_id: Self::ID, mv: G::Move, child_id: Self::ID);
    fn get_children(&self, id: Self::ID) -> &[(Self::ID, G::Move)];
}
