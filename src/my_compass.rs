#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Compass {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
    Center,
}

impl Default for Compass {
    fn default() -> Self {
        Compass::Center
    }
}

impl From<(i32, i32)> for Compass {
    fn from(value: (i32, i32)) -> Self {
        match value {
            (0, 0) => Compass::Center,
            (0, -1) => Compass::N,
            (1, -1) => Compass::NE,
            (1, 0) => Compass::E,
            (1, 1) => Compass::SE,
            (0, 1) => Compass::S,
            (-1, 1) => Compass::SW,
            (-1, 0) => Compass::W,
            (-1, -1) => Compass::NW,
            _ => panic!("bad compass tuple"),
        }
    }
}

impl Compass {
    pub fn flip(&self) -> Self {
        match self {
            Compass::N => Compass::S,
            Compass::NE => Compass::SW,
            Compass::E => Compass::W,
            Compass::SE => Compass::NW,
            Compass::S => Compass::N,
            Compass::SW => Compass::NE,
            Compass::W => Compass::E,
            Compass::NW => Compass::SE,
            Compass::Center => Compass::Center,
        }
    }
    pub fn clockwise(&self) -> Self {
        match self {
            Compass::N => Compass::NE,
            Compass::NE => Compass::E,
            Compass::E => Compass::SE,
            Compass::SE => Compass::S,
            Compass::S => Compass::SW,
            Compass::SW => Compass::W,
            Compass::W => Compass::NW,
            Compass::NW => Compass::N,
            Compass::Center => Compass::Center,
        }
    }
    pub fn counterclockwise(&self) -> Self {
        match self {
            Compass::N => Compass::NW,
            Compass::NW => Compass::W,
            Compass::W => Compass::SW,
            Compass::SW => Compass::S,
            Compass::S => Compass::SE,
            Compass::SE => Compass::E,
            Compass::E => Compass::NE,
            Compass::NE => Compass::N,
            Compass::Center => Compass::Center,
        }
    }
    pub fn is_cardinal(&self) -> bool {
        match self {
            Compass::N | Compass::E | Compass::S | Compass::W => true,
            _ => false,
        }
    }
    pub fn is_ordinal(&self) -> bool {
        match self {
            Compass::NE | Compass::SE | Compass::SW | Compass::NW => true,
            _ => false,
        }
    }
    pub fn is_center(&self) -> bool {
        *self == Compass::Center
    }
}
