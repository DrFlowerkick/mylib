// This is an example for usage of monte carlo tree search lib

pub mod mcts_tic_tac_toe;

use crate::my_map_point::*;
use crate::my_map_two_dim::*;
use crate::my_monte_carlo_tree_search::{MCTSPlayer, TwoPlayer};
pub const X: usize = 3;
pub const Y: usize = X;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum TicTacToeStatus {
    #[default]
    Vacant,
    Player(TwoPlayer),
    Tie,
}
impl std::fmt::Display for TicTacToeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicTacToeStatus::Vacant => write!(f, " "),
            TicTacToeStatus::Tie => write!(f, "T"),
            TicTacToeStatus::Player(p) => match p {
                TwoPlayer::Me => write!(f, "X"),
                TwoPlayer::Opp => write!(f, "O"),
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
        }
    }

    pub fn get_status(&self) -> TicTacToeStatus {
        // check if tie
        if self.map.iter().all(|(_, v)| v.is_not_vacant()) {
            return TicTacToeStatus::Tie;
        }
        // check row and columns
        for rc in 0..3 {
            // check row with cell.y()
            if let TicTacToeStatus::Player(player) =
                self.get_status_for_one_line(self.map.iter_row(rc).map(|(_, v)| v))
            {
                return TicTacToeStatus::Player(player);
            }
            // check col with cell.x()
            if let TicTacToeStatus::Player(player) =
                self.get_status_for_one_line(self.map.iter_column(rc).map(|(_, v)| v))
            {
                return TicTacToeStatus::Player(player);
            }
        }

        // check diagonals
        if let TicTacToeStatus::Player(player) =
            self.get_status_for_one_line(self.iter_diagonal_top_left())
        {
            return TicTacToeStatus::Player(player);
        }
        if let TicTacToeStatus::Player(player) =
            self.get_status_for_one_line(self.iter_diagonal_top_right())
        {
            return TicTacToeStatus::Player(player);
        }

        // game is still running
        TicTacToeStatus::Vacant
    }
    fn get_status_for_one_line<'a>(
        &self,
        line: impl Iterator<Item = &'a TicTacToeStatus>,
    ) -> TicTacToeStatus {
        let mut winner = TicTacToeStatus::Vacant;
        for (index, element) in line.enumerate() {
            if index == 0 {
                match element {
                    TicTacToeStatus::Player(player) => winner = TicTacToeStatus::Player(*player),
                    _ => return TicTacToeStatus::Vacant,
                }
            } else if winner != *element {
                return TicTacToeStatus::Vacant;
            }
        }
        winner
    }
    fn iter_diagonal_top_left(&self) -> impl Iterator<Item = &'_ TicTacToeStatus> {
        [(0_usize, 0_usize), (1, 1), (2, 2)]
            .iter()
            .map(move |p| self.map.get((*p).into()))
    }
    fn iter_diagonal_top_right(&self) -> impl Iterator<Item = &'_ TicTacToeStatus> {
        [(2_usize, 0_usize), (1, 1), (0, 2)]
            .iter()
            .map(move |p| self.map.get((*p).into()))
    }
    pub fn apply_player_move(
        &mut self,
        cell: MapPoint<X, Y>,
        player: TwoPlayer,
    ) {
        if self
            .map
            .swap_value(cell, TicTacToeStatus::Player(player))
            .is_not_vacant()
        {
            panic!("Set player on not vacant cell.");
        }
    }
    // required for Ultimate TicTacToe
    pub fn set_tie(&mut self, cell: MapPoint<X, Y>) {
        if self
            .map
            .swap_value(cell, TicTacToeStatus::Tie)
            .is_not_vacant()
        {
            panic!("Set tie on not vacant cell.");
        }
    }
    pub fn get_cell_value(&self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        *self.map.get(cell)
    }
    pub fn get_first_vacant_cell(&self) -> Option<(MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter().find(|(_, v)| v.is_vacant())
    }
    pub fn count_player_cells(&self, count_player: TwoPlayer) -> usize {
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
    pub fn count_non_vacant_cells(&self) -> usize {
        self.map.iter().filter(|(_, v)| v.is_not_vacant()).count()
    }
    fn check_threat_for_one_line<'a>(
        &self,
        my_threats: &mut u8,
        opp_threats: &mut u8,
        line: impl Iterator<Item = &'a TicTacToeStatus>,
    ) {
        let mut me: u8 = 0;
        let mut opp: u8 = 0_u8;
        let mut vacant: u8 = 0;
        for element in line {
            match element {
                TicTacToeStatus::Vacant => vacant += 1,
                TicTacToeStatus::Player(TwoPlayer::Me) => me += 1,
                TicTacToeStatus::Player(TwoPlayer::Opp) => opp += 1,
                TicTacToeStatus::Tie => return,
            }
            if (me > 0 && opp > 0) || vacant > 1 {
                return;
            }
        }
        match (me, opp, vacant) {
            (2, 0, 1) => *my_threats += 1,
            (0, 2, 1) => *opp_threats += 1,
            _ => (),
        }
    }
    pub fn get_threats(&self) -> (u8, u8) {
        let mut me_threat = 0;
        let mut opp_threat = 0;
        for rc in 0..3 {
            self.check_threat_for_one_line(
                &mut me_threat,
                &mut opp_threat,
                self.map.iter_row(rc).map(|(_, v)| v),
            );
            self.check_threat_for_one_line(
                &mut me_threat,
                &mut opp_threat,
                self.map.iter_column(rc).map(|(_, v)| v),
            );
        }
        self.check_threat_for_one_line(
            &mut me_threat,
            &mut opp_threat,
            self.iter_diagonal_top_left(),
        );
        self.check_threat_for_one_line(
            &mut me_threat,
            &mut opp_threat,
            self.iter_diagonal_top_right(),
        );
        (me_threat, opp_threat)
    }
}
