use super::{
    ExpansionPolicy, GameCache, Heuristic, HeuristicCache, MCTSAlgo, MCTSGame, MCTSNode, PlainNode,
    SimulationPolicy, UCTPolicy, UTCCache,
};
use rand::prelude::IteratorRandom;

pub struct PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    pub nodes: Vec<PlainNode<G, UP, UC, EP, H>>,
    pub root_index: usize,
    pub exploration_constant: f32,
    pub depth: usize,
    pub alpha: f32,
    pub game_cache: G::Cache,
    pub heuristic_cache: H::Cache,
    phantom: std::marker::PhantomData<SP>,
}

impl<G, UP, UC, EP, H, SP> PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    pub fn new(exploration_constant: f32, depth: usize, alpha: f32) -> Self {
        Self {
            nodes: vec![],
            root_index: 0,
            exploration_constant,
            depth,
            alpha,
            game_cache: G::Cache::new(),
            heuristic_cache: H::Cache::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn set_exploration_constant(&mut self, exploration_constant: f32) {
        self.exploration_constant = exploration_constant;
    }

    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }
}

impl<G, UP, UC, EP, H, SP> MCTSAlgo<G> for PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    fn set_root(&mut self, state: &G::State) -> bool {
        // tree is empty, if PlainMCTS was just created
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
        let expansion_policy = EP::new(
            state,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            self.depth,
            self.alpha,
        );
        self.nodes
            .push(PlainNode::root_node(state.clone(), expansion_policy));
        self.root_index = 0;
        false
    }

    fn iterate(&mut self) {
        let mut path = vec![self.root_index];
        let mut current_index = self.root_index;

        // Selection
        while !self.nodes[current_index].get_children().is_empty() {
            let parent_visits = self.nodes[current_index].get_visits();
            let num_parent_children = self.nodes[current_index].get_children().len();
            // check expansion policy
            if self.nodes[current_index]
                .expansion_policy
                .should_expand(parent_visits, num_parent_children)
            {
                break;
            }

            let mut best_child_index = 0;
            let mut best_utc = f32::NEG_INFINITY;

            for vec_index in 0..num_parent_children {
                let child_index = self.nodes[current_index].get_children()[vec_index];
                let utc = self.nodes[child_index].calc_utc(
                    parent_visits,
                    self.exploration_constant,
                    G::perspective_player(),
                );
                if utc > best_utc {
                    best_utc = utc;
                    best_child_index = child_index;
                }
            }
            path.push(best_child_index);
            current_index = best_child_index;
        }

        // Expansion
        let current_index = if self.nodes[current_index].get_visits() == 0
            || G::evaluate(self.nodes[current_index].get_state(), &mut self.game_cache).is_some()
        {
            // If the node has not been visited yet or is terminal, we need to simulate it.
            current_index
        } else {
            // If the node has been visited, we need to expand it.
            let num_parent_children = self.nodes[current_index].get_children().len();
            let expandable_moves = self.nodes[current_index].expandable_moves();

            // generate new children nodes from expandable moves
            for mv in expandable_moves {
                let new_state =
                    G::apply_move(&self.nodes[current_index].state, &mv, &mut self.game_cache);
                let expansion_policy = EP::new(
                    &new_state,
                    &mut self.game_cache,
                    &mut self.heuristic_cache,
                    self.depth,
                    self.alpha,
                );
                let new_node = PlainNode::new(new_state, mv, expansion_policy);
                self.nodes.push(new_node);
                let child_index = self.nodes.len() - 1;
                self.nodes[current_index].add_child(child_index);
            }
            // take the first newly added child
            let child_index = *self.nodes[current_index]
                .get_children()
                .get(num_parent_children)
                .expect("No children found");
            path.push(child_index);
            child_index
        };

        // Simulation
        // simulation result is expected as follows:
        // 1.0: win for me
        // 0.5: tie
        // 0.0: win for opponent
        // or some heuristic value between 0.0 and 1.0
        let mut current_state = self.nodes[current_index].get_state().clone();
        let mut depth = 0;
        let simulation_result = loop {
            if let Some(final_score) = G::evaluate(&current_state, &mut self.game_cache) {
                break final_score;
            }

            if let Some(heuristic) = SP::should_cutoff(
                &current_state,
                depth,
                &mut self.game_cache,
                &mut self.heuristic_cache,
                Some(G::perspective_player()),
            ) {
                break heuristic;
            }

            current_state = G::apply_move(
                &current_state,
                &G::available_moves(&current_state)
                    .choose(&mut rand::thread_rng())
                    .expect("No available moves"),
                &mut self.game_cache,
            );
            depth += 1;
        };

        // back propagation
        for &node_index in path.iter().rev() {
            self.nodes[node_index].update_stats(simulation_result);
        }
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
