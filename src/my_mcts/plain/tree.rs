// plain implementation of MCTSTree

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree, PlainNode, UTCCache};

// use type definition to keep clippy happy :)
type Node<G, H, A, UC> = PlainNode<
    G,
    H,
    <A as MCTSAlgo<G, H>>::Config,
    UC,
    <A as MCTSAlgo<G, H>>::UTC,
    <A as MCTSAlgo<G, H>>::Expansion,
>;

// base implementation of MCTSTree
pub struct PlainTree<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
{
    pub nodes: Vec<Node<G, H, A, UC>>,
    pub edges: Vec<Vec<(usize, G::Move)>>,
    pub root_id: usize,
    phantom: std::marker::PhantomData<(G, H, UC)>,
}

impl<G, H, A, UC> MCTSTree<G, H, A> for PlainTree<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H, NodeID = usize>,
    UC: UTCCache<G, A::UTC, A::Config>,
{
    type Node = Node<G, H, A, UC>;

    fn new(expected_num_nodes: usize) -> Self {
        PlainTree {
            nodes: Vec::with_capacity(expected_num_nodes),
            edges: Vec::with_capacity(expected_num_nodes),
            root_id: 0,
            phantom: std::marker::PhantomData,
        }
    }

    fn init_root(&mut self, root_value: Self::Node) -> A::NodeID {
        // Clear any existing nodes and edges
        self.nodes.clear();
        self.edges.clear();
        self.nodes.push(root_value);
        self.edges.push(vec![]);
        self.root_id = 0;
        self.root_id
    }

    fn set_root(&mut self, new_root_id: A::NodeID) {
        self.root_id = new_root_id;
    }

    fn root_id(&self) -> Option<A::NodeID> {
        if self.nodes.is_empty() {
            // No nodes in the tree
            None
        } else {
            Some(self.root_id)
        }
    }

    fn get_node(&self, id: A::NodeID) -> &Self::Node {
        &self.nodes[id]
    }

    fn get_node_mut(&mut self, id: A::NodeID) -> &mut Self::Node {
        &mut self.nodes[id]
    }

    fn add_child(&mut self, parent_id: A::NodeID, mv: G::Move, child_value: Self::Node) -> usize {
        let child_id = self.nodes.len();
        self.nodes.push(child_value);
        self.edges.push(vec![]);
        self.link_child(parent_id, mv, child_id);
        child_id
    }

    fn link_child(&mut self, parent_id: A::NodeID, mv: G::Move, child_id: A::NodeID) {
        self.edges
            .get_mut(parent_id)
            .expect("Expected edges of parent.")
            .push((child_id, mv));
    }

    fn get_children(&self, id: A::NodeID) -> &[(A::NodeID, G::Move)] {
        &self.edges[id][..]
    }
}
