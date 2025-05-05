// This is an example for usage of monte carlo tree search lib

pub mod mcts_tic_tac_toe;

use crate::my_map_point::*;
use crate::my_map_two_dim::*;
pub const X: usize = 3;
pub const Y: usize = X;

// TicTacToeStatus is used for cell and game status of TicTacToe
#[repr(i8)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum TicTacToeStatus {
    #[default]
    Vacant = 0,
    Me = 1,
    Opp = -1,
    Tie = 20,
}
impl std::fmt::Display for TicTacToeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicTacToeStatus::Vacant => write!(f, " "),
            TicTacToeStatus::Tie => write!(f, "T"),
            TicTacToeStatus::Me => write!(f, "X"),
            TicTacToeStatus::Opp => write!(f, "O"),
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
        matches!(self, Self::Me | Self::Opp)
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

        // check diagonals
        match self.get_status_for_one_line(self.iter_diagonal_top_left()) {
            TicTacToeStatus::Me => return TicTacToeStatus::Me,
            TicTacToeStatus::Opp => return TicTacToeStatus::Opp,
            _ => (),
        }
        match self.get_status_for_one_line(self.iter_diagonal_top_right()) {
            TicTacToeStatus::Me => return TicTacToeStatus::Me,
            TicTacToeStatus::Opp => return TicTacToeStatus::Opp,
            _ => (),
        }

        // check row and columns
        for rc in 0..3 {
            // check row with cell.y()
            match self.get_status_for_one_line(self.map.iter_row(rc).map(|(_, v)| v)) {
                TicTacToeStatus::Me => return TicTacToeStatus::Me,
                TicTacToeStatus::Opp => return TicTacToeStatus::Opp,
                _ => (),
            }
            // check col with cell.x()
            match self.get_status_for_one_line(self.map.iter_column(rc).map(|(_, v)| v)) {
                TicTacToeStatus::Me => return TicTacToeStatus::Me,
                TicTacToeStatus::Opp => return TicTacToeStatus::Opp,
                _ => (),
            }
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
                    TicTacToeStatus::Me => winner = TicTacToeStatus::Me,
                    TicTacToeStatus::Opp => winner = TicTacToeStatus::Opp,
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
    pub fn set_cell_value(&mut self, cell: MapPoint<X, Y>, value: TicTacToeStatus) {
        self
            .map
            .set(cell, value);
    }
    pub fn get_cell_value(&self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        *self.map.get(cell)
    }
    pub fn get_first_vacant_cell(&self) -> Option<(MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter().find(|(_, v)| v.is_vacant())
    }
    pub fn count_me_cells(&self) -> usize {
        self.map
            .iter()
            .filter(|(_, v)| matches!(**v, TicTacToeStatus::Me))
            .count()
    }
    pub fn count_opp_cells(&self) -> usize {
        self.map
            .iter()
            .filter(|(_, v)| matches!(**v, TicTacToeStatus::Opp))
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
                TicTacToeStatus::Me => me += 1,
                TicTacToeStatus::Opp => opp += 1,
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
