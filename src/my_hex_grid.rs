// implementation of hex coordinates using cube coordinates, see https://www.redblobgames.com/grids/hexagons/
// we use a Point3D to navigate with the constraint of x + y + z = 0 . This constraint results in a diagonal
// plane in 3D Space. This plane is our hex map.

use crate::my_geometry::my_point::Point3D;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug, Clone)]
pub struct HexGrid<T> {
    pub grid: HashMap<Point3D, T>,
}

impl<T> HexGrid<T> {
    pub fn new() -> Self {
        HexGrid {
            grid: HashMap::new(),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        HexGrid {
            grid: HashMap::with_capacity(capacity),
        }
    }
    pub fn with_capacity_from_radius(radius: usize) -> Self {
        let capacity = 1 + (1..=radius).map(|r| r * 6).sum::<usize>();
        HexGrid {
            grid: HashMap::with_capacity(capacity),
        }
    }
    pub fn get(&self, hex: &Point3D) -> Option<&T> {
        self.grid.get(hex)
    }
    pub fn get_mut(&mut self, hex: &Point3D) -> Option<&mut T> {
        self.grid.get_mut(hex)
    }
    pub fn set(&mut self, hex: Point3D, value: T) -> Option<&mut T> {
        if !hex.is_hex_coordinate() {
            return None;
        }
        match self.grid.entry(hex) {
            Entry::Vacant(ent) => Some(ent.insert(value)),
            Entry::Occupied(ent) => {
                let ent = ent.into_mut();
                *ent = value;
                Some(ent)
            }
        }
    }
}

// Hex Maps can be either "Pointy" or "Flat" top. Therefore we define a generic hex coordinates trait for handling
// hex coordinates and two hex orientation traits for orientation specific functions.

pub trait HexCoordinates {
    fn hex_distance(&self, hex_b: Self) -> i64;
    fn is_hex_coordinate(&self) -> bool;
}

pub trait PointyTopHex {
    const EAST: Self;
    const NORTH_EAST: Self;
    const NORTH_WEST: Self;
    const WEST: Self;
    const SOUTH_WEST: Self;
    const SOUTH_EAST: Self;
    fn iter_pth_neighbors(&self) -> Box<dyn Iterator<Item = Self>>;
}
pub trait FlatTopHex {
    const NORTH: Self;
    const WEST_NORTH: Self;
    const WEST_SOUTH: Self;
    const SOUTH: Self;
    const EAST_SOUTH: Self;
    const EAST_NORTH: Self;
    fn iter_fth_neighbors(&self) -> Box<dyn Iterator<Item = Self>>;
}

impl HexCoordinates for Point3D {
    fn hex_distance(&self, hex_b: Self) -> i64 {
        let delta = self.subtract(hex_b);
        delta.x.max(delta.y).max(delta.z)
    }
    fn is_hex_coordinate(&self) -> bool {
        self.x + self.y + self.z == 0
    }
}

impl PointyTopHex for Point3D {
    const EAST: Self = Point3D { x: 1, y: -1, z: 0 };
    const WEST: Self = Point3D { x: -1, y: 1, z: 0 };
    const NORTH_EAST: Self = Point3D { x: 1, y: 0, z: -1 };
    const SOUTH_WEST: Self = Point3D { x: -1, y: 0, z: 1 };
    const NORTH_WEST: Self = Point3D { x: 0, y: 1, z: -1 };
    const SOUTH_EAST: Self = Point3D { x: 0, y: -1, z: 1 };

    fn iter_pth_neighbors(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            [
                Self::EAST,
                Self::NORTH_EAST,
                Self::NORTH_WEST,
                Self::WEST,
                Self::SOUTH_WEST,
                Self::SOUTH_EAST,
            ]
            .into_iter(),
        )
    }
}

impl FlatTopHex for Point3D {
    const EAST_SOUTH: Self = Point3D { x: 1, y: -1, z: 0 };
    const WEST_NORTH: Self = Point3D { x: -1, y: 1, z: 0 };
    const EAST_NORTH: Self = Point3D { x: 1, y: 0, z: -1 };
    const WEST_SOUTH: Self = Point3D { x: -1, y: 0, z: 1 };
    const NORTH: Self = Point3D { x: 0, y: 1, z: -1 };
    const SOUTH: Self = Point3D { x: 0, y: -1, z: 1 };

    fn iter_fth_neighbors(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            [
                Self::NORTH,
                Self::WEST_NORTH,
                Self::WEST_SOUTH,
                Self::SOUTH,
                Self::EAST_SOUTH,
                Self::EAST_NORTH,
            ]
            .into_iter(),
        )
    }
}
