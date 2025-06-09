// Plain implementation of MCTS

use super::{
    ExpansionPolicy, GameCache, Heuristic, HeuristicCache, MCTSAlgo, MCTSConfig, MCTSGame,
    MCTSNode, MCTSTree, PlainNode, PlainTree, SimulationPolicy, TranspositionHashMap,
    TranspositionTable, UCTPolicy, UTCCache,
};
use rand::prelude::IteratorRandom;

// plain type of TranspositionHashMap, which works with PlainTree
pub type PlainTTHashMap<State> = TranspositionHashMap<State, usize>;

// Use PlainMCTS with your specific implementations of the MCTS traits.
pub struct PlainMCTS<G, H, MC, UC, TT, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    TT: TranspositionTable<G::State, usize>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    pub tree: PlainTree<G, H, Self, UC>,
    pub mcts_config: MC,
    pub heuristic_config: H::Config,
    pub game_cache: G::Cache,
    pub heuristic_cache: H::Cache,
    pub transposition_table: TT,
    phantom: std::marker::PhantomData<()>,
}

impl<G, H, MC, UC, TT, UP, EP, SP> PlainMCTS<G, H, MC, UC, TT, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    TT: TranspositionTable<G::State, usize>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    pub fn new(mcts_config: MC, heuristic_config: H::Config) -> Self {
        Self {
            tree: PlainTree::new(),
            mcts_config,
            heuristic_config,
            game_cache: G::Cache::new(),
            heuristic_cache: H::Cache::new(),
            transposition_table: TT::new(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, H, MC, UC, TT, UP, EP, SP> MCTSAlgo<G, H> for PlainMCTS<G, H, MC, UC, TT, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    TT: TranspositionTable<G::State, usize>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    type Tree = PlainTree<G, H, Self, UC>;
    type NodeID = usize;
    type Config = MC;
    type TranspositionTable = TT;
    type UTC = UP;
    type Expansion = EP;
    type Simulation = SP;

    fn set_root(&mut self, state: &G::State) -> bool {
        // tree is empty, if PlainMCTS was just created
        if let Some(root_id) = self.tree.root_id() {
            // search for node with state in transposition table
            if let Some(node_of_state_id) = self.transposition_table.get(state) {
                self.tree.set_root(*node_of_state_id);
                return true;
            }

            // search for node with state in tree. node must be a either child or grand child of root:
            // children of root: result of my or opponent moves
            // grand children of root: result of opponent or my moves
            // unwrap() is safe here because we just checked that root_id exists
            // ToDo: this is only true for two player games. transposition table is better for multiplayer
            if let Some((new_root_id, _)) =
                self.tree
                    .get_children(root_id)
                    .iter()
                    .map(|&(my_move_node_id, _)| {
                        (
                            my_move_node_id,
                            self.tree.get_node(my_move_node_id).get_state(),
                        )
                    })
                    .chain(self.tree.get_children(root_id).iter().flat_map(
                        |&(my_move_node_id, _)| {
                            self.tree.get_children(my_move_node_id).iter().map(
                                |&(opponent_move_node_id, _)| {
                                    (
                                        opponent_move_node_id,
                                        self.tree.get_node(opponent_move_node_id).get_state(),
                                    )
                                },
                            )
                        },
                    ))
                    .find(|(_, move_node_state)| *move_node_state == state)
            {
                self.tree.set_root(new_root_id);
                return true;
            }
        }
        // state not found -> reset root
        let expansion_policy = EP::new(
            state,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            &self.heuristic_config,
        );
        let new_root = PlainNode::new(state.clone(), expansion_policy);
        let root_id = self.tree.init_root(new_root);
        self.transposition_table = TT::new();
        self.transposition_table.insert(state.clone(), root_id);
        false
    }

    fn iterate(&mut self) {
        // separate parameters of PlainMCTS to satisfy borrow checker
        let (tree, mcts_config, heuristic_config, game_cache, heuristic_cache, transposition_table) = (
            &mut self.tree,
            &self.mcts_config,
            &self.heuristic_config,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            &mut self.transposition_table,
        );

        let root_id = tree
            .root_id()
            .expect("Tree root must be initialized before iterate.");
        let mut path = vec![root_id];
        let mut current_id = root_id;
        let mut new_children: Vec<Self::NodeID> = Vec::new();

        // select and expand until at least one new child is created or a leaf without children is found
        loop {
            // Selection
            while !tree.get_children(current_id).is_empty() {
                let parent_visits = tree.get_node(current_id).get_visits();
                let num_parent_children = tree.get_children(current_id).len();
                // check expansion policy
                if tree.get_node(current_id).should_expand(
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
                    let (child_index, _) = tree.get_children(current_id)[vec_index];
                    let utc = tree.get_node_mut(child_index).calc_utc(
                        parent_visits,
                        G::perspective_player(),
                        mcts_config,
                    );
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
                || G::evaluate(tree.get_node(current_id).get_state(), game_cache).is_some()
            {
                // If the node has not been visited yet or is terminal, we need to simulate it.
                break;
            } else {
                // If the node has been visited, we need to expand it.
                let num_parent_children = tree.get_children(current_id).len();
                let expandable_moves = tree.get_node_mut(current_id).expandable_moves(
                    num_parent_children,
                    mcts_config,
                    heuristic_config,
                );

                // generate new children nodes from expandable moves
                for mv in expandable_moves {
                    let new_state =
                        G::apply_move(tree.get_node(current_id).get_state(), &mv, game_cache);
                    if let Some(&cached_node_id) = transposition_table.get(&new_state) {
                        tree.link_child(current_id, mv, cached_node_id);
                        let visits = tree.get_node(cached_node_id).get_visits();
                        if visits == 0 {
                            new_children.push(cached_node_id);
                        } else {
                            let get_accumulated_value =
                                tree.get_node(cached_node_id).get_accumulated_value();
                            back_propagation(tree, &path, get_accumulated_value / visits as f32);
                        }
                        continue;
                    }
                    let expansion_policy =
                        EP::new(&new_state, game_cache, heuristic_cache, heuristic_config);
                    let new_node = PlainNode::new(new_state.clone(), expansion_policy);
                    let new_child_id = tree.add_child(current_id, mv, new_node);
                    transposition_table.insert(new_state, new_child_id);
                    new_children.push(new_child_id);
                }
                // take the first newly added child
                let Some(child_index) = new_children.first() else { continue; };
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
            if let Some(final_score) = G::evaluate(&current_state, game_cache) {
                break final_score;
            }

            if let Some(heuristic) = SP::should_cutoff(
                &current_state,
                depth,
                game_cache,
                heuristic_cache,
                Some(G::perspective_player()),
                mcts_config,
                heuristic_config,
            ) {
                break heuristic;
            }

            current_state = G::apply_move(
                &current_state,
                &G::available_moves(&current_state)
                    .choose(&mut rand::thread_rng())
                    .expect("No available moves"),
                game_cache,
            );
            depth += 1;
        };

        // back propagation
        back_propagation(tree, &path, simulation_result);
    }

    fn select_move(&self) -> &G::Move {
        let root_id = self
            .tree
            .root_id()
            .expect("Root node must be initialized before selecting a move");
        let (_, mv) = self
            .tree
            .get_children(root_id)
            .iter()
            .max_by_key(|&&(child_id, _)| self.tree.get_node(child_id).get_visits())
            .expect("could not find move id");
        mv
    }
}

fn back_propagation<G, H, A, T>(tree: &mut T, path: &[A::NodeID], result: f32)
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
