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

pub trait MCTSNode<G: MCTSGame> {
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
}

pub trait MCTSAlgo<G: MCTSGame> {
    fn set_root(&mut self, state: &G::State) -> bool;
    fn iterate(&mut self);
    fn select_move(&self) -> &G::Move;
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
    ) -> bool {
        false
    }
    fn expandable_moves<'a>(
        &'a mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &'a G::State,
        _mcts_config: &G::Config,
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
    fn early_cut_off_upper_bound(&self) -> f32;
    fn early_cut_off_lower_bound(&self) -> f32;
    fn evaluate_state_recursive_depth(&self) -> usize;
    fn evaluate_state_recursive_alpha(&self) -> f32;
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
    fn evaluate_state_recursive(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
        _depth: usize,
        _alpha: f32,
    ) -> f32 {
        Self::evaluate_state(state, game_cache, heuristic_cache, None, heuristic_config)
    }
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
    ) -> Vec<G::Move> {
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
        heuristic_moves.into_iter().map(|(_, mv)| mv).collect()
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
