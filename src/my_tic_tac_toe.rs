// This is an example for usage of monte carlo tree search lib

pub mod mcts_tic_tac_toe;

use rand::prelude::*;
use rand::seq::IteratorRandom;
use crate::my_map_point::*;
use crate::my_map_two_dim::*;
use crate::my_monte_carlo_tree_search::*;
pub const X: usize = 3;
pub const Y: usize = X;
pub const N: usize = X * Y;


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TicTacToeStatus {
    Vacant,
    Player(MonteCarloPlayer),
    Tie,
}

impl TicTacToeStatus {
    pub fn is_vacant(&self) -> bool {
        *self == Self::Vacant
    }
    pub fn is_not_vacant(&self) -> bool {
        *self != Self::Vacant
    }
    pub fn is_player(&self) -> bool {
        match self {
            Self::Player(_) => true,
            _ => false,
        }
    }
}

impl Default for TicTacToeStatus {
    fn default() -> Self {
        TicTacToeStatus::Vacant
    }
}

#[derive(Copy, Clone, Default)]
pub struct TicTacToeGameData {
    map: MyMap2D<TicTacToeStatus, X, Y, N>,
    status: TicTacToeStatus,
}

impl PartialEq for TicTacToeGameData {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl TicTacToeGameData {
    pub fn new() -> Self {
        TicTacToeGameData {
            map: MyMap2D::init(TicTacToeStatus::Vacant),
            status: TicTacToeStatus::Vacant,
        }
    }
    fn check_status_for_one_line<'a>(&self, line: impl Iterator<Item = &'a TicTacToeStatus>) -> TicTacToeStatus {
        let mut winner = TicTacToeStatus::Tie;
        for (index, element) in line.enumerate() {
            if index == 0 {
                match element {
                    TicTacToeStatus::Player(player) => winner = TicTacToeStatus::Player(*player),
                    _ => return TicTacToeStatus::Tie,
                }
            } else if winner != *element {
                return TicTacToeStatus::Tie
            }
        }
        winner
    }
    fn check_status(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        // check row with cell.y()
        match self.check_status_for_one_line(self.map.iter_row(cell.y()).map(|(_, v)| v)) {
            TicTacToeStatus::Player(player) => {
                self.status = TicTacToeStatus::Player(player);
                return self.status
            },
            _ => (),
        }
        // check col with cell.x()
        match self.check_status_for_one_line(self.map.iter_column(cell.x()).map(|(_, v)| v)) {
            TicTacToeStatus::Player(player) => {
                self.status = TicTacToeStatus::Player(player);
                return self.status
            },
            _ => (),
        }
        // check neg diag, if cell.x() == cell.y()
        if cell.x() == cell.y() {
            match self.check_status_for_one_line(self.map.iter_diagonale_top_left().map(|(_, v)| v)) {
                TicTacToeStatus::Player(player) => {
                    self.status = TicTacToeStatus::Player(player);
                    return self.status
                },
                _ => (),
            }
            
        }
        // check pos diag, if cell.x() + cell.y() == 2
        if cell.x() + cell.y() == 2 {
            match self.check_status_for_one_line(self.map.iter_diagonale_top_right().map(|(_, v)| v)) {
                TicTacToeStatus::Player(player) => {
                    self.status = TicTacToeStatus::Player(player);
                    return self.status
                },
                _ => (),
            }
        }
        // set to Tie, if no Vacant left
        if self.map.iter().find(|(_, v)| v.is_vacant()).is_none() {
            self.status = TicTacToeStatus::Tie;
        }
        self.status
    }
    fn calc_line_heuristic<'a>(&self, line: impl Iterator<Item = &'a TicTacToeStatus>) -> f32 {
        let mut count: u8 = 0;
        let mut line_owner: Option<MonteCarloPlayer> = None;
        for cell in line {
            match cell {
                TicTacToeStatus::Vacant => (),
                TicTacToeStatus::Tie => return 0.0,
                TicTacToeStatus::Player(player) => match line_owner {
                    Some(owner) => if *player == owner {
                        count += 1;
                    } else {
                        return 0.0;
                    },
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
            None => 0.0
        }
    }
    pub fn calc_heuristic_(&self) -> f32 {
        let mut heuristic = 0.0;
        for rc in 0..3 {
            heuristic += self.calc_line_heuristic(self.map.iter_row(rc).map(|(_, v)| v));
            heuristic += self.calc_line_heuristic(self.map.iter_column(rc).map(|(_, v)| v));
        }
        heuristic += self.calc_line_heuristic(self.map.iter_diagonale_top_left().map(|(_, v)| v));
        heuristic += self.calc_line_heuristic(self.map.iter_diagonale_top_right().map(|(_, v)| v));
        heuristic
    }
    pub fn set_opp(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.set_player(cell, MonteCarloPlayer::Opp)
    }
    pub fn set_me(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.set_player(cell, MonteCarloPlayer::Me)
    }
    pub fn set_player(&mut self, cell: MapPoint<X, Y>, player: MonteCarloPlayer) -> TicTacToeStatus {
        self.map.set(cell, TicTacToeStatus::Player(player));
        self.check_status(cell)
    }
    pub fn set_vacant(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.map.set(cell, TicTacToeStatus::Vacant);
        self.status = TicTacToeStatus::Vacant;
        self.status
    }
    pub fn set_tie(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.map.set(cell, TicTacToeStatus::Tie);
        self.check_status(cell)
    }
    pub fn get_cell_value(&self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        *self.map.get(cell)
    }
    pub fn get_first_vacant_cell(&self) -> Option<(MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter().find(|(_, v)| v.is_vacant())
    }
    pub fn count_player_cells(&self, count_player: MonteCarloPlayer) -> usize {
        self.map.iter().filter(|(_, v)| match v {
            TicTacToeStatus::Player(player) => *player == count_player,
            _ => false,
        }).count()
    }
    pub fn iter_map(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter()
    }
}