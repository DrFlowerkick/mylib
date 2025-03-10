// This is an example for usage of monte carlo tree search lib

pub mod mcts_tic_tac_toe;

use crate::my_map_point::*;
use crate::my_map_two_dim::*;
use crate::my_monte_carlo_tree_search::{
    MonteCarloGameData, MonteCarloGameDataUpdate, MonteCarloPlayer, MonteCarloPlayerAction,
};
pub const X: usize = 3;
pub const Y: usize = X;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum TicTacToeStatus {
    #[default]
    Vacant,
    Player(MonteCarloPlayer),
    Tie,
}
impl std::fmt::Display for TicTacToeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicTacToeStatus::Vacant => write!(f, " "),
            TicTacToeStatus::Tie => write!(f, "T"),
            TicTacToeStatus::Player(p) => match p {
                MonteCarloPlayer::Me => write!(f, "X"),
                MonteCarloPlayer::Opp => write!(f, "O"),
            },
        }
    }
}

impl TicTacToeStatus {
    pub fn is_vacant(&self) -> bool {
        *self == Self::Vacant
    }
    pub fn is_not_vacant(&self) -> bool {
        *self != Self::Vacant
    }
    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player(_))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct TicTacToeGameData {
    map: MyMap2D<TicTacToeStatus, X, Y>,
    status: TicTacToeStatus,
    num_me_cells: u8,
    num_opp_cells: u8,
    // required for Ultimate TicTacToe
    num_tie_cells: u8,
}

impl std::fmt::Display for TicTacToeGameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─┬─┬─┐")?;
        writeln!(
            f,
            "│{}│{}│{}│",
            self.map.get_row(0)[0],
            self.map.get_row(0)[1],
            self.map.get_row(0)[2]
        )?;
        writeln!(f, "├─┼─┼─┤")?;
        writeln!(
            f,
            "│{}│{}│{}│",
            self.map.get_row(1)[0],
            self.map.get_row(1)[1],
            self.map.get_row(1)[2]
        )?;
        writeln!(f, "├─┼─┼─┤")?;
        writeln!(
            f,
            "│{}│{}│{}│",
            self.map.get_row(2)[0],
            self.map.get_row(2)[1],
            self.map.get_row(2)[2]
        )?;
        write!(f, "└─┴─┴─┘")
    }
}

impl TicTacToeGameData {
    pub fn new() -> Self {
        TicTacToeGameData {
            map: MyMap2D::init(TicTacToeStatus::Vacant),
            status: TicTacToeStatus::Vacant,
            num_me_cells: 0,
            num_opp_cells: 0,
            num_tie_cells: 0,
        }
    }
    fn check_status_for_one_line<'a>(
        &self,
        line: impl Iterator<Item = &'a TicTacToeStatus>,
    ) -> TicTacToeStatus {
        let mut winner = TicTacToeStatus::Tie;
        for (index, element) in line.enumerate() {
            if index == 0 {
                match element {
                    TicTacToeStatus::Player(player) => winner = TicTacToeStatus::Player(*player),
                    _ => return TicTacToeStatus::Tie,
                }
            } else if winner != *element {
                return TicTacToeStatus::Tie;
            }
        }
        winner
    }
    fn check_status(&mut self, cell: MapPoint<X, Y>, check_lines: bool) -> TicTacToeStatus {
        if check_lines {
            // check row with cell.y()
            if let TicTacToeStatus::Player(player) =
                self.check_status_for_one_line(self.map.iter_row(cell.y()).map(|(_, v)| v))
            {
                self.status = TicTacToeStatus::Player(player);
                return self.status;
            }
            // check col with cell.x()
            if let TicTacToeStatus::Player(player) =
                self.check_status_for_one_line(self.map.iter_column(cell.x()).map(|(_, v)| v))
            {
                self.status = TicTacToeStatus::Player(player);
                return self.status;
            }
            // check neg diag, if cell.x() == cell.y()
            if cell.x() == cell.y() {
                if let TicTacToeStatus::Player(player) =
                    self.check_status_for_one_line(self.iter_diagonal_top_left())
                {
                    self.status = TicTacToeStatus::Player(player);
                    return self.status;
                }
            }
            // check pos diag, if cell.x() + cell.y() == 2
            if cell.x() + cell.y() == 2 {
                if let TicTacToeStatus::Player(player) =
                    self.check_status_for_one_line(self.iter_diagonal_top_right())
                {
                    self.status = TicTacToeStatus::Player(player);
                    return self.status;
                }
            }
        }
        // set to Tie, if no Vacant left
        if self.num_me_cells + self.num_opp_cells + self.num_tie_cells == 9 {
            self.status = TicTacToeStatus::Tie;
        }
        self.status
    }
    fn iter_diagonal_top_left(&self) -> impl Iterator<Item = &'_ TicTacToeStatus> {
        [(0_usize, 0_usize), (1, 1), (2, 2)]
            .into_iter()
            .map(|p| self.map.get(p.into()))
    }
    fn iter_diagonal_top_right(&self) -> impl Iterator<Item = &'_ TicTacToeStatus> {
        [(2_usize, 0_usize), (1, 1), (0, 2)]
            .into_iter()
            .map(|p| self.map.get(p.into()))
    }
    fn calc_line_heuristic<'a>(&self, line: impl Iterator<Item = &'a TicTacToeStatus>) -> f32 {
        let mut count: u8 = 0;
        let mut line_owner: Option<MonteCarloPlayer> = None;
        for cell in line {
            match cell {
                TicTacToeStatus::Vacant => (),
                TicTacToeStatus::Tie => return 0.0,
                TicTacToeStatus::Player(player) => match line_owner {
                    Some(owner) => {
                        if *player == owner {
                            count += 1;
                        } else {
                            return 0.0;
                        }
                    }
                    None => {
                        line_owner = Some(*player);
                        count += 1;
                    }
                },
            }
        }
        let line_heuristic = match count {
            1 => 1.0,
            2 => 10.0,
            _ => 100.0,
        };
        match line_owner {
            Some(player) => match player {
                MonteCarloPlayer::Me => line_heuristic,
                MonteCarloPlayer::Opp => -line_heuristic,
            },
            None => 0.0,
        }
    }
    pub fn calc_heuristic_(&self) -> f32 {
        let mut heuristic = 0.0;
        for rc in 0..3 {
            heuristic += self.calc_line_heuristic(self.map.iter_row(rc).map(|(_, v)| v));
            heuristic += self.calc_line_heuristic(self.map.iter_column(rc).map(|(_, v)| v));
        }
        heuristic += self.calc_line_heuristic(self.iter_diagonal_top_left());
        heuristic += self.calc_line_heuristic(self.iter_diagonal_top_right());
        heuristic
    }
    pub fn set_opp(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.set_player(cell, MonteCarloPlayer::Opp)
    }
    pub fn set_me(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.set_player(cell, MonteCarloPlayer::Me)
    }
    pub fn set_player(
        &mut self,
        cell: MapPoint<X, Y>,
        player: MonteCarloPlayer,
    ) -> TicTacToeStatus {
        let check_lines = match player {
            MonteCarloPlayer::Me => {
                self.num_me_cells += 1;
                self.num_me_cells > 2
            }
            MonteCarloPlayer::Opp => {
                self.num_opp_cells += 1;
                self.num_opp_cells > 2
            }
        };
        if self
            .map
            .swap_value(cell, TicTacToeStatus::Player(player))
            .is_not_vacant()
        {
            panic!("Set player on not vacant cell.");
        }
        self.check_status(cell, check_lines)
    }
    // required for Ultimate TicTacToe
    pub fn set_tie(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.num_tie_cells += 1;
        if self
            .map
            .swap_value(cell, TicTacToeStatus::Tie)
            .is_not_vacant()
        {
            panic!("Set tie on not vacant cell.");
        }
        self.check_status(cell, false)
    }
    pub fn set_all_to_status(&mut self) -> TicTacToeStatus {
        for (_, cell) in self.map.iter_mut() {
            *cell = self.status;
        }
        self.status
    }
    pub fn get_cell_value(&self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        *self.map.get(cell)
    }
    pub fn get_first_vacant_cell(&self) -> Option<(MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter().find(|(_, v)| v.is_vacant())
    }
    pub fn count_player_cells(&self, count_player: MonteCarloPlayer) -> usize {
        self.map
            .iter()
            .filter(|(_, v)| match v {
                TicTacToeStatus::Player(player) => *player == count_player,
                _ => false,
            })
            .count()
    }
    pub fn iter_map(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter()
    }
}
