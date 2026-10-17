// Plain implementation of MCTS

use super::{
    ExpansionPolicy, Heuristic, MCTSAlgo, MCTSConfig, MCTSGame, MCTSNode, MCTSTree,
    SimulationPolicy, UCTPolicy,
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
        let expansion_policy = EP::new(state, &self.heuristic_config);
        tree.init_root(state.clone(), expansion_policy);
    }

    fn iterate(&self, tree: &mut impl MCTSTree<G, H, Self>) {
        // separate parameters of PlainMCTS to satisfy borrow checker
        // ToDo: we can probably delete this
        //let (mcts_config, heuristic_config) = (&self.mcts_config, &self.heuristic_config);

        let root_id = tree
            .root_id()
            .expect("Tree root must be initialized before iterate.");
        let mut path = vec![root_id];
        let mut current_id = root_id;

        /* ----------------- Select & Expand ----------------- */
        // Select and expand until at least one new child is created or a leaf without children is found.
        // We need to place the selection and expansion in a loop, because it can happen that we reach a
        // node with no children, but all possible moves lead to states that are already in the
        // transposition table. In this case we cannot create any new child nodes, but we also cannot
        // simulate the node, since it has no children. Therefore we need to continue selection from the last
        // node in the path, which is now linked to one or mode nodes in the transposition table. The selection
        // can now continue to these newly linked nodes.
        loop {
            // Selection
            while !tree.get_children(current_id).is_empty() {
                let parent_visits = tree.get_node(current_id).get_visits();
                let num_parent_children = tree.get_children(current_id).len();
                // check expansion policy
                if tree.get_node(current_id).should_expand(
                    parent_visits,
                    num_parent_children,
                    &self.mcts_config,
                    &self.heuristic_config,
                ) {
                    break;
                }

                let mut best_child_index: Option<_> = None;
                let mut best_utc = f32::NEG_INFINITY;

                for vec_index in 0..num_parent_children {
                    let (child_index, _) = tree.get_children(current_id)[vec_index];
                    let utc = tree
                        .get_node_mut(child_index)
                        .calc_utc(parent_visits, &self.mcts_config);
                    if utc > best_utc {
                        best_utc = utc;
                        best_child_index = Some(child_index);
                    }
                }
                let best_child_index = best_child_index.expect("Could not find best child index.");
                path.push(best_child_index);
                current_id = best_child_index;
            }

            // EP; force creation of nodes if current_id is root of tree
            if (tree.get_node(current_id).get_visits() == 0 && current_id != root_id)
                || G::evaluate(tree.get_node(current_id).get_state()).is_some()
            {
                // If the node has not been visited yet or is terminal, we need to simulate it.
                break;
            } else {
                // If the node has been visited, we need to expand it.
                let num_parent_children = tree.get_children(current_id).len();
                // Let rust decide on type of NodeID
                let mut new_children: Vec<_> = Vec::new();
                // ToDo: refactor expandable moves to next_expandable_move()
                // I used in the past an array, because basic ExpansionPolicy is to always expand all
                // possible moves. Therefore I have no need to cache possible moves, which will be
                // expanded immediately. BUT I change the way how to handle a node, which has already
                // been visited via another path (using the TranspositionTable trait). Now the strategy is,
                // that if I stumble during expansion over an existing node, I add this node to path
                // and go back to selection immediately. This may result in some moves not te be expanded.
                // Therefore I have to cache moves although for the basic ExpansionPolicy.
                // A side effect is, that next_expandable_move() is better for dynamic memory,
                // since I do not have to allocate memory for the vector of expandable_moves.
                let expandable_moves = tree.get_node_mut(current_id).expandable_moves_s(
                    num_parent_children,
                    &self.mcts_config,
                    &self.heuristic_config,
                );
                assert!(
                    !expandable_moves.is_empty(),
                    "expandable moves should never be empty."
                );

                // generate new children nodes from expandable moves
                for mv in expandable_moves {
                    let new_state = G::apply_move(tree.get_node(current_id).get_state(), &mv);
                    // ToDo: this is double link to a node in the tree.
                    // But how do we use double links?
                    // What happens, if we visit this node again later? Back propagation will only move back
                    // along the path of the current iteration, the second parent will not be updated, since
                    // it is not part of the path.
                    // So why do it at all? Because we reduce the number of nodes and therefore branches
                    // in the tree. Regarding the back propagation we do not benefit from the double link,
                    // since we do not update the second parent.
                    // But we benefit from first time linking to a node that has already been visited, since we
                    // can use its accumulated value for back propagation, see below. Therefore this path instantly
                    // gets a value of a branch that has already been explored. This is a good thing, since we do
                    // not have to explore the same branch again. But see comment below, where I discuss the possibility
                    // of moving to the already explored node instead of back propagating its value.
                    //
                    // The big question is: should we back propagate along multiply parents? What is the
                    // state of the art in MCTS regarding this? I have to check the literature.
                    // The answer is: no, we should not back propagate along multiply parents. The reason is that the
                    // back propagation is based on the path that was taken during the selection and expansion phase.
                    // Researching this question I realized, that a transposition table changes the way UTC works,
                    // since now separate parents may visit the same child. UTC is based upon one formula:
                    // N(Parent) = sum(N(Child)) for all children of Parent.
                    // To always comply with this formula, we have to separate visits from different parents for the
                    // exploration_score. These visits should be saved in the edge. On the other hand for the
                    // exploitation_score we can use the accumulated value and visits of the child, since this is
                    // independent of the parent. So we have to separate the visits for exploration and exploitation.
                    //
                    // To as much flexible as possible, we will use traits for exploitation and exploration, which may
                    // be used by node and/ or edge. We although add a "coordination" trait, which implementations
                    // decide, how to use the exploitation and exploration traits from node and edge.
                    if let Some(cached_node_id) = tree.get_cached_node_id(&new_state) {
                        tree.link_child(current_id, mv, cached_node_id);
                        let visits = tree.get_node(cached_node_id).get_visits();
                        if visits == 0 {
                            // Node does exist, but has not been visited yet. This can happen, if the node was created
                            // in a previous iteration, but has not been selected yet.
                            new_children.push(cached_node_id);
                        } else {
                            // The node has already been visited and hence explored, so we use its accumulated value divided by
                            // the number of visits to back propagate along the path.
                            // And since this node has already been explored, we do not add it to the new_children vector,
                            // since we do not want to select it again in this iteration.

                            // ToDo: instead of back propagating here we could set this node as current node and move to new
                            // cycle of selection and expansion. This would be more consistent with the MCTS algorithm,
                            // since we would move along an explored path instead of back propagating some value of a sub tree.
                            let get_accumulated_value =
                                tree.get_node(cached_node_id).get_accumulated_value();
                            back_propagation(tree, &path, get_accumulated_value / visits as f32);
                        }
                        continue;
                    }
                    let expansion_policy = EP::new(&new_state, &self.heuristic_config);
                    let new_child_id = tree.add_child(current_id, mv, new_state, expansion_policy);
                    new_children.push(new_child_id);
                }
                // Just take the first newly added child if any.
                // They are all equivalent, since they have not been visited yet.
                let Some(child_index) = new_children.first() else {
                    // ToDo: is this possibly? AND is continue what I want here?
                    continue;
                };
                path.push(*child_index);
                current_id = *child_index;
                break;
            };
        }

        // SP
        // simulation result is expected as follows:
        // 1.0: win for me
        // 0.5: tie
        // 0.0: win for opponent
        // or some heuristic value between 0.0 and 1.0
        let mut current_state = tree.get_node(current_id).get_state().clone();
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

        // back propagation
        back_propagation(tree, &path, simulation_result);
    }

    fn select_move<'a>(&'a self, tree: &'a impl MCTSTree<G, H, Self>) -> &'a G::Move {
        let root_id = tree
            .root_id()
            .expect("Root node must be initialized before selecting a move");
        let (_, mv) = tree
            .get_children(root_id)
            .iter()
            .max_by_key(|&&(child_id, _)| tree.get_node(child_id).get_visits())
            .expect("could not find move id");
        mv
    }
}

fn back_propagation<G, H, A, T>(tree: &mut T, path: &[T::NodeID], result: f32)
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
{
    for &node_id in path.iter().rev() {
        tree.get_node_mut(node_id).update_stats(result);
    }
}
