// simple minimap, mainly used in TicTacToe

use std::convert::TryFrom;
use std::fmt::Display;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum CellIndex3x3 {
    #[default]
    TL = 0,
    TM = 1,
    TR = 2,
    ML = 3,
    MM = 4,
    MR = 5,
    BL = 6,
    BM = 7,
    BR = 8,
}

impl CellIndex3x3 {
    pub fn cell_weight(&self) -> f32 {
        match self {
            CellIndex3x3::MM => 4.0,
            CellIndex3x3::TL | CellIndex3x3::TR | CellIndex3x3::BL | CellIndex3x3::BR => 3.0,
            CellIndex3x3::TM | CellIndex3x3::ML | CellIndex3x3::MR | CellIndex3x3::BM => 2.0,
        }
    }
}

impl From<CellIndex3x3> for usize {
    fn from(cell: CellIndex3x3) -> Self {
        cell as usize
    }
}

impl TryFrom<usize> for CellIndex3x3 {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CellIndex3x3::TL),
            1 => Ok(CellIndex3x3::TM),
            2 => Ok(CellIndex3x3::TR),
            3 => Ok(CellIndex3x3::ML),
            4 => Ok(CellIndex3x3::MM),
            5 => Ok(CellIndex3x3::MR),
            6 => Ok(CellIndex3x3::BL),
            7 => Ok(CellIndex3x3::BM),
            8 => Ok(CellIndex3x3::BR),
            _ => Err(()),
        }
    }
}

impl TryFrom<(u8, u8)> for CellIndex3x3 {
    type Error = ();
    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        match value {
            (0, 0) => Ok(CellIndex3x3::TL),
            (0, 1) => Ok(CellIndex3x3::TM),
            (0, 2) => Ok(CellIndex3x3::TR),
            (1, 0) => Ok(CellIndex3x3::ML),
            (1, 1) => Ok(CellIndex3x3::MM),
            (1, 2) => Ok(CellIndex3x3::MR),
            (2, 0) => Ok(CellIndex3x3::BL),
            (2, 1) => Ok(CellIndex3x3::BM),
            (2, 2) => Ok(CellIndex3x3::BR),
            _ => Err(()),
        }
    }
}

impl From<CellIndex3x3> for (u8, u8) {
    fn from(cell: CellIndex3x3) -> Self {
        match cell {
            CellIndex3x3::TL => (0, 0),
            CellIndex3x3::TM => (0, 1),
            CellIndex3x3::TR => (0, 2),
            CellIndex3x3::ML => (1, 0),
            CellIndex3x3::MM => (1, 1),
            CellIndex3x3::MR => (1, 2),
            CellIndex3x3::BL => (2, 0),
            CellIndex3x3::BM => (2, 1),
            CellIndex3x3::BR => (2, 2),
        }
    }
}

impl Display for CellIndex3x3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellIndex3x3::TL => write!(f, "TL"),
            CellIndex3x3::TM => write!(f, "TM"),
            CellIndex3x3::TR => write!(f, "TR"),
            CellIndex3x3::ML => write!(f, "ML"),
            CellIndex3x3::MM => write!(f, "MM"),
            CellIndex3x3::MR => write!(f, "MR"),
            CellIndex3x3::BL => write!(f, "BL"),
            CellIndex3x3::BM => write!(f, "BM"),
            CellIndex3x3::BR => write!(f, "BR"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct MyMap3x3<T> {
    cells: [T; 9],
}

impl<T: Default + Clone + Copy> MyMap3x3<T> {
    pub fn new() -> Self {
        MyMap3x3 {
            cells: [T::default(); 9],
        }
    }
    pub fn init(value: T) -> Self {
        MyMap3x3 { cells: [value; 9] }
    }
    pub fn get_cell(&self, index: CellIndex3x3) -> &T {
        &self.cells[usize::from(index)]
    }
    pub fn get_cell_mut(&mut self, index: CellIndex3x3) -> &mut T {
        &mut self.cells[usize::from(index)]
    }
    pub fn set_cell(&mut self, index: CellIndex3x3, value: T) {
        self.cells[usize::from(index)] = value;
    }
    pub fn set_all_cells(&mut self, value: T) {
        self.cells = [value; 9];
    }
    pub fn iterate(&self) -> impl Iterator<Item = (CellIndex3x3, &T)> {
        self.cells
            .iter()
            .enumerate()
            .map(|(i, cell)| (CellIndex3x3::try_from(i).unwrap(), cell))
    }
}
