// type definition and functions of MCTSNode and MCTSTree
// since these traits are coupled by MCTSNode::ID, they always have to be implemented together

use rand::seq::SliceRandom;

use super::{ExpansionPolicy, Heuristic, MCTSGame, MCTSNode, MCTSTree, UCTPolicy, UTCCache};

// plain implementation of MCTSNode
pub struct PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    pub state: G::State,
    pub visits: usize,
    pub accumulated_value: f32,
    pub mv: Option<G::Move>,
    pub children: Vec<usize>,
    pub utc_cache: UC,
    pub expansion_policy: EP,

    phantom: std::marker::PhantomData<(UP, H)>,
}

impl<G, UP, UC, EP, H> MCTSNode<G, EP, H> for PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    type ID = usize;

    fn init_root_id() -> Self::ID {
        0 // The root node is always at index 0
    }
    fn root_node(state: G::State, expansion_policy: EP) -> Self {
        PlainNode {
            expansion_policy,
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: None,
            children: vec![],
            utc_cache: UC::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn new(state: G::State, mv: G::Move, expansion_policy: EP) -> Self {
        PlainNode {
            expansion_policy,
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: Some(mv),
            children: vec![],
            utc_cache: UC::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn add_child(&mut self, child_id: Self::ID) {
        self.children.push(child_id);
    }
    fn get_children(&self) -> &[Self::ID] {
        &self.children[..]
    }
    fn get_state(&self) -> &G::State {
        &self.state
    }
    fn get_move(&self) -> Option<&G::Move> {
        self.mv.as_ref()
    }
    fn get_visits(&self) -> usize {
        self.visits
    }
    fn get_accumulated_value(&self) -> f32 {
        self.accumulated_value
    }
    fn update_stats(&mut self, result: f32) {
        self.visits += 1;
        self.accumulated_value += result;
        self.utc_cache.update_exploitation(
            self.visits,
            self.accumulated_value,
            G::last_player(&self.state),
            G::perspective_player(),
        );
    }
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &G::Config,
    ) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.utc_cache.get_exploitation(
            self.visits,
            self.accumulated_value,
            G::last_player(&self.state),
            perspective_player,
        );
        self.utc_cache
            .update_exploration(self.visits, parent_visits, mcts_config);
        let exploration = self
            .utc_cache
            .get_exploration(self.visits, parent_visits, mcts_config);
        exploitation + exploration
    }
    fn expansion_policy(&self) -> &EP {
        &self.expansion_policy
    }
    fn expansion_policy_mut(&mut self) -> &mut EP {
        &mut self.expansion_policy
    }
    fn expandable_moves(
        &mut self,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        let mut expandable_moves = self
            .expansion_policy
            .expandable_moves(
                self.visits,
                self.children.len(),
                &self.state,
                mcts_config,
                heuristic_config,
            )
            .collect::<Vec<_>>();
        expandable_moves.shuffle(&mut rand::thread_rng());
        expandable_moves
    }
}

// plain implementation of MCTSTree
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

    fn init_root(&mut self, root_value: PlainNode<G, UP, UC, EP, H>) -> usize {
        self.nodes.clear(); // Clear any existing nodes
        self.nodes.push(root_value);
        self.root_id = PlainNode::<G, UP, UC, EP, H>::init_root_id();
        self.root_id
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

    fn get_node(&self, id: usize) -> &PlainNode<G, UP, UC, EP, H> {
        &self.nodes[id]
    }

    fn get_node_mut(&mut self, id: usize) -> &mut PlainNode<G, UP, UC, EP, H> {
        &mut self.nodes[id]
    }

    fn add_child(&mut self, parent_id: usize, child_value: PlainNode<G, UP, UC, EP, H>) -> usize {
        let child_id = self.nodes.len();
        self.get_node_mut(parent_id).add_child(child_id);
        self.nodes.push(child_value);
        child_id
    }
}
