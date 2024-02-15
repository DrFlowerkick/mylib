use crate::my_geometry::my_point::Point;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Compass {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
    #[default]
    Center,
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

impl From<Compass> for (i32, i32) {
    fn from(value: Compass) -> Self {
        match value {
            Compass::Center => (0, 0),
            Compass::N => (0, -1),
            Compass::NE => (1, -1),
            Compass::E => (1, 0),
            Compass::SE => (1, 1),
            Compass::S => (0, 1),
            Compass::SW => (-1, 1),
            Compass::W => (-1, 0),
            Compass::NW => (-1, -1),
        }
    }
}

impl From<Point> for Compass {
    fn from(value: Point) -> Self {
        match (value.x, value.y) {
            (0, 0) => Compass::Center,
            (0, -1) => Compass::N,
            (1, -1) => Compass::NE,
            (1, 0) => Compass::E,
            (1, 1) => Compass::SE,
            (0, 1) => Compass::S,
            (-1, 1) => Compass::SW,
            (-1, 0) => Compass::W,
            (-1, -1) => Compass::NW,
            _ => panic!("bad compass point"),
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
    pub fn mirror_x_axis(&self) -> Self {
        match self {
            Compass::N => Compass::S,
            Compass::NE => Compass::SE,
            Compass::NW => Compass::SW,
            Compass::S => Compass::N,
            Compass::SE => Compass::NE,
            Compass::SW => Compass::NW,
            _ => *self,
        }
    }
    pub fn mirror_y_axis(&self) -> Self {
        match self {
            Compass::E => Compass::W,
            Compass::NE => Compass::NW,
            Compass::SE => Compass::SW,
            Compass::W => Compass::E,
            Compass::NW => Compass::NE,
            Compass::SW => Compass::SE,
            _ => *self,
        }
    }
    pub fn get_ordinals(&self) -> Option<[Compass; 2]> {
        match self {
            Compass::N => Some([Compass::NW, Compass::NE]),
            Compass::E => Some([Compass::NE, Compass::SE]),
            Compass::S => Some([Compass::SW, Compass::SE]),
            Compass::W => Some([Compass::NW, Compass::SW]),
            _ => None,
        }
    }
    pub fn get_cardinal(&self, other: &Self) -> Option<Compass> {
        match (self, other) {
            (Compass::NW, Compass::NE) | (Compass::NE, Compass::NW) => Some(Compass::N),
            (Compass::NE, Compass::SE) | (Compass::SE, Compass::NE) => Some(Compass::E),
            (Compass::SW, Compass::SE) | (Compass::SE, Compass::SW) => Some(Compass::S),
            (Compass::NW, Compass::SW) | (Compass::SW, Compass::NW) => Some(Compass::W),
            _ => None,
        }
    }
    pub fn is_cardinal(&self) -> bool {
        matches!(self, Compass::N | Compass::E | Compass::S | Compass::W)
    }
    pub fn is_ordinal(&self) -> bool {
        matches!(self, Compass::NE | Compass::SE | Compass::SW | Compass::NW)
    }
    pub fn is_center(&self) -> bool {
        *self == Compass::Center
    }
}
