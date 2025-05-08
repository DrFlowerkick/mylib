// type definition and functions of mcts node

use rand::seq::SliceRandom;

use super::{ExpansionPolicy, Heuristic, MCTSGame, MCTSNode, UCTPolicy, UTCCache};

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

impl<G, UP, UC, EP, H> PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    pub fn root_node(state: G::State, expansion_policy: EP) -> Self {
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
    pub fn new(state: G::State, mv: G::Move, expansion_policy: EP) -> Self {
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
    pub fn add_child(&mut self, child_index: usize) {
        self.children.push(child_index);
    }
    pub fn get_children(&self) -> &Vec<usize> {
        &self.children
    }
    pub fn expandable_moves<'a>(&'a mut self) -> Vec<G::Move> {
        let mut expandable_moves = self
            .expansion_policy
            .expandable_moves(self.visits, self.children.len(), &self.state)
            .collect::<Vec<_>>();
        expandable_moves.shuffle(&mut rand::thread_rng());
        expandable_moves
    }
}

impl<G, UP, UC, EP, H> MCTSNode<G> for PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
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
            G::current_player(&self.state),
            G::perspective_player(),
        );
    }
    fn calc_utc(&mut self, parent_visits: usize, c: f32, perspective_player: G::Player) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.utc_cache.get_exploitation(
            self.visits,
            self.accumulated_value,
            G::current_player(&self.state),
            perspective_player,
        );
        self.utc_cache
            .update_exploration(self.visits, parent_visits, c);
        let exploration = self
            .utc_cache
            .get_exploration(self.visits, parent_visits, c);
        exploitation + exploration
    }
}
