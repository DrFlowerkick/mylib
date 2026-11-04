// plain implementation of MCTSTree
use super::{
    super::utils::BfsWalker, BaseEdge, BaseNode, Heuristic, MCTSAlgo, MCTSEdge, MCTSGame, MCTSNode,
    MCTSTree, PruneToRoot, TranspositionHashMap, TranspositionTable, UCTStatsPolicy,
};

// plain type of TranspositionHashMap, which works with PlainTree
pub type PlainTTHashMap<State> = TranspositionHashMap<State, usize>;

type Edges<G, H, A, UC> = Vec<Vec<(usize, BaseEdge<G, H, A, UC>)>>;
type ROChildrenIterItem<'a, G, H, A, UN, UE> = (
    PlainPathNode,
    &'a BaseNode<G, H, A, UN>,
    &'a BaseEdge<G, H, A, UE>,
);
type MutChildrenIterItem<'a, G, H, A, UN, UE> = (
    PlainPathNode,
    &'a mut BaseNode<G, H, A, UN>,
    &'a mut BaseEdge<G, H, A, UE>,
);

#[derive(Clone)]
pub struct PlainTree<G, H, A, USP, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    USP: UCTStatsPolicy<G, H, A>,
    TT: TranspositionTable<G::State, usize>,
{
    nodes: Vec<BaseNode<G, H, A, USP::Node>>,
    edges: Edges<G, H, A, USP::Edge>,
    transposition_table: TT,
    root_id: usize,
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

    #[allow(clippy::type_complexity)]
    fn get_mut_parent_and_ro_children<'a>(
        &'a mut self,
        parent_id: usize,
    ) -> (
        &'a mut BaseNode<G, H, A, USP::Node>,
        impl Iterator<Item = ROChildrenIterItem<'a, G, H, A, USP::Node, USP::Edge>>,
    ) {
        // set references to edges and nodes to enable separate reference handling
        let (nodes, edges) = (&mut self.nodes, &self.edges);
        // split nodes to enable separated reference access to parent and children nodes.
        // since we may have a transposition table, a child may be before it's parent.
        let (before_parent, rest) = nodes.split_at_mut(parent_id);
        let (parent, after_parent) = rest.split_first_mut().expect("parent_id in range");
        // make before_parent and after_parent ro
        let before_parent = &*before_parent;
        let after_parent = &*after_parent;
        // get iter
        let children_iter =
            edges[parent_id]
                .iter()
                .enumerate()
                .map(move |(edge_idx, (child_id, edge))| {
                    let node = if *child_id > parent_id {
                        &after_parent[child_id - parent_id - 1]
                    } else {
                        &before_parent[*child_id]
                    };
                    (
                        PlainPathNode::Via {
                            parent_id,
                            edge_idx,
                        },
                        node,
                        edge,
                    )
                });
        (parent, children_iter)
    }

    #[allow(clippy::type_complexity)]
    fn get_mut_parent_and_mut_children<'a>(
        &'a mut self,
        parent_id: usize,
    ) -> (
        &'a mut BaseNode<G, H, A, USP::Node>,
        impl Iterator<Item = MutChildrenIterItem<'a, G, H, A, USP::Node, USP::Edge>>,
    ) {
        let nodes_len = self.nodes.len();
        // we need raw pointer to nodes to provide &mut in iterator
        let nodes_ptr = self.nodes.as_mut_ptr();
        let edges = &mut self.edges[parent_id];

        debug_assert!(parent_id < nodes_len);
        debug_assert!(
            edges.iter().enumerate().all(|(i, (id, _))| {
                *id < nodes_len && *id != parent_id && edges[..i].iter().all(|(o, _)| o != id)
            }),
            "child ids must be in range, differ from the parent and be pairwise distinct"
        );

        // SAFETY: parent_id < nodes_len; all child ids are in range, pairwise distinct and
        // different from parent_id (invariant, see add_edge and link_edge). Hence all created
        // &mut references point to distinct elements. The edges live in a separate field, and
        // `&'a mut self` prevents any other access while the references are alive.
        let parent = unsafe { &mut *nodes_ptr.add(parent_id) };
        let children = edges
            .iter_mut()
            .enumerate()
            .map(move |(edge_idx, (child_id, edge))| {
                let child = unsafe { &mut *nodes_ptr.add(*child_id) };
                (
                    PlainPathNode::Via {
                        parent_id,
                        edge_idx,
                    },
                    child,
                    edge,
                )
            });
        (parent, children)
    }

    fn inner_update_solver(&mut self, id: usize) -> bool {
        let (parent, children_iter) = self.get_mut_parent_and_ro_children(id);
        parent.update_solver(children_iter.map(|(_, c, _)| c))
    }

    fn inner_link_child(
        &mut self,
        parent_id: usize,
        mv: G::Move,
        child_id: usize,
    ) -> PlainPathNode {
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

    fn get_parent_accumulated_value(&self, id: usize) -> f32 {
        // 1.) check if node returns accumulated value (Option<f32>)
        // 2.) if none, nodes do not contain visits -> edge based, but no incoming edge available
        // 3.) iter edges and return sum accumulated value of each edge
        if let Some(acc) = self.nodes[id].get_accumulated_value() {
            acc
        } else {
            self.edges[id]
                .iter()
                .filter_map(|(_, e)| e.get_accumulated_value())
                .sum()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlainPathNode {
    Root(usize),
    Via { parent_id: usize, edge_idx: usize },
}

impl PlainPathNode {
    fn get_parent_id(&self) -> usize {
        match self {
            PlainPathNode::Root(root_id) => *root_id,
            PlainPathNode::Via { parent_id, .. } => *parent_id,
        }
    }
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

    fn get_cached_node_id(&self, state: &G::State) -> Option<Self::NodeID> {
        self.transposition_table.get(state).copied()
    }

    fn get_node_state(&self, id: Self::NodeID) -> &G::State {
        self.nodes[id].get_state()
    }

    fn set_node_outcome(&mut self, id: Self::NodeID, outcome: G::Outcome) {
        self.nodes[id].set_outcome(outcome);
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
        self.inner_link_child(parent_id, mv, child_id)
    }

    fn link_child(
        &mut self,
        parent_id: Self::NodeID,
        mv: G::Move,
        child_id: Self::NodeID,
    ) -> Self::PathNode {
        // assert valid input for linking
        assert_ne!(parent_id, child_id);
        assert!(child_id < self.nodes.len());
        assert!(self.edges[parent_id].iter().all(|e| e.0 != child_id));
        self.inner_link_child(parent_id, mv, child_id)
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

    fn get_target_accumulated_value(&self, path_node: Self::PathNode) -> f32 {
        // 1.) check if path_node points to root or some node
        // 2.) root: call fn get_parent_accumulated_value(root_id)
        // 3.) some node: check if node returns accumulated value (Option<f32>)
        // 4.) if none, nodes do not contain accumulated vale -> check edge accumulated value
        // 5.) edge returns accumulated value (Option<f32>) -> expect Some(f32)
        match path_node {
            PlainPathNode::Root(root_id) => self.get_parent_accumulated_value(root_id),
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => {
                if let Some(acc) = self.nodes[self.target(path_node)].get_accumulated_value() {
                    acc
                } else {
                    self.edges[parent_id][edge_idx]
                        .1
                        .get_accumulated_value()
                        .expect("node or edge must contain accumulated value.")
                }
            }
        }
    }

    fn get_target_outcome(&self, path_node: Self::PathNode) -> Option<G::Outcome> {
        self.nodes[self.target(path_node)].get_outcome()
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
            // previous_player for exploitation score is current player of root
            let previous_player = G::current_player(self.nodes[self.root_id].get_state());
            self.edges[self.root_id]
                .iter()
                .map(|(_, e)| e.calc_exploitation_score(previous_player))
                .sum()
        }
    }

    fn select_child_with_best_uct(
        &mut self,
        path_node: Self::PathNode,
        mcts_config: &A::Config,
    ) -> Result<Self::PathNode, bool> {
        // in selection, target of path_node is current node id
        let parent_visits = self.get_parent_visits(self.target(path_node));
        let (parent, children_iter) = self.get_mut_parent_and_mut_children(self.target(path_node));
        let is_fully_expanded = parent.is_fully_expanded();
        let mut best_path_node: Option<_> = None;
        let mut best_uct = f32::NEG_INFINITY;

        for (pn, child, edge_to_child) in
            children_iter.filter(|(_, c, _)| parent.filter_selection(c))
        {
            let node_uct = child.calc_exploitation_score()
                + child.calc_exploration_score(parent_visits, mcts_config);
            let previous_player = G::previous_player(child.get_state());
            let edge_uct = edge_to_child.calc_exploitation_score(previous_player)
                + edge_to_child.calc_exploration_score(previous_player, parent_visits, mcts_config);

            let uct = node_uct + edge_uct;
            if uct > best_uct {
                best_uct = uct;
                best_path_node = Some(pn);
            }
        }

        best_path_node.ok_or_else(|| {
            if is_fully_expanded {
                self.inner_update_solver(self.target(path_node));
                assert!(
                    self.get_target_outcome(path_node).is_some(),
                    "no path nodes for selection and update of parent solver did not solve it."
                );

                false
            } else {
                true
            }
        })
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
        self.nodes[id].should_expand(visits, num_children, mcts_config, heuristic_config)
    }

    fn pop_target_expandable_move(
        &mut self,
        path_node: Self::PathNode,
        force: bool,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move> {
        let visits = self.get_target_visits(path_node);
        let id = self.target(path_node);
        let num_children = self.count_children(id);
        self.nodes[id].pop_expandable_move(
            visits,
            num_children,
            force,
            mcts_config,
            heuristic_config,
        )
    }

    fn update_stats(&mut self, path_node: Self::PathNode, result: f32) {
        match path_node {
            PlainPathNode::Root(root_id) => {
                self.nodes[root_id].update_stats(result);
            }
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

    fn update_solver(&mut self, path_node: Self::PathNode, previous_solver_changed: bool) -> bool {
        let node_id = path_node.get_parent_id();
        // update solver, if previous solver changed and return, if self solver changed
        if previous_solver_changed {
            self.inner_update_solver(node_id)
        } else {
            false
        }
    }

    fn update_root_solver(&mut self) -> bool {
        self.inner_update_solver(self.root_id)
    }

    fn filter_select_move(&self, path_node: Self::PathNode) -> bool {
        match path_node {
            PlainPathNode::Root(_) => true,
            PlainPathNode::Via {
                parent_id,
                edge_idx,
            } => self.nodes[parent_id]
                .filter_select_move(&self.nodes[self.edges[parent_id][edge_idx].0]),
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

// pruning to root is only allowed for PlainTree, if no transposition table is used,
// since the table would contain old (and therefore wrong) indices after pruning.
impl<G, H, A, USP, TT> PruneToRoot for PlainTree<G, H, A, USP, TT>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    USP: UCTStatsPolicy<G, H, A>,
    TT: TranspositionTable<G::State, usize>,
{
    fn prune_to_root(&mut self) {
        let Some(root_id) = self.root_id() else {
            // uninitialized tree
            return;
        };
        if root_id == 0 {
            // nothing to prune
            return;
        }

        // remove all nodes and edges before root_id
        self.nodes.drain(..root_id);
        self.edges.drain(..root_id);

        // reindex all remaining children
        for edge in self.edges.iter_mut() {
            for (child_id, _) in edge.iter_mut() {
                *child_id -= root_id;
            }
        }

        // reset root id
        self.set_root(0);
        // refill transposition table
        self.transposition_table.clear();
        let (tt, nodes) = (&mut self.transposition_table, &self.nodes);
        for (id, node) in nodes.iter().enumerate() {
            tt.insert(node.get_state().clone(), id);
        }
    }
}
