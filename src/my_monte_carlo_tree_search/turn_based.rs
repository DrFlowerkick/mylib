use super::{MCTSAlgo, MCTSNode, MCTSGame, MonteCarloPlayer};
use rand::prelude::IteratorRandom;

pub struct TurnBasedNode<G: MCTSGame> {
    pub state: G::State,
    pub visits: usize,
    pub accumulated_value: f32,
    pub mv: Option<G::Move>,
    pub children: Vec<usize>,
}

impl<G: MCTSGame> MCTSNode<G> for TurnBasedNode<G> {
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
    fn add_simulation_result(&mut self, result: f32) {
        self.accumulated_value += result;
    }
    fn increment_visits(&mut self) {
        self.visits += 1;
    }
}

impl<G: MCTSGame> TurnBasedNode<G> {
    pub fn root_node(state: G::State) -> Self {
        TurnBasedNode {
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: None,
            children: vec![],
        }
    }
    pub fn new(state: G::State, mv: G::Move) -> Self {
        TurnBasedNode {
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: Some(mv),
            children: vec![],
        }
    }
    pub fn add_child(&mut self, child_index: usize) {
        self.children.push(child_index);
    }
    pub fn get_children(&self) -> &Vec<usize> {
        &self.children
    }
    pub fn calc_utc(&self, parent_visits: usize, c: f32) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let raw_exploitation = self.accumulated_value / self.visits as f32;
        // current_player is the now active player.
        // The exploitation score of a node is calculated from the perspective of the player
        // who acted last. With a two player game, the I'm the last player, if the current player
        // is the opponent.
        let exploitation = match G::current_player(&self.state) {
            MonteCarloPlayer::Me => 1.0 - raw_exploitation, // last player was opponent
            MonteCarloPlayer::Opp => raw_exploitation,      // last player was me
        };
        let exploration = c * ((parent_visits as f32).ln() / self.visits as f32).sqrt();
        exploitation + exploration
    }
}

pub struct TurnBasedMCTS<G: MCTSGame> {
    pub nodes: Vec<TurnBasedNode<G>>,
    pub root_index: usize,
    pub exploration_constant: f32,
}

impl<G: MCTSGame> TurnBasedMCTS<G> {
    pub fn new(exploration_constant: f32) -> Self {
        Self {
            nodes: vec![],
            root_index: 0,
            exploration_constant,
        }
    }
}

impl<G: MCTSGame> MCTSAlgo<G> for TurnBasedMCTS<G> {
    fn iterate(&mut self) {
        let mut path = vec![self.root_index];
        let mut current_index = self.root_index;

        // Selection
        while !self.nodes[current_index].get_children().is_empty() {
            let parent_visits = self.nodes[current_index].get_visits();
            let mut best_child_index = 0;
            let mut best_utc = f32::NEG_INFINITY;

            for &child_index in self.nodes[current_index].get_children() {
                let utc =
                    self.nodes[child_index].calc_utc(parent_visits, self.exploration_constant);
                if utc > best_utc {
                    best_utc = utc;
                    best_child_index = child_index;
                }
            }
            path.push(best_child_index);
            current_index = best_child_index;
        }

        // Expansion
        let current_index = if G::is_terminal(self.nodes[current_index].get_state())
            || self.nodes[current_index].get_visits() == 0
        {
            // If the node is terminal or has not been visited yet, we need to simulate it.
            current_index
        } else {
            // If the node has been visited, we need to expand it by generating its children.
            let current_state = self.nodes[current_index].get_state().clone();
            for mv in G::available_moves(&current_state) {
                let new_state = G::apply_move(&current_state, &mv);
                let new_node = TurnBasedNode::new(new_state, mv);
                self.nodes.push(new_node);
                let child_index = self.nodes.len() - 1;
                self.nodes[current_index].add_child(child_index);
            }
            let child_index = *self.nodes[current_index]
                .get_children()
                .first()
                .expect("No children found");
            path.push(child_index);
            child_index
        };

        // Simulation
        let mut current_state = self.nodes[current_index].get_state().clone();
        while !G::is_terminal(&current_state) {
            let random_move = G::available_moves(&current_state)
                .choose(&mut rand::thread_rng())
                .expect("No available moves");
            current_state = G::apply_move(&current_state, &random_move);
        }
        // simulation result is expected as follows:
        // 1.0: win for me
        // 0.5: tie
        // 0.0: win for opponent
        let simulation_result = G::evaluate(&current_state);

        // back propagation
        for &node_index in path.iter().rev() {
            self.nodes[node_index].increment_visits();
            self.nodes[node_index].add_simulation_result(simulation_result);
        }
    }

    fn set_root(&mut self, state: &G::State) -> bool {
        // tree is empty, if TurnBasedMCTS was just created
        if !self.nodes.is_empty() {
            // search for node with same state
            if let Some(new_root) = self.nodes[self.root_index]
                .get_children()
                .iter()
                .flat_map(|&my_move_nodes| self.nodes[my_move_nodes].get_children())
                .find(|&&opponent_move_nodes| self.nodes[opponent_move_nodes].get_state() == state)
            {
                self.root_index = *new_root;
                return true;
            }
        }
        self.nodes.clear();
        self.nodes.push(TurnBasedNode::root_node(state.clone()));
        self.root_index = 0;
        false
    }

    fn select_move(&self) -> &G::Move {
        let move_index = self.nodes[self.root_index]
            .get_children()
            .iter()
            .max_by_key(|&&child_index| self.nodes[child_index].get_visits())
            .expect("could not find move_index");
        self.nodes[*move_index]
            .get_move()
            .expect("node did not contain move")
    }
}
