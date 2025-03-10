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
