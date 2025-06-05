// type definition and functions of mcts tree

use super::{
    ExpansionPolicy, Heuristic, MCTSGame, MCTSNode, MCTSTree, PlainNode, UCTPolicy, UTCCache,
};
use anyhow::Context;

pub struct PlainTree<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    pub nodes: Vec<PlainNode<G, UP, UC, EP, H>>,
    pub root_id: usize,

    phantom: std::marker::PhantomData<(G, EP, H)>,
}

impl<G, UP, UC, EP, H> MCTSTree<G, PlainNode<G, UP, UC, EP, H>, EP, H>
    for PlainTree<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self {
        let root_id = PlainNode::<G, UP, UC, EP, H>::init_root_id();
        let nodes = vec![];
        PlainTree {
            nodes,
            root_id,
            phantom: std::marker::PhantomData,
        }
    }

    fn init_root(&mut self, root_value: PlainNode<G, UP, UC, EP, H>) {
        self.nodes.clear(); // Clear any existing nodes
        self.nodes.push(root_value);
        self.root_id = PlainNode::<G, UP, UC, EP, H>::init_root_id();
    }

    fn set_root(&mut self, new_root_id: usize) {
        self.root_id = new_root_id;
    }

    fn root_id(&self) -> Option<usize> {
        if self.nodes.is_empty() {
            None // No nodes in the tree
        } else {
            Some(self.root_id)
        }
    }

    fn get_node(&self, id: usize) -> anyhow::Result<&PlainNode<G, UP, UC, EP, H>> {
        self.nodes
            .get(id)
            .context(format!("Node with given ID {:?} does not exist", id))
    }

    fn get_node_mut(&mut self, id: usize) -> anyhow::Result<&mut PlainNode<G, UP, UC, EP, H>> {
        self.nodes
            .get_mut(id)
            .context(format!("Node with given ID {:?} does not exist", id))
    }

    fn add_child(
        &mut self,
        parent_id: usize,
        child_value: PlainNode<G, UP, UC, EP, H>,
    ) -> anyhow::Result<usize> {
        let child_id = self.nodes.len();
        self.get_node_mut(parent_id)?.add_child(child_id);
        self.nodes.push(child_value);
        Ok(child_id)
    }
}
