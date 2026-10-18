// plain implementation of MCTSTree

use crate::my_mcts::{ExpansionPolicy, MCTSNode};

use super::{
    super::utils::BfsWalker, Heuristic, MCTSAlgo, MCTSGame, MCTSTree, PlainNode,
    TranspositionHashMap, TranspositionTable, UTCCache,
};

// plain type of TranspositionHashMap, which works with PlainTree
pub type PlainTTHashMap<State> = TranspositionHashMap<State, usize>;

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

impl<G, H, A, UC, TT> PlainTree<G, H, A, UC, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
    TT: TranspositionTable<G::State, usize>,
{
    pub fn new(expected_num_nodes: usize) -> Self {
        PlainTree {
            nodes: Vec::with_capacity(expected_num_nodes),
            edges: Vec::with_capacity(expected_num_nodes),
            transposition_table: TranspositionTable::new(expected_num_nodes),
            root_id: 0,
            phantom: std::marker::PhantomData,
        }
    }
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

    fn init_root(&mut self, state: G::State) -> Self::NodeID {
        // Clear any existing nodes and edges
        self.root_id = 0;
        self.transposition_table.clear();
        self.transposition_table.insert(state.clone(), self.root_id);
        self.nodes.clear();
        self.edges.clear();
        let root_node = PlainNode::new(state);
        self.nodes.push(root_node);
        self.edges.push(vec![]);
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

    fn get_node_id(
        &self,
        state: &<G as MCTSGame>::State,
        start_node_id: Self::NodeID,
        max_depth: usize,
    ) -> Option<Self::NodeID> {
        // search for node with state in cache (transposition table)
        if let Some(node_of_state_id) = self.get_cached_node_id(state) {
            return Some(node_of_state_id);
        }

        // if not found, try it with bfs from start_node_id
        if self.nodes.len() <= start_node_id {
            // start_node_id is not in the tree
            return None;
        }

        // breadth-first search (BFS) for node with given state
        let bfs = BfsWalker::new(start_node_id, None, max_depth);
        bfs.into_iter(self)
            .find(|id| self.get_node_state(*id) == state)
    }

    fn add_child(&mut self, parent_id: Self::NodeID, mv: G::Move, state: G::State) -> usize {
        let child_id = self.nodes.len();
        self.transposition_table.insert(state.clone(), child_id);
        let child = PlainNode::new(state);
        self.nodes.push(child);
        self.edges.push(vec![]);
        self.link_child(parent_id, mv, child_id);
        child_id
    }

    fn get_cached_node_id(&self, state: &<G as MCTSGame>::State) -> Option<Self::NodeID> {
        self.transposition_table.get(state).copied()
    }

    fn link_child(&mut self, parent_id: Self::NodeID, mv: G::Move, child_id: Self::NodeID) {
        self.edges
            .get_mut(parent_id)
            .expect("Expected edges of parent.")
            .push((child_id, mv));
    }

    // ToDo: this must be reworked later, when we add different ways of handling data in tree (NodeOnly, EdgeOnly, Hybrid).
    fn get_node_visits(&self, id: Self::NodeID) -> usize {
        self.nodes[id].visits
    }

    fn get_node_state(&self, id: Self::NodeID) -> &<G as MCTSGame>::State {
        &self.nodes[id].state
    }

    fn should_node_expand(
        &self,
        id: Self::NodeID,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> bool {
        let visits = self.get_node_visits(id);
        let num_children = self.count_children(id);
        self.nodes[id].expansion_policy.should_expand(
            visits,
            num_children,
            mcts_config,
            heuristic_config,
        )
    }

    fn pop_node_expandable_move(
        &mut self,
        id: Self::NodeID,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move> {
        let visits = self.get_node_visits(id);
        let num_children = self.count_children(id);
        let node = &mut self.nodes[id];
        let (state, ep) = (&node.state, &mut node.expansion_policy);
        ep.pop_expandable_move(state, visits, num_children, mcts_config, heuristic_config)
    }

    fn count_children(&self, id: Self::NodeID) -> usize {
        self.edges[id].len()
    }

    fn get_children_ids<'a>(
        &'a self,
        id: Self::NodeID,
    ) -> Box<dyn Iterator<Item = Self::NodeID> + 'a> {
        Box::new(self.edges[id].iter().map(|(id, _)| *id))
    }

    fn get_move(&self, parent_id: Self::NodeID, id: Self::NodeID) -> &<G as MCTSGame>::Move {
        self.edges[parent_id]
            .iter()
            .find(|(child_id, _)| *child_id == id)
            .map(|(_, mv)| mv)
            .expect("expected valid child id")
    }

    fn get_node_mut(&mut self, id: Self::NodeID) -> &mut Self::Node {
        &mut self.nodes[id]
    }
}
