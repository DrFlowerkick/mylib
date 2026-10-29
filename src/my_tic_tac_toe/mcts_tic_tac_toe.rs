use super::*;
use crate::my_mcts::{GamePlayer, MCTSGame, WinDrawLoss};

#[derive(Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct TicTacToePlayerAction {
    pub cell: CellIndex3x3,
}

impl GamePlayer for TicTacToeStatus {
    fn next(&self) -> Self {
        match self {
            TicTacToeStatus::First => TicTacToeStatus::Second,
            TicTacToeStatus::Second => TicTacToeStatus::First,
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
            current_player: TicTacToeStatus::First,
            last_player: TicTacToeStatus::First,
        }
    }
    pub fn set_current_player(&mut self, player: TicTacToeStatus) {
        match player {
            TicTacToeStatus::First | TicTacToeStatus::Second => self.current_player = player,
            _ => panic!("Invalid player"),
        }
    }
    pub fn next_player(&mut self) {
        self.last_player = self.current_player;
        self.current_player = self.current_player.next();
    }
}

#[derive(Clone)]
pub struct TicTacToeMCTSGame {}

impl MCTSGame for TicTacToeMCTSGame {
    type State = TicTacToeGame;
    type Move = TicTacToePlayerAction;
    type Player = TicTacToeStatus;
    type Outcome = WinDrawLoss;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a> {
        Box::new(state.ttt.map.iterate().filter_map(|(cell, v)| {
            if v.is_vacant() {
                Some(TicTacToePlayerAction { cell })
            } else {
                None
            }
        }))
    }

    fn apply_move(state: &Self::State, mv: &Self::Move) -> Self::State {
        let mut new_state = *state;
        // apply the move for current player
        new_state.ttt.set_cell_value(mv.cell, state.current_player);
        // set the next player
        new_state.next_player();
        new_state
    }

    fn evaluate(state: &Self::State) -> Option<WinDrawLoss> {
        state.ttt.get_status().evaluate()
    }

    fn current_player(state: &Self::State) -> Self::Player {
        state.current_player
    }
    fn last_player(state: &Self::State) -> Self::Player {
        state.last_player
    }
    fn perspective_player() -> Self::Player {
        TicTacToeStatus::First
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{prelude::*, rng};

    use crate::my_mcts::{
        BaseConfig, DefaultSimulationPolicy, DynamicC, ExpandAll, GameOutcome,
        HybridWithScoreCache, MCTSAlgo, MostVisited, MostVisitedAndMaxMean, NoHeuristic, NoSolver,
        NoTranspositionTable, NodeOnly, PlainMCTS, PlainTree, ProgressiveWidening, SolverBounds,
        StaticC, TranspositionHashMap,
    };

    type TTTBasicMCTS = PlainMCTS<
        TicTacToeMCTSGame,
        NoHeuristic,
        BaseConfig<TicTacToeStatus>,
        StaticC,
        ExpandAll<TicTacToeMCTSGame>,
        DefaultSimulationPolicy,
        MostVisited,
        NoSolver,
    >;
    type TTTBasicMCTSTree =
        PlainTree<TicTacToeMCTSGame, NoHeuristic, TTTBasicMCTS, NodeOnly, NoTranspositionTable>;
    type TTTFullMCTS = PlainMCTS<
        TicTacToeMCTSGame,
        NoHeuristic,
        BaseConfig<TicTacToeStatus>,
        DynamicC,
        ProgressiveWidening<TicTacToeMCTSGame, BaseConfig<TicTacToeStatus>>,
        DefaultSimulationPolicy,
        MostVisitedAndMaxMean,
        SolverBounds,
    >;
    type TTTFullMCTSTree = PlainTree<
        TicTacToeMCTSGame,
        NoHeuristic,
        TTTFullMCTS,
        HybridWithScoreCache,
        TranspositionHashMap<TicTacToeGame, usize>,
    >;

    use std::time::{Duration, Instant};
    const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(200);
    const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(50);
    const EXPECTED_NUM_NODES: usize = 100_000;

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
            let mcts_tic_tac_toe: TTTBasicMCTS =
                PlainMCTS::new(BaseConfig::default(), NoHeuristic {});
            let mut mcts_tree_tic_tac_toe: TTTBasicMCTSTree = PlainTree::new(EXPECTED_NUM_NODES);
            let mut ttt_game_data = TicTacToeGame::new();
            ttt_game_data.set_current_player(TicTacToeStatus::First);
            let mut time_out = TIME_OUT_FIRST_TURN;

            while TicTacToeMCTSGame::evaluate(&ttt_game_data).is_none() {
                match ttt_game_data.current_player {
                    TicTacToeStatus::First => {
                        let start = Instant::now();
                        mcts_tic_tac_toe.set_root(&ttt_game_data, &mut mcts_tree_tic_tac_toe, 2);
                        while start.elapsed() < time_out {
                            mcts_tic_tac_toe.iterate(&mut mcts_tree_tic_tac_toe);
                        }
                        time_out = TIME_OUT_SUCCESSIVE_TURNS;
                        let selected_move = *mcts_tic_tac_toe.select_move(&mcts_tree_tic_tac_toe);
                        eprintln!("me : {}", selected_move.cell);
                        ttt_game_data =
                            TicTacToeMCTSGame::apply_move(&ttt_game_data, &selected_move);
                    }
                    TicTacToeStatus::Second => {
                        // let opp act by choosing a random action
                        let opp_move = TicTacToeMCTSGame::available_moves(&ttt_game_data)
                            .choose(&mut rng())
                            .expect("No available moves");
                        eprintln!("opp: {}", opp_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(&ttt_game_data, &opp_move);
                    }
                    _ => {
                        eprintln!("Invalid player");
                        panic!("Invalid player");
                    }
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", ttt_game_data.ttt);
            match ttt_game_data.ttt.get_status() {
                TicTacToeStatus::First => eprintln!("me winner"),
                TicTacToeStatus::Second => {
                    eprintln!("opp winner");
                    panic!("opp should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    panic!("vacant: Game ended without winner!?");
                }
            }
            wins += TicTacToeMCTSGame::evaluate(&ttt_game_data)
                .unwrap()
                .to_score();
        }
        println!("{} wins out of 50 matches.", wins);
        assert!(wins > 45.0);
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_starting_player_opp() {
        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mcts_tic_tac_toe: TTTBasicMCTS =
                PlainMCTS::new(BaseConfig::default(), NoHeuristic {});
            let mut mcts_tree_tic_tac_toe: TTTBasicMCTSTree = PlainTree::new(EXPECTED_NUM_NODES);
            let mut ttt_game_data = TicTacToeGame::new();
            ttt_game_data.set_current_player(TicTacToeStatus::First);
            let mut time_out = TIME_OUT_FIRST_TURN;

            while TicTacToeMCTSGame::evaluate(&ttt_game_data).is_none() {
                match ttt_game_data.current_player {
                    TicTacToeStatus::First => {
                        let start = Instant::now();
                        mcts_tic_tac_toe.set_root(&ttt_game_data, &mut mcts_tree_tic_tac_toe, 2);
                        while start.elapsed() < time_out {
                            mcts_tic_tac_toe.iterate(&mut mcts_tree_tic_tac_toe);
                        }
                        time_out = TIME_OUT_SUCCESSIVE_TURNS;
                        let selected_move = *mcts_tic_tac_toe.select_move(&mcts_tree_tic_tac_toe);
                        eprintln!("me : {}", selected_move.cell);
                        ttt_game_data =
                            TicTacToeMCTSGame::apply_move(&ttt_game_data, &selected_move);
                    }
                    TicTacToeStatus::Second => {
                        // let opp act by choosing a random action
                        let opp_move = TicTacToeMCTSGame::available_moves(&ttt_game_data)
                            .choose(&mut rng())
                            .expect("No available moves");
                        eprintln!("opp: {}", opp_move.cell);
                        ttt_game_data = TicTacToeMCTSGame::apply_move(&ttt_game_data, &opp_move);
                    }
                    _ => {
                        eprintln!("Invalid player");
                        panic!("Invalid player");
                    }
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", ttt_game_data.ttt);
            match ttt_game_data.ttt.get_status() {
                TicTacToeStatus::First => eprintln!("me winner"),
                TicTacToeStatus::Second => {
                    eprintln!("opp winner");
                    panic!("opp should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    panic!("vacant: Game ended without winner!?");
                }
            }
            wins += TicTacToeMCTSGame::evaluate(&ttt_game_data)
                .unwrap()
                .to_score();
        }
        println!("{} wins out of 50 matches.", wins);
        assert!(wins > 45.0);
    }

    #[test]
    fn test_new_mcts_traits_with_tic_tac_toe_versus_mcts() {
        let mut wins = 0.0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let first_mcts_tic_tac_toe: TTTBasicMCTS =
                PlainMCTS::new(BaseConfig::default(), NoHeuristic {});
            let mut first_mcts_tree_tic_tac_toe: TTTBasicMCTSTree =
                PlainTree::new(EXPECTED_NUM_NODES);
            let mut first_ttt_game_data = TicTacToeGame::new();
            first_ttt_game_data.set_current_player(TicTacToeStatus::First);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let second_mcts_tic_tac_toe: TTTFullMCTS =
                PlainMCTS::new(BaseConfig::default(), NoHeuristic {});
            let mut second_mcts_tree_tic_tac_toe: TTTFullMCTSTree =
                PlainTree::new(EXPECTED_NUM_NODES);
            let mut second_ttt_game_data = TicTacToeGame::new();
            second_ttt_game_data.set_current_player(TicTacToeStatus::Second);
            let mut second_time_out = TIME_OUT_FIRST_TURN;

            let mut first = true;
            while TicTacToeMCTSGame::evaluate(&first_ttt_game_data).is_none() {
                let mut iteration_counter: usize = 0;
                if first {
                    let start = Instant::now();
                    first_mcts_tic_tac_toe.set_root(
                        &first_ttt_game_data,
                        &mut first_mcts_tree_tic_tac_toe,
                        2,
                    );
                    while start.elapsed() < first_time_out {
                        first_mcts_tic_tac_toe.iterate(&mut first_mcts_tree_tic_tac_toe);
                        iteration_counter += 1;
                    }
                    first_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                    let selected_move =
                        *first_mcts_tic_tac_toe.select_move(&first_mcts_tree_tic_tac_toe);
                    eprintln!(
                        "first : {} (Iterations: {})",
                        selected_move.cell, iteration_counter
                    );
                    first_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&first_ttt_game_data, &selected_move);
                    second_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&second_ttt_game_data, &selected_move);
                    first = false;
                } else {
                    let start = Instant::now();
                    second_mcts_tic_tac_toe.set_root(
                        &second_ttt_game_data,
                        &mut second_mcts_tree_tic_tac_toe,
                        2,
                    );
                    while start.elapsed() < second_time_out {
                        second_mcts_tic_tac_toe.iterate(&mut second_mcts_tree_tic_tac_toe);
                        iteration_counter += 1;
                    }
                    second_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                    let selected_move =
                        *second_mcts_tic_tac_toe.select_move(&second_mcts_tree_tic_tac_toe);
                    eprintln!(
                        "second: {} (Iterations: {}",
                        selected_move.cell, iteration_counter
                    );
                    second_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&second_ttt_game_data, &selected_move);
                    first_ttt_game_data =
                        TicTacToeMCTSGame::apply_move(&first_ttt_game_data, &selected_move);
                    first = true;
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", first_ttt_game_data.ttt);
            match first_ttt_game_data.ttt.get_status() {
                TicTacToeStatus::First => {
                    eprintln!("first winner");
                    panic!("first should not win");
                }
                TicTacToeStatus::Second => {
                    eprintln!("second winner");
                    panic!("second should not win");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    panic!("vacant: Game ended without winner!?");
                }
            }
            wins += TicTacToeMCTSGame::evaluate(&first_ttt_game_data)
                .unwrap()
                .to_score();
        }
        println!("{} wins out of 50 matches.", wins);
        assert_eq!(wins, 25.0);
    }
}
