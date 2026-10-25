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
    type UCT = UP;
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

        let mut current_path_node = tree.root_path_node();
        let mut path = vec![current_path_node];

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
            while tree.count_children(tree.target(current_path_node)) > 0 {
                // check expansion policy
                if tree.should_target_expand(
                    current_path_node,
                    &self.mcts_config,
                    &self.heuristic_config,
                ) {
                    break;
                }

                let mut best_path_node: Option<_> = None;
                let mut best_uct = f32::NEG_INFINITY;
                let children_steps: Vec<_> = tree
                    .get_children_path_nodes(tree.target(current_path_node))
                    .collect();

                for path_node in children_steps {
                    let uct = tree.calc_uct(path_node, &self.mcts_config);
                    if uct > best_uct {
                        best_uct = uct;
                        best_path_node = Some(path_node);
                    }
                }
                let best_path_node = best_path_node.expect("Could not find best child index.");
                path.push(best_path_node);
                current_path_node = best_path_node;
            }

            // Expansion
            // path.len() > 1 -> path is not at root
            let current_id = tree.target(current_path_node);
            if (tree.get_target_visits(current_path_node) == 0 && path.len() > 1)
                || G::evaluate(tree.get_node_state(current_id)).is_some()
            {
                // If the node is not root and has not been visited yet or is terminal, we need to simulate it.
                break 'selection_and_expansion;
            }

            // If the node has been visited and is not terminal or if we are at root, we need to expand it.
            let mut selected_path_node = None;

            // generate new children nodes from expandable moves
            while let Some(mv) = tree.pop_target_expandable_move(
                current_path_node,
                &self.mcts_config,
                &self.heuristic_config,
            ) {
                let new_state = G::apply_move(tree.get_node_state(current_id), &mv);
                if let Some(cached_node_id) = tree.get_cached_node_id(&new_state) {
                    let linked_path_node = tree.link_child(current_id, mv, cached_node_id);
                    if tree.get_target_visits(linked_path_node) == 0 {
                        // Node does exist, but has not been visited yet. This can happen, if the node was created
                        // in a previous iteration, but has not been selected yet.
                        selected_path_node = Some(linked_path_node);
                    } else {
                        // The existing node has already been visited. We select linked_path_node as current_path_node,
                        // add linked_path_node to path, and go back to selection.
                        path.push(linked_path_node);
                        current_path_node = linked_path_node;
                        continue 'selection_and_expansion;
                    }
                } else {
                    let new_child_path_node = tree.add_child(current_id, mv, new_state);
                    selected_path_node = Some(new_child_path_node);
                }
            }

            let Some(selected_path_node) = selected_path_node else {
                panic!(
                    "if a node is not final, there should always be at least on expandable move."
                );
            };
            path.push(selected_path_node);
            current_path_node = selected_path_node;
            break;
        }

        /* ----------------- Simulation ----------------- */
        // simulation result is expected as follows:
        // 1.0: win for me
        // 0.5: tie
        // 0.0: win for opponent
        // or some heuristic value between 0.0 and 1.0
        let mut current_state = tree.get_node_state(tree.target(current_path_node)).clone();
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
            .get_children_path_nodes(root_id)
            .max_by_key(|&pn| tree.get_visits_from_parent(pn))
            .expect("could not identify max visited child");
        tree.get_move(most_visited_child_step)
            .expect("could not find move")
    }
}
