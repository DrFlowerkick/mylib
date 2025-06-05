// Plain implementation of MCTS

use super::{
    ExpansionPolicy, GameCache, Heuristic, HeuristicCache, MCTSAlgo, MCTSGame, MCTSNode, MCTSTree,
    PlainNode, PlainTree, SimulationPolicy, UCTPolicy, UTCCache,
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
    pub tree: PlainTree<G, UP, UC, EP, H>,
    pub mcts_config: G::Config,
    pub heuristic_config: H::Config,
    pub game_cache: G::Cache,
    pub heuristic_cache: H::Cache,
    phantom: std::marker::PhantomData<(UP, UC, EP, SP)>,
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
    pub fn new(mcts_config: G::Config, heuristic_config: H::Config) -> Self {
        Self {
            tree: PlainTree::<G, UP, UC, EP, H>::new(),
            mcts_config,
            heuristic_config,
            game_cache: G::Cache::new(),
            heuristic_cache: H::Cache::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn set_mcts_config(&mut self, mcts_config: G::Config) {
        self.mcts_config = mcts_config;
    }

    pub fn set_heuristic_config(&mut self, heuristic_config: H::Config) {
        self.heuristic_config = heuristic_config;
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
        if let Some(root_id) = self.tree.root_id() {
            // search for node with same state
            // unwrap() is safe here because we just checked that root_id exists
            if let Some((new_root_id, _)) = self
                .tree
                .get_node(root_id)
                .get_children()
                .iter()
                .map(|&my_move_node_id| self.tree.get_node(my_move_node_id))
                .flat_map(|my_move_node| {
                    my_move_node
                        .get_children()
                        .iter()
                        .map(|&opponent_move_node_id| {
                            (opponent_move_node_id, self.tree.get_node(opponent_move_node_id).get_state())
                        })
                })
                .find(|(_, opponent_move_node_state)| *opponent_move_node_state == state)
            {
                self.tree.set_root(new_root_id);
                return true;
            }
        }

        let expansion_policy = EP::new(
            state,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            &self.heuristic_config,
        );
        let new_root = PlainNode::root_node(state.clone(), expansion_policy);
        self.tree.init_root(new_root);
        false
    }

    fn iterate(&mut self) {
        // separate parameters of PlainMCTS to satisfy borrow checker
        let (tree, mcts_config, heuristic_config, game_cache, heuristic_cache) = (
            &mut self.tree,
            &self.mcts_config,
            &self.heuristic_config,
            &mut self.game_cache,
            &mut self.heuristic_cache,
        );

        // Ensure the root node is initialized
        let root_id = tree
            .root_id()
            .expect("Root node must be initialized before iterating");
        let mut path = vec![root_id];
        let mut current_id = root_id;

        // Selection
        while !tree.get_node(current_id).get_children().is_empty() {
            let parent_visits = tree.get_node(current_id).get_visits();
            let num_parent_children = tree.get_node(current_id).get_children().len();
            // check expansion policy
            if tree.get_node(current_id).expansion_policy().should_expand(
                parent_visits,
                num_parent_children,
                mcts_config,
                heuristic_config,
            ) {
                break;
            }

            let mut best_child_index: Option<_> = None;
            let mut best_utc = f32::NEG_INFINITY;

            for vec_index in 0..num_parent_children {
                let child_index = tree
                    .get_node(current_id)
                    .get_children()[vec_index];
                let utc = tree
                    .get_node_mut(child_index)
                    .calc_utc(parent_visits, G::perspective_player(), mcts_config);
                if utc > best_utc {
                    best_utc = utc;
                    best_child_index = Some(child_index);
                }
            }
            let best_child_index = best_child_index.expect("Could not find best child index.");
            path.push(best_child_index);
            current_id = best_child_index;
        }

        // Expansion; force creation of nodes if current_id is root of tree
        let current_id = if (tree.get_node(current_id).get_visits() == 0 && current_id != root_id)
            || G::evaluate(tree.get_node(current_id).get_state(), game_cache).is_some()
        {
            // If the node has not been visited yet or is terminal, we need to simulate it.
            current_id
        } else {
            // If the node has been visited, we need to expand it.
            let num_parent_children = tree.get_node(current_id).get_children().len();
            let expandable_moves = tree
                .get_node_mut(current_id)
                .expandable_moves(&self.mcts_config, heuristic_config);

            // generate new children nodes from expandable moves
            for mv in expandable_moves {
                let new_state =
                    G::apply_move(tree.get_node(current_id).get_state(), &mv, game_cache);
                let expansion_policy =
                    EP::new(&new_state, game_cache, heuristic_cache, heuristic_config);
                let new_node = PlainNode::new(new_state, mv, expansion_policy);
                tree.add_child(current_id, new_node);
            }
            // take the first newly added child
            let child_index = tree
                .get_node(current_id)
                .get_children()
                .get(num_parent_children)
                .expect("No children at current node");
            path.push(*child_index);
            *child_index
        };

        // Simulation
        // simulation result is expected as follows:
        // 1.0: win for me
        // 0.5: tie
        // 0.0: win for opponent
        // or some heuristic value between 0.0 and 1.0
        let mut current_state = tree.get_node(current_id).get_state().clone();
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
                &self.mcts_config,
                &self.heuristic_config,
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
        for &node_id in path.iter().rev() {
            tree.get_node_mut(node_id).update_stats(simulation_result);
        }
    }

    fn select_move(&self) -> &G::Move {
        let root_id = self
            .tree
            .root_id()
            .expect("Root node must be initialized before selecting a move");
        let move_id= self
            .tree
            .get_node(root_id)
            .get_children()
            .iter()
            .max_by_key(|&&child_id| self.tree.get_node(child_id).get_visits())
            .expect("could not find move id");
        self.tree
            .get_node(*move_id)
            .get_move()
            .expect("node did not contain move")
    }
}
