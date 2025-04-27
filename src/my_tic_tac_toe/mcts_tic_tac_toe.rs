// moving implementation of Monte Carlo Tree Search traits in separate module to prevent
// copying these implementations, if other crates uses my_tic_tac_toe functions and copies
// all dependencies in one file.

use super::*;
use crate::my_monte_carlo_tree_search::{
    MCTSGame, MCTSTurnBasedGame, MonteCarloGameData, MonteCarloGameDataUpdate, MonteCarloPlayer,
    MonteCarloPlayerAction,
};
use rand::prelude::*;
use rand::seq::IteratorRandom;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct TicTacToePlayerAction {
    pub cell: MapPoint<X, Y>,
}

impl TicTacToePlayerAction {
    pub fn execute_action(&self) {
        // in real player app use println!() instead of eprintln!()
        eprintln!("{} {}", self.cell.x(), self.cell.y());
    }
}

impl MonteCarloPlayerAction for TicTacToePlayerAction {
    fn downcast_self(player_action: &impl MonteCarloPlayerAction) -> &Self {
        match player_action.as_any().downcast_ref::<Self>() {
            Some(ttt_pa) => ttt_pa,
            None => panic!("player_action is not of type TicTacToePlayerAction!"),
        }
    }
    fn iter_actions(
        game_data: &impl MonteCarloGameData,
        _player: MonteCarloPlayer,
        _parent_game_turn: usize,
    ) -> Box<dyn Iterator<Item = Self> + '_> {
        let game_data = TicTacToeGameData::downcast_self(game_data);
        Box::new(IterTicTacToePlayerAction::new(game_data))
    }
}

struct IterTicTacToePlayerAction<'a> {
    ttt_data: &'a TicTacToeGameData,
    iter_action: TicTacToePlayerAction,
    iter_finished: bool,
}

impl<'a> IterTicTacToePlayerAction<'a> {
    fn new(ttt_data: &'a TicTacToeGameData) -> Self {
        let mut result = IterTicTacToePlayerAction {
            ttt_data,
            iter_action: TicTacToePlayerAction::default(),
            iter_finished: false,
        };
        match result.ttt_data.map.iter().find(|(_, v)| v.is_vacant()) {
            Some((start_point, _)) => result.iter_action.cell = start_point,
            None => result.iter_finished = true,
        };
        result
    }
}

impl Iterator for IterTicTacToePlayerAction<'_> {
    type Item = TicTacToePlayerAction;

    fn next(&mut self) -> Option<Self::Item> {
        // use iterator data
        if self.iter_finished {
            return None;
        }
        let result = self.iter_action;
        let mut searching_new_action = true;
        while searching_new_action {
            match self.iter_action.cell.forward_x() {
                Some(new_cell) => {
                    self.iter_action.cell = new_cell;
                    searching_new_action = self.ttt_data.map.get(new_cell).is_not_vacant();
                }
                None => {
                    self.iter_finished = true;
                    searching_new_action = false;
                }
            }
        }
        Some(result)
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
pub struct TicTacToeGameDataUpdate {}

impl MonteCarloGameDataUpdate for TicTacToeGameDataUpdate {
    fn downcast_self(_game_data_update: &impl MonteCarloGameDataUpdate) -> &Self {
        &TicTacToeGameDataUpdate {}
    }
    fn iter_game_data_updates(
        _game_data: &impl MonteCarloGameData,
        _force_update: bool,
    ) -> Box<dyn Iterator<Item = Self> + '_> {
        Box::new(vec![].into_iter())
    }
}

impl TicTacToeGameData {
    pub fn choose_random_next_action(&self) -> Option<TicTacToePlayerAction> {
        let mut rng = thread_rng();
        let iter_ttt = IterTicTacToePlayerAction::new(self);
        iter_ttt.choose(&mut rng)
    }
}

impl MonteCarloGameData for TicTacToeGameData {
    fn downcast_self(game_data: &impl MonteCarloGameData) -> &Self {
        match game_data.as_any().downcast_ref::<Self>() {
            Some(ttt_gd) => ttt_gd,
            None => panic!("game_data is not of type TicTacToeGameData!"),
        }
    }
    fn apply_my_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool {
        let player_action = TicTacToePlayerAction::downcast_self(player_action);
        self.set_player(player_action.cell, MonteCarloPlayer::Me);
        true
    }
    fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool {
        let player_action = TicTacToePlayerAction::downcast_self(player_action);
        self.set_player(player_action.cell, MonteCarloPlayer::Opp);
        true
    }
    fn simultaneous_player_actions_for_simultaneous_game_data_change(
        &mut self,
        _my_action: &impl MonteCarloPlayerAction,
        _opp_action: &impl MonteCarloPlayerAction,
    ) {
        // no random game_data updates for TicTacToe
    }
    fn is_game_data_update_required(&self, _force_update: bool) -> bool {
        false
    }
    fn apply_game_data_update(
        &mut self,
        _game_data_update: &impl MonteCarloGameDataUpdate,
        _check_update_consistency: bool,
    ) -> bool {
        true
    }
    fn calc_heuristic(&self) -> f32 {
        self.calc_heuristic_()
    }
    fn check_game_ending(&self, _game_turn: usize) -> bool {
        self.status.is_not_vacant()
    }
    fn game_winner(&self, _game_turn: usize) -> Option<MonteCarloPlayer> {
        match self.status {
            TicTacToeStatus::Player(player) => Some(player),
            _ => None,
        }
    }
    fn check_consistency_of_game_data_during_init_root(
        &mut self,
        _current_game_state: &Self,
        _played_turns: usize,
    ) -> bool {
        //dummy
        true
    }
    fn check_consistency_of_game_data_update(
        &mut self,
        _current_game_state: &Self,
        _game_data_update: &impl MonteCarloGameDataUpdate,
        _played_turns: usize,
    ) -> bool {
        //dummy
        true
    }
    fn check_consistency_of_action_result(
        &mut self,
        _current_game_state: Self,
        _my_action: &impl MonteCarloPlayerAction,
        _opp_action: &impl MonteCarloPlayerAction,
        _played_turns: usize,
        _apply_player_actions_to_game_data: bool,
    ) -> bool {
        //dummy
        true
    }
}

// solving TicTacToe with new MCTS traits
pub struct TicTacToeMCTSGame {}

impl MCTSTurnBasedGame for TicTacToeMCTSGame {
    fn current_player(state: &Self::State) -> MonteCarloPlayer {
        state.current_player
    }
}

impl MCTSGame for TicTacToeMCTSGame {
    type State = TicTacToeGameData;
    type Move = TicTacToePlayerAction;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a> {
        Box::new(IterTicTacToePlayerAction::new(state))
    }

    fn apply_move(state: &Self::State, mv: &Self::Move) -> Self::State {
        let mut new_state = state.clone();
        // apply the move for current player
        new_state.set_player(mv.cell, state.current_player);
        // set the next player
        new_state.next_player();
        new_state
    }

    fn evaluate(state: &Self::State) -> f32 {
        match state.status {
            TicTacToeStatus::Player(MonteCarloPlayer::Me) => 1.0,
            TicTacToeStatus::Player(MonteCarloPlayer::Opp) => 0.0,
            TicTacToeStatus::Tie => 0.5,
            TicTacToeStatus::Vacant => f32::NAN,
        }
    }

    fn is_terminal(state: &Self::State) -> bool {
        state.status.is_not_vacant()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::my_monte_carlo_tree_search::{MonteCarloGameMode, MonteCarloTreeSearch};

    use std::time::{Duration, Instant};
    const MAX_NUMBER_OF_TURNS: usize = 9;
    const FORCE_UPDATE: bool = true;
    const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(200);
    const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(50);
    const WEIGHTING_FACTOR: f32 = 1.40;
    const USE_CACHING: bool = true;
    const DEBUG: bool = true;

    #[test]
    fn calc_max_number_of_possible_nodes() {
        let mut nodes: u64 = 1;
        for i in 1..=9_u64 {
            nodes += (i..=9).product::<u64>();
        }
        assert_eq!(nodes, 986410);
    }

    // test typecasting
    #[test]
    fn ttt_type_casting() {
        let mut ttt_match = TicTacToeGameData::new();
        let player_action = TicTacToePlayerAction::default();
        ttt_match.apply_my_action(&player_action);
        assert_eq!(
            *ttt_match.map.get(MapPoint::<X, Y>::new(0, 0)),
            TicTacToeStatus::Player(MonteCarloPlayer::Me)
        );
        assert_eq!(IterTicTacToePlayerAction::new(&ttt_match).count(), 8);
    }

    // start a TicTacToe match with Me as StartPlayer
    #[test]
    fn ttt_me_start_player_test() {
        let use_heuristic_score = false;
        let mut last_winner: Option<MonteCarloPlayer> = None;
        let mut wins = 0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut ttt_match = TicTacToeGameData::new();
            let mut mcts_player: MonteCarloTreeSearch<
                TicTacToeGameData,
                TicTacToePlayerAction,
                TicTacToeGameDataUpdate,
            > = MonteCarloTreeSearch::new(
                MonteCarloGameMode::ByTurns,
                MAX_NUMBER_OF_TURNS,
                FORCE_UPDATE,
                TIME_OUT_FIRST_TURN,
                TIME_OUT_SUCCESSIVE_TURNS,
                WEIGHTING_FACTOR,
                use_heuristic_score,
                USE_CACHING,
                DEBUG,
            );
            while !ttt_match.check_game_ending(0) {
                let start = mcts_player.init_root(&ttt_match, MonteCarloPlayer::Me);
                mcts_player.expand_tree(start);
                eprint!("me  ");
                let (current_game_data, my_action) = mcts_player.choose_and_execute_actions();
                my_action.execute_action();
                ttt_match = current_game_data;
                if !ttt_match.check_game_ending(0) {
                    // let opp act by choosing a random action
                    match ttt_match.choose_random_next_action() {
                        Some(next_action) => {
                            eprint!("opp ");
                            next_action.execute_action();
                            ttt_match.apply_opp_action(&next_action);
                        }
                        None => (),
                    }
                }
            }
            last_winner = ttt_match.game_winner(0);
            match last_winner {
                Some(player) => match player {
                    MonteCarloPlayer::Me => {
                        wins += 1;
                        eprintln!("me winner ({})", wins);
                    }
                    MonteCarloPlayer::Opp => {
                        eprintln!("opp winner");
                        break;
                    }
                },
                None => eprintln!("tie"),
            }
        }
        assert_ne!(last_winner, Some(MonteCarloPlayer::Opp));
        assert!(wins > 45)
    }

    // start a TicTacToe match with Opp as StartPlayer
    #[test]
    fn ttt_opp_start_player_test() {
        let use_heuristic_score = false;
        let mut last_winner: Option<MonteCarloPlayer> = None;
        let mut wins = 0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut ttt_match = TicTacToeGameData::new();
            // let opp do 1. action by choosing a random action
            match ttt_match.choose_random_next_action() {
                Some(next_action) => {
                    eprint!("opp ");
                    next_action.execute_action();
                    ttt_match.apply_opp_action(&next_action);
                }
                None => (),
            }
            let mut mcts_player: MonteCarloTreeSearch<
                TicTacToeGameData,
                TicTacToePlayerAction,
                TicTacToeGameDataUpdate,
            > = MonteCarloTreeSearch::new(
                MonteCarloGameMode::ByTurns,
                MAX_NUMBER_OF_TURNS,
                FORCE_UPDATE,
                TIME_OUT_FIRST_TURN,
                TIME_OUT_SUCCESSIVE_TURNS,
                WEIGHTING_FACTOR,
                use_heuristic_score,
                USE_CACHING,
                DEBUG,
            );
            while !ttt_match.check_game_ending(0) {
                let start = mcts_player.init_root(&ttt_match, MonteCarloPlayer::Opp);
                mcts_player.expand_tree(start);
                eprint!("me  ");
                let (current_game_data, my_action) = mcts_player.choose_and_execute_actions();
                my_action.execute_action();
                ttt_match = current_game_data;
                if !ttt_match.check_game_ending(0) {
                    // let opp act by choosing a random action
                    match ttt_match.choose_random_next_action() {
                        Some(next_action) => {
                            eprint!("opp ");
                            next_action.execute_action();
                            ttt_match.apply_opp_action(&next_action);
                        }
                        None => (),
                    }
                }
            }
            last_winner = ttt_match.game_winner(0);
            match last_winner {
                Some(player) => match player {
                    MonteCarloPlayer::Me => {
                        wins += 1;
                        eprintln!("me winner ({})", wins);
                    }
                    MonteCarloPlayer::Opp => {
                        eprintln!("opp winner");
                        break;
                    }
                },
                None => eprintln!("tie"),
            }
        }
        assert_ne!(last_winner, Some(MonteCarloPlayer::Opp));
        assert!(wins > 40)
    }

    #[test]
    fn ttt_against_myself() {
        let use_heuristic_score = false;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut ttt_match_first = TicTacToeGameData::new();
            let mut ttt_match_second = TicTacToeGameData::new();

            let mut mcts_first: MonteCarloTreeSearch<
                TicTacToeGameData,
                TicTacToePlayerAction,
                TicTacToeGameDataUpdate,
            > = MonteCarloTreeSearch::new(
                MonteCarloGameMode::ByTurns,
                MAX_NUMBER_OF_TURNS,
                FORCE_UPDATE,
                TIME_OUT_FIRST_TURN,
                TIME_OUT_SUCCESSIVE_TURNS,
                WEIGHTING_FACTOR,
                use_heuristic_score,
                USE_CACHING,
                DEBUG,
            );
            let mut mcts_second: MonteCarloTreeSearch<
                TicTacToeGameData,
                TicTacToePlayerAction,
                TicTacToeGameDataUpdate,
            > = MonteCarloTreeSearch::new(
                MonteCarloGameMode::ByTurns,
                MAX_NUMBER_OF_TURNS,
                FORCE_UPDATE,
                TIME_OUT_FIRST_TURN,
                TIME_OUT_SUCCESSIVE_TURNS,
                WEIGHTING_FACTOR,
                use_heuristic_score,
                USE_CACHING,
                DEBUG,
            );
            let mut first = true;
            while !ttt_match_first.check_game_ending(0) {
                if first {
                    let start = mcts_first.init_root(&ttt_match_first, MonteCarloPlayer::Me);
                    mcts_first.expand_tree(start);
                    eprint!("first  ");
                    let (current_game_data, first_action) = mcts_first.choose_and_execute_actions();
                    ttt_match_first = current_game_data;
                    first_action.execute_action();
                    ttt_match_second.set_opp(first_action.cell);
                    first = false;
                } else {
                    let start = mcts_second.init_root(&ttt_match_second, MonteCarloPlayer::Opp);
                    mcts_second.expand_tree(start);
                    eprint!("second ");
                    let (current_game_data, second_action) =
                        mcts_second.choose_and_execute_actions();
                    ttt_match_second = current_game_data;
                    second_action.execute_action();
                    ttt_match_first.set_opp(second_action.cell);
                    first = true;
                }
            }
            let last_winner = ttt_match_first.game_winner(0);
            match last_winner {
                Some(player) => match player {
                    MonteCarloPlayer::Me => eprintln!("first winner"),
                    MonteCarloPlayer::Opp => eprintln!("second winner"),
                },
                None => eprintln!("tie"),
            }
            assert_eq!(last_winner, None);
        }
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_starting_player_me() {
        use crate::my_monte_carlo_tree_search::{MCTSAlgo, TurnBasedMCTS};

        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut mcts_tic_tac_toe: TurnBasedMCTS<TicTacToeMCTSGame> =
                TurnBasedMCTS::new(WEIGHTING_FACTOR);
            let mut ttt_game_data = TicTacToeGameData::new();
            ttt_game_data.set_current_player(MonteCarloPlayer::Me);
            let mut time_out = TIME_OUT_FIRST_TURN;

            while !TicTacToeMCTSGame::is_terminal(&ttt_game_data) {
                match ttt_game_data.current_player {
                    MonteCarloPlayer::Me => {
                        let start = Instant::now();
                        mcts_tic_tac_toe.set_root(&ttt_game_data);
                        while start.elapsed() < time_out {
                            mcts_tic_tac_toe.iterate();
                        }
                        time_out = TIME_OUT_SUCCESSIVE_TURNS;
                        let selected_move = mcts_tic_tac_toe.select_move();
                        eprintln!("me : {}", selected_move.cell);
                        ttt_game_data =
                            TicTacToeMCTSGame::apply_move(&ttt_game_data, selected_move);
                    }
                    MonteCarloPlayer::Opp => {
                        // let opp act by choosing a random action
                        let opp_move = TicTacToeMCTSGame::available_moves(&ttt_game_data)
                            .choose(&mut thread_rng())
                            .expect("No available moves");
                        eprintln!("opp: {}", opp_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(&ttt_game_data, &opp_move);
                    }
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", ttt_game_data);
            match ttt_game_data.status {
                TicTacToeStatus::Player(MonteCarloPlayer::Me) => eprintln!("me winner"),
                TicTacToeStatus::Player(MonteCarloPlayer::Opp) => {
                    eprintln!("opp winner");
                    assert!(false, "opp should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    assert!(false, "vacant: Game ended without winner!?");
                },
            }
            wins += TicTacToeMCTSGame::evaluate(&ttt_game_data);
        }
        println!("{} wins out of 50 matches.", wins);
        assert!(wins > 45.0);
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_starting_player_opp() {
        use crate::my_monte_carlo_tree_search::{MCTSAlgo, TurnBasedMCTS};

        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut mcts_tic_tac_toe: TurnBasedMCTS<TicTacToeMCTSGame> =
                TurnBasedMCTS::new(WEIGHTING_FACTOR);
            let mut ttt_game_data = TicTacToeGameData::new();
            ttt_game_data.set_current_player(MonteCarloPlayer::Opp);
            let mut time_out = TIME_OUT_FIRST_TURN;

            while !TicTacToeMCTSGame::is_terminal(&ttt_game_data) {
                match ttt_game_data.current_player {
                    MonteCarloPlayer::Me => {
                        let start = Instant::now();
                        mcts_tic_tac_toe.set_root(&ttt_game_data);
                        while start.elapsed() < time_out {
                            mcts_tic_tac_toe.iterate();
                        }
                        time_out = TIME_OUT_SUCCESSIVE_TURNS;
                        let selected_move = mcts_tic_tac_toe.select_move();
                        eprintln!("me : {}", selected_move.cell);
                        ttt_game_data =
                            TicTacToeMCTSGame::apply_move(&ttt_game_data, selected_move);
                    }
                    MonteCarloPlayer::Opp => {
                        // let opp act by choosing a random action
                        let opp_move = TicTacToeMCTSGame::available_moves(&ttt_game_data)
                            .choose(&mut thread_rng())
                            .expect("No available moves");
                        eprintln!("opp: {}", opp_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(&ttt_game_data, &opp_move);
                    }
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", ttt_game_data);
            match ttt_game_data.status {
                TicTacToeStatus::Player(MonteCarloPlayer::Me) => eprintln!("me winner"),
                TicTacToeStatus::Player(MonteCarloPlayer::Opp) => {
                    eprintln!("opp winner");
                    assert!(false, "opp should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    assert!(false, "vacant: Game ended without winner!?");
                },
            }
            wins += TicTacToeMCTSGame::evaluate(&ttt_game_data);
        }
        println!("{} wins out of 50 matches.", wins);
        assert!(wins > 45.0);
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_versus_mcts() {
        use crate::my_monte_carlo_tree_search::{MCTSAlgo, TurnBasedMCTS};

        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut first_mcts_tic_tac_toe: TurnBasedMCTS<TicTacToeMCTSGame> =
                TurnBasedMCTS::new(WEIGHTING_FACTOR);
            let mut first_ttt_game_data = TicTacToeGameData::new();
            first_ttt_game_data.set_current_player(MonteCarloPlayer::Me);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let mut second_mcts_tic_tac_toe: TurnBasedMCTS<TicTacToeMCTSGame> =
                TurnBasedMCTS::new(WEIGHTING_FACTOR);
            let mut second_ttt_game_data = TicTacToeGameData::new();
            second_ttt_game_data.set_current_player(MonteCarloPlayer::Opp);
            let mut second_time_out = TIME_OUT_FIRST_TURN;

            let mut first = true;

            while !TicTacToeMCTSGame::is_terminal(&first_ttt_game_data) {
                if first {
                    let start = Instant::now();
                    first_mcts_tic_tac_toe.set_root(&first_ttt_game_data);
                    while start.elapsed() < first_time_out {
                        first_mcts_tic_tac_toe.iterate();
                    }
                    first_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                    let selected_move = first_mcts_tic_tac_toe.select_move();
                    eprintln!("first : {}", selected_move.cell);
                    first_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&first_ttt_game_data, selected_move);
                    second_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&second_ttt_game_data, selected_move);
                    first = false;
                } else {
                    let start = Instant::now();
                    second_mcts_tic_tac_toe.set_root(&second_ttt_game_data);
                    while start.elapsed() < second_time_out {
                        second_mcts_tic_tac_toe.iterate();
                    }
                    second_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                    let selected_move = second_mcts_tic_tac_toe.select_move();
                    eprintln!("second: {}", selected_move.cell);
                    second_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&second_ttt_game_data, selected_move);
                    first_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&first_ttt_game_data, selected_move);
                    first = true;
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", first_ttt_game_data);
            match first_ttt_game_data.status {
                TicTacToeStatus::Player(MonteCarloPlayer::Me) => {
                    eprintln!("first winner");
                    assert!(false, "first should not win");
                },
                TicTacToeStatus::Player(MonteCarloPlayer::Opp) => {
                    eprintln!("second winner");
                    assert!(false, "second should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    assert!(false, "vacant: Game ended without winner!?");
                },
            }
            wins += TicTacToeMCTSGame::evaluate(&first_ttt_game_data);
        }
        println!("{} wins out of 50 matches.", wins);
        assert_eq!(wins, 25.0);
    }
}
