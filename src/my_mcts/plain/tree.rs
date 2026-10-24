// plain implementation of MCTSTree
use super::{
    super::utils::BfsWalker, BaseEdge, BaseNode, ExpansionPolicy, Heuristic, MCTSAlgo, MCTSEdge,
    MCTSGame, MCTSNode, MCTSTree, TranspositionHashMap, TranspositionTable, UTCCache,
};

// plain type of TranspositionHashMap, which works with PlainTree
pub type PlainTTHashMap<State> = TranspositionHashMap<State, usize>;

// ToDo: we will probably need separate UC for Node and Edge in the future
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
    pub nodes: Vec<BaseNode<G, H, A, UC>>,
    pub edges: Vec<Vec<(usize, BaseEdge<G, H, A, UC>)>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlainPathNode {
    Root(usize),
    Via { parent_id: usize, edge_idx: usize },
}

impl<G, H, A, UC, TT> MCTSTree<G, H, A> for PlainTree<G, H, A, UC, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
    TT: TranspositionTable<G::State, usize>,
{
    type NodeID = usize;
    type PathNode = PlainPathNode;

    fn root_id(&self) -> Option<Self::NodeID> {
        if self.nodes.is_empty() {
            // No nodes in the tree
            None
        } else {
            Some(self.root_id)
        }
    }

    fn init_root(&mut self, state: G::State) -> Self::NodeID {
        // Clear any existing nodes and edges
        self.root_id = 0;
        self.transposition_table.clear();
        self.transposition_table.insert(state.clone(), self.root_id);
        self.nodes.clear();
        self.edges.clear();
        let root_node = BaseNode::new(state);
        self.nodes.push(root_node);
        self.edges.push(vec![]);
        self.root_id
    }

    fn set_root(&mut self, new_root_id: Self::NodeID) {
        self.root_id = new_root_id;
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
        let bfs = BfsWalker::new(self, None, max_depth);
        bfs.into_iter(self)
            .map(|pn| self.target(pn))
            .find(|id| self.get_node_state(*id) == state)
    }

    fn get_cached_node_id(&self, state: &<G as MCTSGame>::State) -> Option<Self::NodeID> {
        self.transposition_table.get(state).copied()
    }

    fn get_node_state(&self, id: Self::NodeID) -> &<G as MCTSGame>::State {
        &self.nodes[id].state
    }

    fn add_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        state: G::State,
    ) -> Self::PathNode {
        let child_id = self.nodes.len();
        self.transposition_table.insert(state.clone(), child_id);
        let child = BaseNode::new(state);
        self.nodes.push(child);
        self.edges.push(vec![]);
        self.link_child(parent_id, mv, child_id)
    }

    fn link_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        child_id: Self::NodeID,
    ) -> Self::PathNode {
        let edge_idx = self.edges[parent_id].len();
        self.edges
            .get_mut(parent_id)
            .expect("Expected edges of parent.")
            .push((child_id, BaseEdge::new(mv)));
        PlainPathNode::Via {
            parent_id,
            edge_idx,
        }
    }

    fn count_children(&self, id: Self::NodeID) -> usize {
        self.edges[id].len()
    }

    fn get_children_path_nodes<'a>(
        &'a self,
        parent_id: Self::NodeID,
    ) -> Box<dyn Iterator<Item = Self::PathNode> + 'a> {
        Box::new(
            self.edges[parent_id]
                .iter()
                .enumerate()
                .map(move |(edge_idx, _)| PlainPathNode::Via {
                    parent_id,
                    edge_idx,
                }),
        )
    }

    fn root_path_node(&self) -> Self::PathNode {
        PlainPathNode::Root(self.root_id)
    }

    fn target(&self, path_node: Self::PathNode) -> Self::NodeID {
        match path_node {
            PlainPathNode::Root(root_id) => root_id,
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => self.edges[parent_id][edge_idx].0,
        }
    }

    // ToDo: this must be reworked later, when we add different ways of handling data in tree (NodeOnly, EdgeOnly, Hybrid).
    fn get_target_visits(&self, path_node: Self::PathNode) -> usize {
        self.nodes[self.target(path_node)].visits
    }

    // ToDo: this must be reworked later, when we add different ways of handling data in tree (NodeOnly, EdgeOnly, Hybrid).
    fn calc_utc(&mut self, path_node: Self::PathNode, mcts_config: &A::Config) -> f32 {
        let parent_visits = match path_node {
            PlainPathNode::Root(_) => 0,
            PlainPathNode::Via { parent_id, .. } => self.nodes[parent_id].visits,
        };
        let child_id = self.target(path_node);
        self.nodes[child_id].calc_utc(parent_visits, mcts_config)
    }

    fn should_target_expand(
        &self,
        path_node: Self::PathNode,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> bool {
        let visits = self.get_target_visits(path_node);
        let id = self.target(path_node);
        let num_children = self.count_children(id);
        self.nodes[id].expansion_policy.should_expand(
            visits,
            num_children,
            mcts_config,
            heuristic_config,
        )
    }

    fn pop_target_expandable_move(
        &mut self,
        path_node: Self::PathNode,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move> {
        let visits = self.get_target_visits(path_node);
        let id = self.target(path_node);
        let num_children = self.count_children(id);
        let node = &mut self.nodes[id];
        let (state, ep) = (&node.state, &mut node.expansion_policy);
        ep.pop_expandable_move(state, visits, num_children, mcts_config, heuristic_config)
    }

    // ToDo: this must be reworked later, when we add different ways of handling data in tree (NodeOnly, EdgeOnly, Hybrid).
    fn update_stats(&mut self, path_node: Self::PathNode, result: f32) {
        match path_node {
            PlainPathNode::Root(root_id) => self.nodes[root_id].update_stats(result),
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => self.nodes[self.edges[parent_id][edge_idx].0].update_stats(result),
        }
    }

    fn get_move(&self, path_node: Self::PathNode) -> Option<&G::Move> {
        match path_node {
            PlainPathNode::Root(_) => None,
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => Some(self.edges[parent_id][edge_idx].1.get_move()),
        }
    }
}
