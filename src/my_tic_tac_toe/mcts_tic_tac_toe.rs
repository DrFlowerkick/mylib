use super::*;
use crate::my_monte_carlo_tree_search::{GameCache, MCTSGame, MCTSPlayer};
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct TicTacToePlayerAction {
    pub cell: CellIndex3x3,
}

impl MCTSPlayer for TicTacToeStatus {
    fn next(&self) -> Self {
        match self {
            TicTacToeStatus::Me => TicTacToeStatus::Opp,
            TicTacToeStatus::Opp => TicTacToeStatus::Me,
            _ => panic!("Invalid player"),
        }
    }
}

// TicTacToeGameData is used by UltTTT. Since I want to make memory usage as small as possible,
// I separate current_player from TicTacToeGameData.
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TicTacToeGame {
    pub ttt: TicTacToeGameData,
    pub current_player: TicTacToeStatus,
    pub last_player: TicTacToeStatus,
}

impl TicTacToeGame {
    pub fn new() -> Self {
        TicTacToeGame {
            ttt: TicTacToeGameData::new(),
            current_player: TicTacToeStatus::Me,
            last_player: TicTacToeStatus::Me,
        }
    }
    pub fn set_current_player(&mut self, player: TicTacToeStatus) {
        match player {
            TicTacToeStatus::Me | TicTacToeStatus::Opp => self.current_player = player,
            _ => panic!("Invalid player"),
        }
    }
    pub fn next_player(&mut self) {
        self.last_player = self.current_player;
        self.current_player = self.current_player.next();
    }
}

pub struct TicTacToeMCTSGame {}

impl MCTSGame for TicTacToeMCTSGame {
    type State = TicTacToeGame;
    type Move = TicTacToePlayerAction;
    type Player = TicTacToeStatus;
    type Cache = TicTacToeGameCache;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a> {
        Box::new(state.ttt.map.iterate().filter_map(|(cell, v)| {
            if v.is_vacant() {
                Some(TicTacToePlayerAction { cell })
            } else {
                None
            }
        }))
    }

    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        game_cache: &mut Self::Cache,
    ) -> Self::State {
        if let Some(cached_state) = game_cache.get_applied_state(state, mv) {
            return *cached_state;
        }
        let mut new_state = *state;
        // apply the move for current player
        new_state.ttt.set_cell_value(mv.cell, state.current_player);
        // set the next player
        new_state.next_player();
        // insert the new state into the cache
        game_cache.insert_applied_state(state, mv, new_state);
        new_state
    }

    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32> {
        if let Some(cached_value) = game_cache.get_terminal_value(state) {
            return *cached_value;
        }
        let evaluation = state.ttt.get_status().evaluate();
        game_cache.insert_terminal_value(state, evaluation);
        evaluation
    }

    fn current_player(state: &Self::State) -> Self::Player {
        state.current_player
    }
    fn last_player(state: &Self::State) -> Self::Player {
        state.last_player
    }
    fn perspective_player() -> Self::Player {
        TicTacToeStatus::Me
    }
}

pub struct TicTacToeGameCache {
    // No move cache, because calc of move is cheaper than caching
    pub state_cache: HashMap<TicTacToeGameData, Option<f32>>,
}

impl GameCache<TicTacToeGame, TicTacToePlayerAction> for TicTacToeGameCache {
    fn new() -> Self {
        TicTacToeGameCache {
            state_cache: HashMap::new(),
        }
    }
    fn get_terminal_value(&self, state: &TicTacToeGame) -> Option<&Option<f32>> {
        self.state_cache.get(&state.ttt)
    }
    fn insert_terminal_value(&mut self, state: &TicTacToeGame, value: Option<f32>) {
        self.state_cache.insert(state.ttt, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    use crate::my_monte_carlo_tree_search::{
        CachedUTC, DefaultSimulationPolicy, DynamicC, ExpandAll, MCTSAlgo, NoHeuristic, NoUTCCache,
        PWDefault, PlainMCTS, StaticC,
    };

    type PWDefaultTTT = PWDefault<TicTacToeMCTSGame>;
    type ExpandAllTTT = ExpandAll<TicTacToeMCTSGame>;

    use std::time::{Duration, Instant};
    const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(200);
    const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(50);
    const WEIGHTING_FACTOR: f32 = 1.40;

    #[test]
    fn calc_max_number_of_possible_nodes() {
        let mut nodes: u64 = 1;
        for i in 1..=9_u64 {
            nodes += (i..=9).product::<u64>();
        }
        assert_eq!(nodes, 986410);
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_starting_player_me() {
        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut mcts_tic_tac_toe: PlainMCTS<
                TicTacToeMCTSGame,
                StaticC,
                NoUTCCache,
                ExpandAllTTT,
                NoHeuristic,
                DefaultSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut ttt_game_data = TicTacToeGame::new();
            ttt_game_data.set_current_player(TicTacToeStatus::Me);
            let mut time_out = TIME_OUT_FIRST_TURN;

            while TicTacToeMCTSGame::evaluate(&ttt_game_data, &mut mcts_tic_tac_toe.game_cache)
                .is_none()
            {
                match ttt_game_data.current_player {
                    TicTacToeStatus::Me => {
                        let start = Instant::now();
                        mcts_tic_tac_toe.set_root(&ttt_game_data);
                        while start.elapsed() < time_out {
                            mcts_tic_tac_toe.iterate();
                        }
                        time_out = TIME_OUT_SUCCESSIVE_TURNS;
                        let selected_move = *mcts_tic_tac_toe.select_move();
                        eprintln!("me : {}", selected_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(
                            &ttt_game_data,
                            &selected_move,
                            &mut mcts_tic_tac_toe.game_cache,
                        );
                    }
                    TicTacToeStatus::Opp => {
                        // let opp act by choosing a random action
                        let opp_move = TicTacToeMCTSGame::available_moves(&ttt_game_data)
                            .choose(&mut thread_rng())
                            .expect("No available moves");
                        eprintln!("opp: {}", opp_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(
                            &ttt_game_data,
                            &opp_move,
                            &mut mcts_tic_tac_toe.game_cache,
                        );
                    }
                    _ => {
                        eprintln!("Invalid player");
                        assert!(false, "Invalid player");
                    }
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", ttt_game_data.ttt);
            match ttt_game_data.ttt.get_status() {
                TicTacToeStatus::Me => eprintln!("me winner"),
                TicTacToeStatus::Opp => {
                    eprintln!("opp winner");
                    assert!(false, "opp should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    assert!(false, "vacant: Game ended without winner!?");
                }
            }
            wins += TicTacToeMCTSGame::evaluate(&ttt_game_data, &mut mcts_tic_tac_toe.game_cache)
                .unwrap();
        }
        println!("{} wins out of 50 matches.", wins);
        assert!(wins > 45.0);
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_starting_player_opp() {
        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut mcts_tic_tac_toe: PlainMCTS<
                TicTacToeMCTSGame,
                StaticC,
                NoUTCCache,
                ExpandAllTTT,
                NoHeuristic,
                DefaultSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut ttt_game_data = TicTacToeGame::new();
            ttt_game_data.set_current_player(TicTacToeStatus::Me);
            let mut time_out = TIME_OUT_FIRST_TURN;

            while TicTacToeMCTSGame::evaluate(&ttt_game_data, &mut mcts_tic_tac_toe.game_cache)
                .is_none()
            {
                match ttt_game_data.current_player {
                    TicTacToeStatus::Me => {
                        let start = Instant::now();
                        mcts_tic_tac_toe.set_root(&ttt_game_data);
                        while start.elapsed() < time_out {
                            mcts_tic_tac_toe.iterate();
                        }
                        time_out = TIME_OUT_SUCCESSIVE_TURNS;
                        let selected_move = *mcts_tic_tac_toe.select_move();
                        eprintln!("me : {}", selected_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(
                            &ttt_game_data,
                            &selected_move,
                            &mut mcts_tic_tac_toe.game_cache,
                        );
                    }
                    TicTacToeStatus::Opp => {
                        // let opp act by choosing a random action
                        let opp_move = TicTacToeMCTSGame::available_moves(&ttt_game_data)
                            .choose(&mut thread_rng())
                            .expect("No available moves");
                        eprintln!("opp: {}", opp_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(
                            &ttt_game_data,
                            &opp_move,
                            &mut mcts_tic_tac_toe.game_cache,
                        );
                    }
                    _ => {
                        eprintln!("Invalid player");
                        assert!(false, "Invalid player");
                    }
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", ttt_game_data.ttt);
            match ttt_game_data.ttt.get_status() {
                TicTacToeStatus::Me => eprintln!("me winner"),
                TicTacToeStatus::Opp => {
                    eprintln!("opp winner");
                    assert!(false, "opp should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    assert!(false, "vacant: Game ended without winner!?");
                }
            }
            wins += TicTacToeMCTSGame::evaluate(&ttt_game_data, &mut mcts_tic_tac_toe.game_cache)
                .unwrap();
        }
        println!("{} wins out of 50 matches.", wins);
        assert!(wins > 45.0);
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_versus_mcts() {
        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut first_mcts_tic_tac_toe: PlainMCTS<
                TicTacToeMCTSGame,
                StaticC,
                NoUTCCache,
                ExpandAllTTT,
                NoHeuristic,
                DefaultSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut first_ttt_game_data = TicTacToeGame::new();
            first_ttt_game_data.set_current_player(TicTacToeStatus::Me);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let mut second_mcts_tic_tac_toe: PlainMCTS<
                TicTacToeMCTSGame,
                DynamicC,
                CachedUTC,
                PWDefaultTTT,
                NoHeuristic,
                DefaultSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut second_ttt_game_data = TicTacToeGame::new();
            second_ttt_game_data.set_current_player(TicTacToeStatus::Opp);
            let mut second_time_out = TIME_OUT_FIRST_TURN;

            let mut first = true;
            while TicTacToeMCTSGame::evaluate(
                &first_ttt_game_data,
                &mut first_mcts_tic_tac_toe.game_cache,
            )
            .is_none()
            {
                let mut iteration_counter: usize = 0;
                if first {
                    let start = Instant::now();
                    first_mcts_tic_tac_toe.set_root(&first_ttt_game_data);
                    while start.elapsed() < first_time_out {
                        first_mcts_tic_tac_toe.iterate();
                        iteration_counter += 1;
                    }
                    first_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                    let selected_move = *first_mcts_tic_tac_toe.select_move();
                    eprintln!(
                        "first : {} (Iterations: {})",
                        selected_move.cell, iteration_counter
                    );
                    first_ttt_game_data = TicTacToeMCTSGame::apply_move(
                        &first_ttt_game_data,
                        &selected_move,
                        &mut first_mcts_tic_tac_toe.game_cache,
                    );
                    second_ttt_game_data = TicTacToeMCTSGame::apply_move(
                        &second_ttt_game_data,
                        &selected_move,
                        &mut second_mcts_tic_tac_toe.game_cache,
                    );
                    first = false;
                } else {
                    let start = Instant::now();
                    second_mcts_tic_tac_toe.set_root(&second_ttt_game_data);
                    while start.elapsed() < second_time_out {
                        second_mcts_tic_tac_toe.iterate();
                        iteration_counter += 1;
                    }
                    second_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                    let selected_move = *second_mcts_tic_tac_toe.select_move();
                    eprintln!(
                        "second: {} (Iterations: {}",
                        selected_move.cell, iteration_counter
                    );
                    second_ttt_game_data = TicTacToeMCTSGame::apply_move(
                        &second_ttt_game_data,
                        &selected_move,
                        &mut second_mcts_tic_tac_toe.game_cache,
                    );
                    first_ttt_game_data = TicTacToeMCTSGame::apply_move(
                        &first_ttt_game_data,
                        &selected_move,
                        &mut first_mcts_tic_tac_toe.game_cache,
                    );
                    first = true;
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", first_ttt_game_data.ttt);
            match first_ttt_game_data.ttt.get_status() {
                TicTacToeStatus::Me => {
                    eprintln!("first winner");
                    assert!(false, "first should not win");
                }
                TicTacToeStatus::Opp => {
                    eprintln!("second winner");
                    assert!(false, "second should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    assert!(false, "vacant: Game ended without winner!?");
                }
            }
            wins += TicTacToeMCTSGame::evaluate(
                &first_ttt_game_data,
                &mut first_mcts_tic_tac_toe.game_cache,
            )
            .unwrap();
        }
        println!("{} wins out of 50 matches.", wins);
        assert_eq!(wins, 25.0);
    }
}
