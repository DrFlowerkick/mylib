// Plain implementation of MCTS

use super::{
    ExpansionPolicy, Heuristic, MCTSAlgo, MCTSConfig, MCTSGame, MCTSTree, SimulationPolicy,
    UCTPolicy,
};
use rand::{prelude::IteratorRandom, rng};

// Use PlainMCTS with your specific implementations of the MCTS traits.
#[derive(Clone, Copy)]
pub struct PlainMCTS<G, H, MC, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig<G::Player>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    pub mcts_config: MC,
    pub heuristic_config: H::Config,
    phantom: std::marker::PhantomData<(UP, EP, SP)>,
}

impl<G, H, MC, UP, EP, SP> PlainMCTS<G, H, MC, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig<G::Player>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    pub fn new(mcts_config: MC, heuristic_config: H::Config) -> Self {
        Self {
            mcts_config,
            heuristic_config,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, H, MC, UP, EP, SP> MCTSAlgo<G, H> for PlainMCTS<G, H, MC, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig<G::Player>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    type Config = MC;
    type UTC = UP;
    type Expansion = EP;
    type Simulation = SP;

    fn set_root(
        &self,
        state: &G::State,
        tree: &mut impl MCTSTree<G, H, Self>,
        max_depth: usize,
    ) -> bool {
        // check for state in tree limited by max_depth
        if let Some(root_id) = tree.root_id()
            && let Some(node_of_state_id) = tree.get_node_id(state, root_id, max_depth)
        {
            tree.set_root(node_of_state_id);
            return true;
        }
        // state not found -> reset root
        self.reset_root(state, tree);
        false
    }

    fn reset_root(&self, state: &<G as MCTSGame>::State, tree: &mut impl MCTSTree<G, H, Self>) {
        tree.init_root(state.clone());
    }

    fn iterate(&self, tree: &mut impl MCTSTree<G, H, Self>) {
        // separate parameters of PlainMCTS to satisfy borrow checker
        // ToDo: we can probably delete this
        //let (mcts_config, heuristic_config) = (&self.mcts_config, &self.heuristic_config);

        let root_id = tree
            .root_id()
            .expect("Tree root must be initialized before iterate.");
        let mut path = vec![];
        let mut current_id = root_id;

        /* ----------------- Select & Expand ----------------- */
        // Select and expand until at least one new child is created or a leaf without children is found.
        // We need to place the selection and expansion in a loop, because it can happen that we reach a
        // node with no children, but all possible moves lead to states that are already in the
        // transposition table. In this case we cannot create any new child nodes, but we also cannot
        // simulate the node, since it has no children. Therefore we need to continue selection from the last
        // node in the path, which is now linked to one or mode nodes in the transposition table. The selection
        // can now continue to these newly linked nodes.
        'selection_and_expansion: loop {
            // Selection
            while tree.count_children(current_id) > 0 {
                // check expansion policy
                if tree.should_node_expand(current_id, &self.mcts_config, &self.heuristic_config) {
                    break;
                }

                let mut best_step: Option<_> = None;
                let mut best_utc = f32::NEG_INFINITY;
                let children_steps: Vec<_> = tree.get_children_steps(current_id).collect();

                for step in children_steps {
                    let utc = tree.calc_utc(step, &self.mcts_config);
                    if utc > best_utc {
                        best_utc = utc;
                        best_step = Some(step);
                    }
                }
                let best_step = best_step.expect("Could not find best child index.");
                path.push(best_step);
                current_id = tree.target(best_step);
            }

            // Expansion
            // force creation of nodes if current_id is root of tree
            if (tree.get_node_visits(current_id) == 0 && current_id != root_id)
                || G::evaluate(tree.get_node_state(current_id)).is_some()
            {
                // If the node has not been visited yet or is terminal, we need to simulate it.
                break;
            } else {
                // If the node has been visited, we need to expand it.
                let mut selected_step = None;

                // generate new children nodes from expandable moves
                while let Some(mv) = tree.pop_node_expandable_move(
                    current_id,
                    &self.mcts_config,
                    &self.heuristic_config,
                ) {
                    let new_state = G::apply_move(tree.get_node_state(current_id), &mv);
                    if let Some(cached_node_id) = tree.get_cached_node_id(&new_state) {
                        let step = tree.link_child(current_id, mv, cached_node_id);
                        let visits = tree.get_node_visits(cached_node_id);
                        if visits == 0 {
                            // Node does exist, but has not been visited yet. This can happen, if the node was created
                            // in a previous iteration, but has not been selected yet.
                            selected_step = Some(step);
                        } else {
                            // The existing node has already been visited. We select this node as current_id,
                            // add step to path, and go back to selection.
                            path.push(step);
                            current_id = cached_node_id;
                            continue 'selection_and_expansion;
                        }
                    } else {
                        let new_child_step = tree.add_child(current_id, mv, new_state);
                        selected_step = Some(new_child_step);
                    }
                }

                let Some(selected_step) = selected_step else {
                    panic!(
                        "if a node is not final, there should always be at least on expandable move."
                    );
                };
                path.push(selected_step);
                current_id = tree.target(selected_step);
                break 'selection_and_expansion;
            };
        }

        /* ----------------- Simulation ----------------- */
        // simulation result is expected as follows:
        // 1.0: win for me
        // 0.5: tie
        // 0.0: win for opponent
        // or some heuristic value between 0.0 and 1.0
        let mut current_state = tree.get_node_state(current_id).clone();
        let mut depth = 0;
        let simulation_result = loop {
            if let Some(final_score) = G::evaluate(&current_state) {
                break final_score;
            }

            if let Some(heuristic) = SP::should_cutoff(
                &current_state,
                depth,
                Some(G::perspective_player()),
                &self.mcts_config,
                &self.heuristic_config,
            ) {
                break heuristic;
            }

            current_state = G::apply_move(
                &current_state,
                &G::available_moves(&current_state)
                    .choose(&mut rng())
                    .expect("No available moves"),
            );
            depth += 1;
        };

        /* ----------------- Back Propagation ----------------- */
        for step in path {
            tree.update_stats(step, simulation_result);
        }
    }

    fn select_move<'a>(&'a self, tree: &'a impl MCTSTree<G, H, Self>) -> &'a G::Move {
        let root_id = tree
            .root_id()
            .expect("Root node must be initialized before selecting a move");
        let most_visited_child_step = tree
            .get_children_steps(root_id)
            .max_by_key(|&step| tree.get_node_visits(tree.target(step)))
            .expect("could not find move id");
        tree.get_move(most_visited_child_step)
    }
}
