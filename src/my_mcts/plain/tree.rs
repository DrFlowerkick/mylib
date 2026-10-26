use crate::my_mcts::UCTCache;

// plain implementation of MCTSTree
use super::{
    super::utils::BfsWalker, BaseEdge, BaseNode, ExpansionPolicy, Heuristic, MCTSAlgo, MCTSEdge,
    MCTSGame, MCTSNode, MCTSTree, TranspositionHashMap, TranspositionTable, UCTStatsPolicy,
};

// plain type of TranspositionHashMap, which works with PlainTree
pub type PlainTTHashMap<State> = TranspositionHashMap<State, usize>;

type Edges<G, H, A, UC> = Vec<Vec<(usize, BaseEdge<G, H, A, UC>)>>;

#[derive(Clone)]
pub struct PlainTree<G, H, A, USP, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    USP: UCTStatsPolicy<G, H, A>,
    TT: TranspositionTable<G::State, usize>,
{
    pub nodes: Vec<BaseNode<G, H, A, USP::Node>>,
    pub edges: Edges<G, H, A, USP::Edge>,
    pub transposition_table: TT,
    pub root_id: usize,
}

impl<G, H, A, USP, TT> PlainTree<G, H, A, USP, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    USP: UCTStatsPolicy<G, H, A>,
    TT: TranspositionTable<G::State, usize>,
{
    pub fn new(expected_num_nodes: usize) -> Self {
        PlainTree {
            nodes: Vec::with_capacity(expected_num_nodes),
            edges: Vec::with_capacity(expected_num_nodes),
            transposition_table: TranspositionTable::new(expected_num_nodes),
            root_id: 0,
        }
    }

    fn get_parent_visits(&self, id: usize) -> usize {
        // 1.) check if node returns visits (Option<usize>)
        // 2.) if none, nodes do not contain visits -> edge based, but no incoming edge available
        // 3.) iter edges and return sum visits of each edge
        if let Some(visits) = self.nodes[id].get_visits() {
            visits
        } else {
            self.edges[id]
                .iter()
                .filter_map(|(_, e)| e.get_visits())
                .sum()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlainPathNode {
    Root(usize),
    Via { parent_id: usize, edge_idx: usize },
}

impl<G, H, A, USP, TT> MCTSTree<G, H, A> for PlainTree<G, H, A, USP, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    USP: UCTStatsPolicy<G, H, A>,
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

    fn get_target_visits(&self, path_node: Self::PathNode) -> usize {
        // 1.) check if path_node points to root or some node
        // 2.) root: call fn get_parent_visits(root_id)
        // 3.) some node: check if node returns visits (Option<usize>)
        // 4.) if none, nodes do not contain visits -> check edge visits
        // 5.) edge returns visits (Option<usize>) -> expect Some(usize)
        match path_node {
            PlainPathNode::Root(root_id) => self.get_parent_visits(root_id),
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => {
                if let Some(visits) = self.nodes[self.target(path_node)].get_visits() {
                    visits
                } else {
                    self.edges[parent_id][edge_idx]
                        .1
                        .get_visits()
                        .expect("node or edge must contain visits.")
                }
            }
        }
    }

    fn get_visits_from_parent(&self, path_node: Self::PathNode) -> usize {
        // 1.) return 0, if path_node points to root, since root has no parent
        // 2.) check if edge uct_cache returns visits (Option<usize>)
        // 3.) if None, target node uct_cache returns visits (Option<usize>) -> expect Some(usize)
        match path_node {
            PlainPathNode::Root(_) => 0,
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => {
                if let Some(visits) = self.edges[parent_id][edge_idx].1.get_visits() {
                    visits
                } else {
                    self.nodes[self.edges[parent_id][edge_idx].0]
                        .get_visits()
                        .expect("node or edge must contain visits.")
                }
            }
        }
    }

    fn get_win_probability(&self) -> f32 {
        if self.nodes[self.root_id].get_accumulated_value().is_some() {
            // node contains exploitation stats
            self.nodes[self.root_id].calc_exploitation_score()
        } else {
            // edges only -> total uct is some of children edges
            // last_player for exploitation score is current player of root
            let last_player = G::current_player(self.nodes[self.root_id].get_state());
            self.edges[self.root_id]
                .iter()
                .map(|(_, e)| e.uct_cache.calc_exploitation_score(last_player))
                .sum()
        }
    }

    fn calc_uct(&mut self, path_node: Self::PathNode, mcts_config: &A::Config) -> f32 {
        match path_node {
            PlainPathNode::Root(_) => {
                // for root we only calc exploitation score, which is equal to the expected
                // win probability of the current tree
                self.get_win_probability()
            }
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => {
                let parent_visits = self.get_parent_visits(parent_id);
                let child_id = self.target(path_node);
                let node_uct = self.nodes[child_id].calc_exploitation_score()
                    + self.nodes[child_id].calc_exploration_score(parent_visits, mcts_config);
                let last_player = G::last_player(self.nodes[child_id].get_state());
                let edge_uct = self.edges[parent_id][edge_idx]
                    .1
                    .calc_exploitation_score(last_player)
                    + self.edges[parent_id][edge_idx].1.calc_exploration_score(
                        last_player,
                        parent_visits,
                        mcts_config,
                    );
                node_uct + edge_uct
            }
        }
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

    fn update_stats(&mut self, path_node: Self::PathNode, result: f32) {
        match path_node {
            PlainPathNode::Root(root_id) => self.nodes[root_id].update_stats(result),
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => {
                // update states of node
                let child_id = self.edges[parent_id][edge_idx].0;
                self.nodes[child_id].update_stats(result);
                // update stats of edge
                let state = self.nodes[child_id].get_state();
                self.edges[parent_id][edge_idx]
                    .1
                    .update_stats(state, result);
            }
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
