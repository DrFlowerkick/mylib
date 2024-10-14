// type definition and functions of mcts node

use super::{
    MonteCarloGameData,
    MonteCarloPlayerAction,
    MonteCarloGameDataUpdate,
    MonteCarloNodeType,
    MonteCarloPlayer,
    MonteCarloGameMode
};

#[derive(PartialEq, Clone, Copy)]
pub struct MonteCarloNode<
    G: MonteCarloGameData,
    A: MonteCarloPlayerAction,
    U: MonteCarloGameDataUpdate,
> {
    pub game_data: G,
    pub player_action: A,
    pub game_data_update: U,
    pub node_type: MonteCarloNodeType,
    pub next_node: MonteCarloNodeType,
    pub player: MonteCarloPlayer,
    pub game_turn: usize,
    pub heuristic: f32,
    pub alpha: f32,
    pub beta: f32,
    pub wins: f32,
    pub samples: f32,
    pub parent_samples: f32,
    pub exploitation_score: f32, // exploitation_score is needed to choose best action and to choose node to exploit
    pub exploration_score: f32,  // exploration_score is needed to identify nodes for exploration
    pub heuristic_score: f32,
    pub total_score: f32,
    pub pruned_node: bool,
    pub game_end_node: bool, // leave, at which the game ends
}

impl<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate>
    MonteCarloNode<G, A, U>
{
    pub fn new() -> Self {
        MonteCarloNode {
            game_data: G::default(),
            player_action: A::default(),
            game_data_update: U::default(),
            node_type: MonteCarloNodeType::ActionResult,
            next_node: MonteCarloNodeType::ActionResult,
            player: MonteCarloPlayer::Me,
            game_turn: 0,
            heuristic: 0.0,
            alpha: f32::INFINITY,
            beta: f32::NEG_INFINITY,
            wins: 0.0,
            samples: f32::NAN,
            parent_samples: 0.0,
            exploitation_score: 0.0,
            exploration_score: 0.0,
            heuristic_score: 0.0,
            total_score: 0.0,
            pruned_node: false,
            game_end_node: false,
        }
    }
    pub fn new_player_action_child(&self, player_action: A) -> Self {
        let mut new_child = Self::new();
        new_child.player_action = player_action;
        new_child.parent_samples = self.samples;
        new_child.game_turn = self.game_turn;
        new_child.player = self.player;
        new_child
    }
    pub fn new_game_data_update_child(&self, game_data_update: U) -> Self {
        let mut new_child = Self::new();
        new_child.game_data_update = game_data_update;
        new_child.parent_samples = self.samples;
        new_child.game_turn = self.game_turn;
        new_child.player = self.player;
        new_child.node_type = MonteCarloNodeType::GameDataUpdate;
        new_child
    }

    pub fn calc_heuristic(&mut self, use_heuristic_score: bool) {
        if use_heuristic_score {
            self.heuristic = self.game_data.calc_heuristic();
            match self.player {
                MonteCarloPlayer::Me => self.alpha = self.heuristic,
                MonteCarloPlayer::Opp => self.beta = self.heuristic,
            }
        }
    }
    pub fn calc_node_score(&mut self, parent_samples: f32, weighting_factor: f32) {
        if parent_samples != self.parent_samples {
            self.update_exploration_score(parent_samples, weighting_factor);
        }
        self.total_score = match self.player {
            MonteCarloPlayer::Me => {
                self.exploitation_score + self.exploration_score - self.heuristic_score
            }
            MonteCarloPlayer::Opp => {
                self.exploitation_score + self.exploration_score + self.heuristic_score
            }
        };
    }

    pub fn check_game_turn(&mut self, game_mode: MonteCarloGameMode) {
        match game_mode {
            MonteCarloGameMode::SameTurnParallel => {
                if self.player == MonteCarloPlayer::Opp {
                    self.game_turn += 1;
                }
            }
            MonteCarloGameMode::ByTurns => self.game_turn += 1,
        }
    }

    pub fn set_next_node(&mut self, force_update: bool) {
        if !self.game_end_node {
            self.next_node = if self.game_data.is_game_data_update_required(force_update) {
                MonteCarloNodeType::GameDataUpdate
            } else {
                MonteCarloNodeType::ActionResult
            };
        }
    }

    pub fn apply_action(
        &mut self,
        parent_game_data: &G,
        parent_action: &A,
        game_mode: MonteCarloGameMode,
        max_number_of_turns: usize,
        use_heuristic_score: bool,
    ) -> bool {
        // transfer game_data of parent
        self.game_data = *parent_game_data;
        self.samples = 0.0;
        // score_event depends on player action (e.g. scoring points) or end of game
        let mut score_event = self.apply_player_action();
        self.player = self.player.next_player();
        self.check_game_turn(game_mode);
        match game_mode {
            MonteCarloGameMode::SameTurnParallel => {
                if self.player == MonteCarloPlayer::Me {
                    // first check if game ends
                    if self.check_game_ending(max_number_of_turns) {
                        self.calc_heuristic(use_heuristic_score);
                        return true; // save time by skipping all next code, since this is a game_end_node
                    }
                    self.game_data
                        .simultaneous_player_actions_for_simultaneous_game_data_change(
                            parent_action,
                            &self.player_action,
                        );
                }
            }
            MonteCarloGameMode::ByTurns => {
                score_event = self.check_game_ending(max_number_of_turns) || score_event;
            }
        }
        if score_event {
            self.calc_heuristic(use_heuristic_score);
        }
        score_event && use_heuristic_score
    }

    pub fn apply_game_data_update(
        &mut self,
        parent_game_data: &G,
        check_update_consistency: bool,
    ) -> bool {
        // transfer game_data of parent
        self.game_data = *parent_game_data;
        self.samples = 0.0;
        // apply update
        self.game_data
            .apply_game_data_update(&self.game_data_update, check_update_consistency)
    }

    pub fn apply_player_action(&mut self) -> bool {
        match self.player {
            MonteCarloPlayer::Me => self.game_data.apply_my_action(&self.player_action),
            MonteCarloPlayer::Opp => self.game_data.apply_opp_action(&self.player_action),
        }
    }

    pub fn check_game_ending(&mut self, max_number_of_turns: usize) -> bool {
        self.game_end_node = self.game_turn == max_number_of_turns
            || self.game_data.check_game_ending(self.game_turn);
        self.game_end_node
    }

    pub fn calc_simulation_score(&self) -> f32 {
        match self.game_data.game_winner(self.game_turn) {
            Some(player) => match player {
                MonteCarloPlayer::Me => 1.0,
                MonteCarloPlayer::Opp => 0.0,
            },
            None => 0.5,
        }
    }

    pub fn score_simulation_result(
        &mut self,
        simulation_score: f32,
        samples: f32,
        use_heuristic_score: bool,
    ) {
        self.wins += simulation_score;
        self.samples += samples;
        self.exploitation_score = match self.player {
            MonteCarloPlayer::Me => 1.0 - self.wins / self.samples,
            MonteCarloPlayer::Opp => self.wins / self.samples,
        };
        if use_heuristic_score {
            self.heuristic_score = match self.player {
                MonteCarloPlayer::Me => {
                    if self.alpha.is_finite() {
                        self.alpha / self.samples
                    } else {
                        0.0
                    }
                }
                MonteCarloPlayer::Opp => {
                    if self.beta.is_finite() {
                        self.beta / self.samples
                    } else {
                        0.0
                    }
                }
            };
        }
    }

    pub fn update_exploration_score(&mut self, parent_samples: f32, weighting_factor: f32) {
        self.parent_samples = parent_samples;
        self.exploration_score =
            weighting_factor * (self.parent_samples.log10() / self.samples).sqrt();
    }

    pub fn update_consistent_node_during_init_phase(
        &mut self,
        current_game_state: &G,
        played_turns: usize,
        force_update: bool,
    ) -> bool {
        if !force_update
            && !self
                .game_data
                .check_consistency_of_game_data_during_init_root(current_game_state, played_turns)
        {
            return false;
        }

        self.game_data == *current_game_state
    }
}