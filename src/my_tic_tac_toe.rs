// This is an example for usage of monte carlo tree search lib

pub mod mcts_tic_tac_toe;
use std::collections::HashSet;

use crate::my_map_3x3::{CellIndex3x3, MyMap3x3};

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
    pub fn evaluate(&self) -> Option<f32> {
        match self {
            TicTacToeStatus::Me => Some(1.0),
            TicTacToeStatus::Opp => Some(0.0),
            TicTacToeStatus::Tie => Some(0.5),
            TicTacToeStatus::Vacant => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct TicTacToeGameData {
    map: MyMap3x3<TicTacToeStatus>,
}

impl std::fmt::Display for TicTacToeGameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─┬─┬─┐")?;
        writeln!(
            f,
            "│{}│{}│{}│",
            self.map.get_cell(CellIndex3x3::TL),
            self.map.get_cell(CellIndex3x3::TM),
            self.map.get_cell(CellIndex3x3::TR)
        )?;
        writeln!(f, "├─┼─┼─┤")?;
        writeln!(
            f,
            "│{}│{}│{}│",
            self.map.get_cell(CellIndex3x3::ML),
            self.map.get_cell(CellIndex3x3::MM),
            self.map.get_cell(CellIndex3x3::MR)
        )?;
        writeln!(f, "├─┼─┼─┤")?;
        writeln!(
            f,
            "│{}│{}│{}│",
            self.map.get_cell(CellIndex3x3::BL),
            self.map.get_cell(CellIndex3x3::BM),
            self.map.get_cell(CellIndex3x3::BR)
        )?;
        write!(f, "└─┴─┴─┘")
    }
}

impl TicTacToeGameData {
    pub const SCORE_LINES: [[CellIndex3x3; 3]; 8] = [
        [CellIndex3x3::TL, CellIndex3x3::MM, CellIndex3x3::BR],
        [CellIndex3x3::TR, CellIndex3x3::MM, CellIndex3x3::BL],
        [CellIndex3x3::ML, CellIndex3x3::MM, CellIndex3x3::MR],
        [CellIndex3x3::TM, CellIndex3x3::MM, CellIndex3x3::BM],
        [CellIndex3x3::TL, CellIndex3x3::TM, CellIndex3x3::TR],
        [CellIndex3x3::BL, CellIndex3x3::BM, CellIndex3x3::BR],
        [CellIndex3x3::TL, CellIndex3x3::ML, CellIndex3x3::BL],
        [CellIndex3x3::TR, CellIndex3x3::MR, CellIndex3x3::BR],
    ];
    pub fn new() -> Self {
        TicTacToeGameData {
            map: MyMap3x3::init(TicTacToeStatus::Vacant),
        }
    }
    pub fn get_status(&self) -> TicTacToeStatus {
        // check score lines
        for score_line in Self::SCORE_LINES.iter() {
            match score_line
                .iter()
                .map(|cell| *self.map.get_cell(*cell) as i8)
                .sum()
            {
                3 => return TicTacToeStatus::Me,
                -3 => return TicTacToeStatus::Opp,
                _ => (),
            }
        }

        // check if tie
        if self.map.iterate().all(|(_, v)| v.is_not_vacant()) {
            return TicTacToeStatus::Tie;
        }

        // game is still running
        TicTacToeStatus::Vacant
    }

    pub fn get_status_increment(&self, cell: &CellIndex3x3) -> TicTacToeStatus {
        // check cell
        if self.get_cell_value(*cell).is_vacant() {
            return TicTacToeStatus::Vacant;
        }
        // check score lines, which contain cell
        for score_line in Self::SCORE_LINES.iter() {
            if !score_line.contains(cell) {
                continue;
            }
            match score_line
                .iter()
                .map(|cell| *self.map.get_cell(*cell) as i8)
                .sum()
            {
                3 => return TicTacToeStatus::Me,
                -3 => return TicTacToeStatus::Opp,
                _ => (),
            }
        }

        // check if tie
        if self.map.iterate().all(|(_, v)| v.is_not_vacant()) {
            return TicTacToeStatus::Tie;
        }

        // game is still running
        TicTacToeStatus::Vacant
    }
    pub fn set_cell_value(&mut self, cell: CellIndex3x3, value: TicTacToeStatus) {
        self.map.set_cell(cell, value);
    }
    pub fn set_all_cells(&mut self, value: TicTacToeStatus) {
        self.map.set_all_cells(value);
    }
    pub fn get_cell_value(&self, cell: CellIndex3x3) -> TicTacToeStatus {
        *self.map.get_cell(cell)
    }
    pub fn get_first_vacant_cell(&self) -> Option<(CellIndex3x3, &TicTacToeStatus)> {
        self.map.iterate().find(|(_, v)| v.is_vacant())
    }
    pub fn count_me_cells(&self) -> usize {
        self.map
            .iterate()
            .filter(|(_, v)| matches!(**v, TicTacToeStatus::Me))
            .count()
    }
    pub fn count_opp_cells(&self) -> usize {
        self.map
            .iterate()
            .filter(|(_, v)| matches!(**v, TicTacToeStatus::Opp))
            .count()
    }
    pub fn iter_map(&self) -> impl Iterator<Item = (CellIndex3x3, &TicTacToeStatus)> {
        self.map.iterate()
    }
    pub fn count_non_vacant_cells(&self) -> usize {
        self.map
            .iterate()
            .filter(|(_, v)| v.is_not_vacant())
            .count()
    }
    pub fn get_threats(&self) -> (usize, usize) {
        let mut me_threats: HashSet<CellIndex3x3> = HashSet::new();
        let mut opp_threats: HashSet<CellIndex3x3> = HashSet::new();

        for score_line in Self::SCORE_LINES.iter() {
            let (threat, vacant) = score_line.iter().fold(
                (0, CellIndex3x3::default()),
                |(mut threat, mut vacant), element| {
                    let cell_value = self.map.get_cell(*element);
                    if cell_value.is_vacant() {
                        vacant = *element;
                    }
                    threat += *cell_value as i8;
                    (threat, vacant)
                },
            );
            match threat {
                2 => {
                    me_threats.insert(vacant);
                }
                -2 => {
                    opp_threats.insert(vacant);
                }
                _ => (),
            }
        }
        (me_threats.len(), opp_threats.len())
    }
    pub fn get_meta_cell_threats(&self, cell: CellIndex3x3) -> (i8, i8, i8, i8) {
        if self.get_cell_value(cell).is_not_vacant() {
            return (0, 0, 0, 0);
        }

        let mut my_meta_threats = 0;
        let mut my_meta_small_threats = 0;
        let mut opp_meta_threats = 0;
        let mut opp_meta_small_threats = 0;

        for score_line in Self::SCORE_LINES.iter() {
            if !score_line.contains(&cell) {
                continue;
            }
            let threat: i8 = score_line
                .iter()
                .map(|&c| self.get_cell_value(c) as i8)
                .sum();

            match threat {
                2 => my_meta_threats += threat,
                1 => my_meta_small_threats += threat,
                -1 => opp_meta_small_threats -= threat,
                -2 => opp_meta_threats -= threat,
                _ => (),
            }
        }

        (my_meta_threats, my_meta_small_threats, opp_meta_threats, opp_meta_small_threats)
    }

    pub fn board_analysis(&self) -> BoardAnalysis {
        let status = self.get_status();
        let my_cells = self.count_me_cells() as f32;
        let opp_cells = self.count_opp_cells() as f32;
        let (my_threats, opp_threats) = self.get_threats();
        let mut meta_cell_threats = MyMap3x3::default();
        for cell in self
            .iter_map()
            .filter_map(|(c, v)| if v.is_vacant() { Some(c) } else { None })
        {
            let meta_cell_threat = self.get_meta_cell_threats(cell);
            meta_cell_threats.set_cell(cell, meta_cell_threat);
        }
        BoardAnalysis {
            status,
            my_cells,
            opp_cells,
            my_threats: my_threats as f32,
            opp_threats: opp_threats as f32,
            meta_cell_threats,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct BoardAnalysis {
    pub status: TicTacToeStatus,
    pub my_cells: f32,
    pub opp_cells: f32,
    pub my_threats: f32,
    pub opp_threats: f32,
    pub meta_cell_threats: MyMap3x3<(i8, i8, i8, i8)>,
}
