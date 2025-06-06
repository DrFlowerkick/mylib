// plain implementation of MCTSTree

use super::{ExpansionPolicy, Heuristic, MCTSGame, MCTSNode, MCTSTree, PlainNode};

// plain implementation of MCTSTree using PlainNode
pub type PlainTree<G, UP, UC, EP, H> = BaseTree<G, PlainNode<G, UP, UC, EP, H>, EP, H>;

// base implementation of MCTSTree
pub struct BaseTree<G, N, EP, H>
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    pub nodes: Vec<N>,
    pub edges: Vec<Vec<(usize, G::Move)>>,
    pub root_id: usize,

    phantom: std::marker::PhantomData<(G, N, EP, H)>,
}

impl<G, N, EP, H> MCTSTree<G, N, EP, H> for BaseTree<G, N, EP, H>
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    type ID = usize;

    fn new() -> Self {
        BaseTree {
            nodes: vec![],
            edges: vec![],
            root_id: 0,
            phantom: std::marker::PhantomData,
        }
    }

    fn init_root(&mut self, root_value: N) -> Self::ID {
        // Clear any existing nodes and edges
        self.nodes.clear();
        self.edges.clear();
        self.nodes.push(root_value);
        self.edges.push(vec![]);
        self.root_id = 0;
        self.root_id
    }

    fn set_root(&mut self, new_root_id: Self::ID) {
        self.root_id = new_root_id;
    }

    fn root_id(&self) -> Option<Self::ID> {
        if self.nodes.is_empty() {
            // No nodes in the tree
            None
        } else {
            Some(self.root_id)
        }
    }

    fn get_node(&self, id: Self::ID) -> &N {
        &self.nodes[id]
    }

    fn get_node_mut(&mut self, id: Self::ID) -> &mut N {
        &mut self.nodes[id]
    }

    fn add_child(&mut self, parent_id: Self::ID, mv: G::Move, child_value: N) -> usize {
        let child_id = self.nodes.len();
        self.nodes.push(child_value);
        self.edges.push(vec![]);
        self.link_child(parent_id, mv, child_id);
        child_id
    }

    fn link_child(&mut self, parent_id: Self::ID, mv: <G as MCTSGame>::Move, child_id: Self::ID) {
        let edge = self
            .edges
            .get_mut(parent_id)
            .expect("Expected edges of parent.");
        edge.push((child_id, mv));
    }

    fn get_children(&self, id: Self::ID) -> &[(Self::ID, <G as MCTSGame>::Move)] {
        &self.edges[id][..]
    }
}
