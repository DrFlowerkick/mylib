// trait definitions for mcts
// these traits have to implemented by game crate to properly use mcts

pub trait MCTSPlayer: PartialEq {
    fn next(&self) -> Self;
}

pub trait MCTSGame: Sized {
    type State: Clone + PartialEq;
    type Move;
    type Player: MCTSPlayer;
    type Cache: GameCache<Self::State, Self::Move>;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        game_cache: &mut Self::Cache,
    ) -> Self::State;
    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32>;
    fn current_player(state: &Self::State) -> Self::Player;
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
    fn calc_utc(&mut self, parent_visits: usize, base_c: f32, perspective_player: G::Player)
        -> f32;
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
        current_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        let raw = accumulated_value / visits as f32;
        // this works only for 2 player games
        if current_player == perspective_player {
            1.0 - raw
        } else {
            raw
        }
    }

    /// calculates the exploration score with default of constant base_c
    fn exploration_score(visits: usize, parent_visits: usize, base_c: f32) -> f32 {
        base_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

pub trait UTCCache<G: MCTSGame, UP: UCTPolicy<G>> {
    fn new() -> Self;

    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        current_player: G::Player,
        perspective_player: G::Player,
    );

    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        current_player: G::Player,
        perspective_player: G::Player,
    ) -> f32;

    fn update_exploration(&mut self, visits: usize, parent_visits: usize, base_c: f32);

    fn get_exploration(&self, visits: usize, parent_visits: usize, base_c: f32) -> f32;
}

pub trait ExpansionPolicy<G: MCTSGame, H: Heuristic<G>> {
    fn new(state: &G::State, game_cache: &mut G::Cache, heuristic_cache: &mut H::Cache) -> Self;
    fn should_expand(&self, _visits: usize, _num_parent_children: usize) -> bool {
        false
    }
    fn expandable_moves<'a>(
        &'a mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &'a G::State,
    ) -> Box<dyn Iterator<Item = G::Move> + 'a> {
        G::available_moves(state)
    }
}

pub trait Heuristic<G: MCTSGame> {
    type Cache: HeuristicCache<G::State, G::Move>;

    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
    ) -> f32;
    fn evaluate_move(
        state: &G::State,
        mv: &G::Move,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
    ) -> f32;
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
    fn insert_intermediate_score(&mut self, _state: &State, _value: f32) {}
    fn get_move_score(&self, _state: &State, _mv: &Move) -> Option<f32> {
        None
    }
    fn insert_move_score(&mut self, _state: &State, _mv: &Move, _value: f32) {}
}

pub trait SimulationPolicy<G: MCTSGame, H: Heuristic<G>> {
    fn should_cutoff(
        _state: &G::State,
        _depth: usize,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
    ) -> Option<f32> {
        None
    }
}
