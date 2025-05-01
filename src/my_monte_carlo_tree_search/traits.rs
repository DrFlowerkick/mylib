// trait definitions for mcts
// these traits have to implemented by game crate to properly use mcts

use super::MonteCarloPlayer;
use std::any::Any;
use std::hash::Hash;

// Trait for actions players can take to interact with game data.
pub trait MonteCarloPlayerAction: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(player_action: &impl MonteCarloPlayerAction) -> &Self;
    fn iter_actions(
        game_data: &impl MonteCarloGameData,
        player: MonteCarloPlayer,
        parent_game_turn: usize,
    ) -> Box<dyn Iterator<Item = Self> + '_>;
}

// Trait for updating game data after modifications through players. Normally there as some kind of random factor involved, e.g. drawing new resources of several kind from a "bag".
pub trait MonteCarloGameDataUpdate: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(game_data_update: &impl MonteCarloGameDataUpdate) -> &Self;
    fn iter_game_data_updates(
        game_data: &impl MonteCarloGameData,
        force_update: bool,
    ) -> Box<dyn Iterator<Item = Self> + '_>;
}

// trait for game data, which works with Monte Carlo Tree Search
pub trait MonteCarloGameData: Copy + Clone + PartialEq + Eq + Hash + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(game_data: &impl MonteCarloGameData) -> &Self;
    fn apply_my_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool; // true if score event, which results in change of heuristic
    fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool; // true if score event, which results in change of heuristic
    fn simultaneous_player_actions_for_simultaneous_game_data_change(
        &mut self,
        my_action: &impl MonteCarloPlayerAction,
        opp_action: &impl MonteCarloPlayerAction,
    );
    fn is_game_data_update_required(&self, force_update: bool) -> bool;
    fn apply_game_data_update(
        &mut self,
        game_data_update: &impl MonteCarloGameDataUpdate,
        check_update_consistency: bool,
    ) -> bool; // true if consistent
    fn calc_heuristic(&self) -> f32;
    fn check_game_ending(&self, game_turn: usize) -> bool;
    fn game_winner(&self, game_turn: usize) -> Option<MonteCarloPlayer>; // None if tie
    fn check_consistency_of_game_data_during_init_root(
        &mut self,
        current_game_state: &Self,
        played_turns: usize,
    ) -> bool;
    fn check_consistency_of_game_data_update(
        &mut self,
        current_game_state: &Self,
        game_data_update: &impl MonteCarloGameDataUpdate,
        played_turns: usize,
    ) -> bool;
    fn check_consistency_of_action_result(
        &mut self,
        current_game_state: Self,
        my_action: &impl MonteCarloPlayerAction,
        opp_action: &impl MonteCarloPlayerAction,
        played_turns: usize,
        apply_player_actions_to_game_data: bool,
    ) -> bool;
}

// new traits for MCTS. Will replace old traits in the future

pub trait MCTSPlayer: PartialEq {
    fn next(&self) -> Self;
}

pub trait MCTSGame {
    type State: Clone + PartialEq;
    type Move;
    type Player: MCTSPlayer;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(state: &Self::State, mv: &Self::Move) -> Self::State;
    fn is_terminal(state: &Self::State) -> bool;
    fn evaluate(state: &Self::State) -> f32;
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
    fn add_simulation_result(&mut self, result: f32);
    fn increment_visits(&mut self);
}

pub trait MCTSAlgo<G: MCTSGame> {
    fn iterate(&mut self);
    fn set_root(&mut self, state: &G::State) -> bool;
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

pub trait MCTSCache<G: MCTSGame, P: UCTPolicy<G>> {
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

pub trait ExpansionPolicy<G: MCTSGame> {
    fn new(state: &G::State) -> Self;
    fn should_expand(&self, visits: usize, num_parent_children: usize) -> bool;
    fn pop_expandable_move(&mut self, visits: usize, num_parent_children: usize)
        -> Option<G::Move>;
}
