// plain implementation of MCTSTree

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSTree, PlainNode, TranspositionTable, UTCCache};

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
#[derive(Clone)]
pub struct PlainTree<G, H, A, UC, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
    TT: TranspositionTable<G::State, usize>,
{
    pub nodes: Vec<Node<G, H, A, UC>>,
    pub edges: Vec<Vec<(usize, G::Move)>>,
    pub transposition_table: TT,
    pub root_id: usize,
    phantom: std::marker::PhantomData<(G, H, UC)>,
}

impl<G, H, A, UC, TT> MCTSTree<G, H, A> for PlainTree<G, H, A, UC, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
    TT: TranspositionTable<G::State, usize>,
{
    type Node = Node<G, H, A, UC>;
    type NodeID = usize;
    type TranspositionTable = TT;

    fn new(expected_num_nodes: usize) -> Self {
        PlainTree {
            nodes: Vec::with_capacity(expected_num_nodes),
            edges: Vec::with_capacity(expected_num_nodes),
            transposition_table: TranspositionTable::new(expected_num_nodes),
            root_id: 0,
            phantom: std::marker::PhantomData,
        }
    }

    fn init_root(&mut self, root_value: Self::Node) -> Self::NodeID {
        // Clear any existing nodes and edges
        self.nodes.clear();
        self.edges.clear();
        self.nodes.push(root_value);
        self.edges.push(vec![]);
        self.root_id = 0;
        self.root_id
    }

    fn set_root(&mut self, new_root_id: Self::NodeID) {
        self.root_id = new_root_id;
    }

    fn root_id(&self) -> Option<Self::NodeID> {
        if self.nodes.is_empty() {
            // No nodes in the tree
            None
        } else {
            Some(self.root_id)
        }
    }

    fn get_node(&self, id: Self::NodeID) -> &Self::Node {
        &self.nodes[id]
    }

    fn get_node_mut(&mut self, id: Self::NodeID) -> &mut Self::Node {
        &mut self.nodes[id]
    }

    fn add_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        child_value: Self::Node,
    ) -> usize {
        let child_id = self.nodes.len();
        self.nodes.push(child_value);
        self.edges.push(vec![]);
        self.link_child(parent_id, mv, child_id);
        child_id
    }

    fn link_child(&mut self, parent_id: Self::NodeID, mv: G::Move, child_id: Self::NodeID) {
        self.edges
            .get_mut(parent_id)
            .expect("Expected edges of parent.")
            .push((child_id, mv));
    }

    fn get_children(&self, id: Self::NodeID) -> &[(Self::NodeID, G::Move)] {
        &self.edges[id][..]
    }
}
