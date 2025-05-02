use super::{
    ExpansionPolicy, Heuristic, MCTSAlgo, UTCCache, MCTSGame, MCTSNode, PlainNode,
    SimulationPolicy, UCTPolicy,
};
use rand::prelude::IteratorRandom;

pub struct PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    pub nodes: Vec<PlainNode<G, UP, UC, EP, H>>,
    pub root_index: usize,
    pub exploration_constant: f32,
    phantom: std::marker::PhantomData<SP>,
}

impl<G, UP, UC, EP, H, SP> PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    pub fn new(exploration_constant: f32) -> Self {
        Self {
            nodes: vec![],
            root_index: 0,
            exploration_constant,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, UP, UC, EP, H, SP> MCTSAlgo<G> for PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
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
        let current_index = if G::is_terminal(self.nodes[current_index].get_state())
            || self.nodes[current_index].get_visits() == 0
        {
            // If the node is terminal or has not been visited yet, we need to simulate it.
            current_index
        } else {
            // If the node has been visited, we need to expand it by generating its children.
            let visits = self.nodes[current_index].get_visits();
            let num_parent_children = self.nodes[current_index].get_children().len();
            while let Some(mv) = self.nodes[current_index]
                .expansion_policy
                .pop_expandable_move(visits, num_parent_children)
            {
                let new_state = G::apply_move(self.nodes[current_index].get_state(), &mv);
                let new_node = PlainNode::new(new_state, mv);
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
        // simulation result is expected as follows:
        // 1.0: win for me
        // 0.5: tie
        // 0.0: win for opponent
        // or some heuristic value between 0.0 and 1.0
        let mut current_state = self.nodes[current_index].get_state().clone();
        let mut depth = 0;
        let simulation_result = loop {
            if G::is_terminal(&current_state) {
                break G::evaluate(&current_state);
            }
            
            if let Some(heuristic) = SP::should_cutoff(&current_state, depth) {
                break heuristic;
            }

            current_state = G::apply_move(
                &current_state,
                &G::available_moves(&current_state)
                    .choose(&mut rand::thread_rng())
                    .expect("No available moves"),
            );
            depth += 1;
        };

        // back propagation
        for &node_index in path.iter().rev() {
            self.nodes[node_index].increment_visits();
            // if cache is used, add simulation result must be run before incrementing visits, because it updates the cache
            self.nodes[node_index].add_simulation_result(simulation_result);
        }
    }

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
        self.nodes.push(PlainNode::root_node(state.clone()));
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
