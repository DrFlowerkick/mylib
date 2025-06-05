// trait definitions for mcts
// these traits have to implemented by game crate to properly use mcts

use rand::seq::SliceRandom;

pub trait MCTSPlayer: PartialEq {
    fn next(&self) -> Self;
}

pub trait MCTSConfig {
    fn exploration_constant(&self) -> f32;
    fn progressive_widening_constant(&self) -> f32;
    fn progressive_widening_exponent(&self) -> f32;
    fn early_cut_off_depth(&self) -> usize;
}

pub trait MCTSGame: Sized {
    type State: Clone + PartialEq;
    type Move;
    type Player: MCTSPlayer;
    type Cache: GameCache<Self::State, Self::Move>;
    type Config: MCTSConfig;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        game_cache: &mut Self::Cache,
    ) -> Self::State;
    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32>;
    fn current_player(state: &Self::State) -> Self::Player;
    fn last_player(state: &Self::State) -> Self::Player;
    fn perspective_player() -> Self::Player;
}

pub trait MCTSTree<G, N, EP, H>: Sized
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self;
    fn init_root(&mut self, root_value: N);
    fn set_root(&mut self, new_root_id: N::ID);
    fn root_id(&self) -> Option<N::ID>;
    fn get_node(&self, id: N::ID) -> anyhow::Result<&N>;
    fn get_node_mut(&mut self, id: N::ID) -> anyhow::Result<&mut N>;
    fn add_child(&mut self, parent_id: N::ID, child_value: N) -> anyhow::Result<N::ID>;
}

pub trait MCTSNode<G: MCTSGame, EP: ExpansionPolicy<G, H>, H: Heuristic<G>>
where
    G: MCTSGame,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    type ID: Copy + Eq + std::fmt::Debug;

    fn init_root_id() -> Self::ID;
    fn root_node(state: G::State, expansion_policy: EP) -> Self;
    fn new(state: G::State, mv: G::Move, expansion_policy: EP) -> Self;
    fn add_child(&mut self, child_id: Self::ID);
    fn get_children(&self) -> &[Self::ID];
    fn get_state(&self) -> &G::State;
    fn get_move(&self) -> Option<&G::Move> {
        None
    }
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &G::Config,
    ) -> f32;
    fn expansion_policy(&self) -> &EP;
    fn expansion_policy_mut(&mut self) -> &mut EP;
    fn expandable_moves(
        &mut self,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move>;
}

pub trait MCTSAlgo<G: MCTSGame> {
    fn set_root(&mut self, state: &G::State) -> anyhow::Result<bool>;
    fn iterate(&mut self) -> anyhow::Result<()>;
    fn select_move(&self) -> anyhow::Result<&G::Move>;
}

pub trait UCTPolicy<G: MCTSGame> {
    /// calculates the exploitation score from the view of the perspective player
    fn exploitation_score(
        accumulated_value: f32,
        visits: usize,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        let raw = accumulated_value / visits as f32;
        // this works only for 2 player games
        if last_player == perspective_player {
            raw
        } else {
            1.0 - raw
        }
    }

    /// calculates the exploration score with default of constant base_c
    fn exploration_score(visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32 {
        mcts_config.exploration_constant() * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

pub trait UTCCache<G: MCTSGame, UP: UCTPolicy<G>> {
    fn new() -> Self;

    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    );

    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32;

    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &G::Config);

    fn get_exploration(&self, visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32;
}

pub trait ExpansionPolicy<G: MCTSGame, H: Heuristic<G>> {
    fn new(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        heuristic_config: &H::Config,
    ) -> Self;
    fn should_expand(
        &self,
        _visits: usize,
        _num_parent_children: usize,
        _mcts_config: &G::Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        false
    }
    fn expandable_moves<'a>(
        &'a mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &'a G::State,
        _mcts_config: &G::Config,
        _heuristic_config: &H::Config,
    ) -> Box<dyn Iterator<Item = G::Move> + 'a> {
        G::available_moves(state)
    }
}

pub trait SimulationPolicy<G: MCTSGame, H: Heuristic<G>> {
    fn should_cutoff(
        _state: &G::State,
        _depth: usize,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
        _perspective_player: Option<G::Player>,
        _mcts_config: &G::Config,
        _heuristic_config: &H::Config,
    ) -> Option<f32> {
        None
    }
}

pub trait HeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32;
    fn progressive_widening_decay_rate(&self) -> f32;
    fn early_cut_off_upper_bound(&self) -> f32;
    fn early_cut_off_lower_bound(&self) -> f32;
}

pub trait Heuristic<G: MCTSGame> {
    type Cache: HeuristicCache<G::State, G::Move>;
    type Config: HeuristicConfig;

    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        perspective_player: Option<G::Player>,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn evaluate_move(
        state: &G::State,
        mv: &G::Move,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn sort_moves(
        state: &G::State,
        moves: Vec<G::Move>,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> Vec<(f32, G::Move)> {
        let mut heuristic_moves = moves
            .into_iter()
            .map(|mv| {
                (
                    Self::evaluate_move(state, &mv, game_cache, heuristic_cache, heuristic_config),
                    mv,
                )
            })
            .collect::<Vec<_>>();
        heuristic_moves.shuffle(&mut rand::thread_rng());
        heuristic_moves
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        heuristic_moves
    }
}

pub trait RecursiveHeuristicConfig: HeuristicConfig {
    fn max_depth(&self) -> usize;
    fn alpha(&self) -> f32;
    fn alpha_reduction_factor(&self) -> f32;
    fn target_alpha(&self) -> f32;
    fn early_exit_threshold(&self) -> f32;
}

pub trait RecursiveHeuristic<G: MCTSGame>: Heuristic<G>
where
    <Self as Heuristic<G>>::Config: RecursiveHeuristicConfig,
{
    fn evaluate_state_recursive(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
        depth: usize,
        alpha: f32,
    ) -> f32 {
        let base_heuristic =
            Self::evaluate_state(state, game_cache, heuristic_cache, None, heuristic_config);

        if depth == 0 || G::evaluate(state, game_cache).is_some() {
            return base_heuristic;
        }

        let mut worst_response = f32::NEG_INFINITY;
        let next_player_alpha = alpha
            - (alpha - heuristic_config.target_alpha()) * heuristic_config.alpha_reduction_factor();
        // If no constraint on next move, this will be many moves to consider.
        // Therefore we use early exit to reduce calculation time.
        for next_player_move in G::available_moves(state) {
            let next_player_state = G::apply_move(state, &next_player_move, game_cache);

            let response_value = Self::evaluate_state_recursive(
                &next_player_state,
                game_cache,
                heuristic_cache,
                heuristic_config,
                depth - 1,
                next_player_alpha,
            );

            if response_value > worst_response {
                worst_response = response_value;
                // early exit, because next player does have guaranteed win
                if worst_response >= heuristic_config.early_exit_threshold() {
                    break;
                }
            }
        }

        // combine base heuristic with worst case response
        alpha * base_heuristic + (1.0 - alpha) * (1.0 - worst_response)
    }
}

pub trait GameCache<State, Move> {
    fn new() -> Self;
    fn get_applied_state(&self, _state: &State, _mv: &Move) -> Option<&State> {
        None
    }
    fn insert_applied_state(&mut self, _state: &State, _mv: &Move, _result: State) {}
    fn get_terminal_value(&self, _state: &State) -> Option<&Option<f32>> {
        None
    }
    fn insert_terminal_value(&mut self, _state: &State, _value: Option<f32>) {}
}

pub trait HeuristicCache<State, Move> {
    fn new() -> Self;
    fn get_intermediate_score(&self, _state: &State) -> Option<f32> {
        None
    }
    fn insert_intermediate_score(&mut self, _state: &State, _score: f32) {}
    fn get_move_score(&self, _state: &State, _mv: &Move) -> Option<f32> {
        None
    }
    fn insert_move_score(&mut self, _state: &State, _mv: &Move, _score: f32) {}
}
