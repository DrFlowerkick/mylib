// Point is a simple two dimensional point with positive and negative x and y dimension.
// It can also be used as a vector, e.g. see offset().

use std::fmt::Display;

use crate::{my_compass::Compass, my_map_point::MapPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quadrant {
    // positive x and y
    First,
    // negative x, positive y
    Second,
    // negative x and y
    Third,
    // positive x, negative y
    Fourth,
    // on axis
    PositiveX,
    PositiveY,
    NegativeX,
    NegativeY,
    // both 0
    Origin,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Hash)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<(i64, i64)> for Point {
    fn from(value: (i64, i64)) -> Self {
        Point::new(value.0, value.1)
    }
}

// input: a,b ; e.g. -1,20
impl TryFrom<&str> for Point {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let error_message = "Format error. Expected input 'a,b' with a and b being integers".into();
        let Some((x, y)) = value.trim().split_once(',') else {
            return Err(error_message);
        };
        let Ok(x) = x.parse::<i64>() else {
            return Err(error_message);
        };
        let Ok(y) = y.parse::<i64>() else {
            return Err(error_message);
        };
        Ok(Point { x, y })
    }
}

impl From<Cylindrical> for Point {
    fn from(value: Cylindrical) -> Self {
        Point {
            x: (value.r * value.angle.to_radians().cos()) as i64,
            y: (value.r * value.angle.to_radians().sin()) as i64,
        }
    }
}

impl<const X: usize, const Y: usize> From<MapPoint<X, Y>> for Point {
    fn from(value: MapPoint<X, Y>) -> Self {
        Self {
            x: value.x() as i64,
            y: value.y() as i64,
        }
    }
}

impl From<Compass> for Point {
    fn from(value: Compass) -> Self {
        match value {
            Compass::Center => (0, 0).into(),
            Compass::N => (0, -1).into(),
            Compass::NE => (1, -1).into(),
            Compass::E => (1, 0).into(),
            Compass::SE => (1, 1).into(),
            Compass::S => (0, 1).into(),
            Compass::SW => (-1, 1).into(),
            Compass::W => (-1, 0).into(),
            Compass::NW => (-1, -1).into(),
        }
    }
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }
    pub fn switch_xy(&self) -> Point {
        Point {
            x: self.y,
            y: self.x,
        }
    }
    pub fn distance_x(&self, target: impl Into<Point>) -> i64 {
        self.x - target.into().x
    }
    pub fn delta_x(&self, target: impl Into<Point>) -> i64 {
        (self.x - target.into().x).abs()
    }
    pub fn distance_y(&self, target: impl Into<Point>) -> i64 {
        self.y - target.into().y
    }
    pub fn delta_y(&self, target: impl Into<Point>) -> i64 {
        (self.y - target.into().y).abs()
    }
    pub fn distance(&self, target: impl Into<Point>) -> f32 {
        let target = target.into();
        let result = self.distance_x(target).pow(2) + self.distance_y(target).pow(2);
        (result as f32).sqrt()
    }
    pub fn delta(&self, target: impl Into<Point>) -> i64 {
        let target = target.into();
        self.delta_x(target) + self.delta_y(target)
    }
    pub fn quadrant(&self) -> Quadrant {
        match (self.x, self.y) {
            (0, 0) => Quadrant::Origin,
            (1.., 0) => Quadrant::PositiveX,
            (_, 0) => Quadrant::NegativeX,
            (0, 1..) => Quadrant::PositiveY,
            (0, _) => Quadrant::NegativeY,
            (1.., 1..) => Quadrant::First,
            (_, 1..) => Quadrant::Second,
            (1.., _) => Quadrant::Fourth,
            (_, _) => Quadrant::Third,
        }
    }
    pub fn add(&self, offset: impl Into<Point>) -> Point {
        let offset = offset.into();
        Point {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }
    pub fn subtract(&self, offset: impl Into<Point>) -> Point {
        let offset = offset.into();
        Point {
            x: self.x - offset.x,
            y: self.y - offset.y,
        }
    }
    pub fn scale(&self, factor: i64) -> Point {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
    pub fn turn(&self, turn: Turns90, clockwise: bool) -> Point {
        match (turn, clockwise) {
            (Turns90::T0, _) => *self,
            (Turns90::T180, _) => Point::new(-self.x, -self.y),
            (Turns90::T90, true) | (Turns90::T270, false) => Point::new(self.y, -self.x),
            (Turns90::T270, true) | (Turns90::T90, false) => Point::new(-self.y, self.x),
        }
    }
    pub fn turn_around_point(&self, pos: impl Into<Point>, turn: Turns90, clockwise: bool) -> Point {
        let pos = pos.into();
        self.subtract(pos).turn(turn, clockwise).add(pos)
    }
    pub fn scale_toward_point_with_len(&self, target: impl Into<Point>, len: f32) -> Point {
        let target = target.into();
        let vector = Cylindrical::from(target.subtract(*self));
        self.add(vector.set_radius(len))
    }
    pub fn scale_toward_point_with_factor(&self, target: impl Into<Point>, factor: f32) -> Point {
        let target = target.into();
        let vector = Cylindrical::from(target.subtract(*self));
        self.add(vector.stretch(factor))
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct Cylindrical {
    r: f32,
    // in degree
    angle: f32,
}

impl From<Point> for Cylindrical {
    fn from(value: Point) -> Self {
        let r = value.distance(Point::new(0, 0));
        let angle = if (r as i64) == 0 {
            // 0, if zero len vector
            0.0
        } else {
            let alpha = ((value.x as f32) / r).acos().to_degrees();
            if value.y < 0 {
                360.0 - alpha
            } else {
                alpha
            }
        };
        Self { r, angle }
    }
}

impl Cylindrical {
    pub fn new(r: f32, angle: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..360.0).contains(&angle));
        Self { r, angle }
    }
    pub fn radius(&self) -> f32 {
        self.r
    }
    pub fn angle(&self) -> f32 {
        self.angle
    }
    pub fn set_radius(&self, r: f32) -> Self {
        assert!(r >= 0.0);
        Self {
            r,
            angle: self.angle,
        }
    }
    pub fn stretch(&self, factor: f32) -> Self {
        assert!(factor >= 0.0);
        Self {
            r: self.r * factor,
            angle: self.angle,
        }
    }
    pub fn set_angle(&self, angle: f32) -> Self {
        assert!((0.0..360.0).contains(&angle));
        Self { r: self.r, angle }
    }
    pub fn rotate(&self, rotation_angle: f32) -> Self {
        Self {
            r: self.r,
            angle: (self.angle + rotation_angle) % 360.0,
        }
    }
    pub fn quadrant(&self) -> Quadrant {
        if self.r == 0.0 {
            Quadrant::Origin
        } else if self.angle == 0.0 {
            Quadrant::PositiveX
        } else if self.angle > 0.0 && self.angle < 90.0 {
            Quadrant::First
        } else if self.angle == 90.0 {
            Quadrant::PositiveY
        } else if self.angle > 90.0 && self.angle < 180.0 {
            Quadrant::Second
        } else if self.angle == 180.0 {
            Quadrant::NegativeX
        } else if self.angle > 180.0 && self.angle < 270.0 {
            Quadrant::Third
        } else if self.angle == 270.0 {
            Quadrant::NegativeY
        } else {
            Quadrant::Fourth
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default, Hash)]
pub enum Turns90 {
    #[default]
    T0,
    T90,
    T180,
    T270,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default, Hash)]
pub struct Point3D {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<(i64, i64, i64)> for Point3D {
    fn from(value: (i64, i64, i64)) -> Self {
        Point3D::new(value.0, value.1, value.2)
    }
}

impl Point3D {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Point3D { x, y, z }
    }

    pub fn add(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        Point3D {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn subtract(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        Point3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn length(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2) + self.z.pow(2)) as f64).sqrt()
    }

    pub fn cross_product(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        Point3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub const fn rotate_x_turn90(&self, t: Turns90) -> Self {
        match t {
            Turns90::T0 => *self,
            Turns90::T90 => Point3D {
                x: self.x,
                y: -self.z,
                z: self.y,
            },
            Turns90::T180 => Point3D {
                x: self.x,
                y: -self.y,
                z: -self.z,
            },
            Turns90::T270 => Point3D {
                x: self.x,
                y: self.z,
                z: -self.y,
            },
        }
    }

    pub const fn rotate_y_turn90(&self, t: Turns90) -> Self {
        match t {
            Turns90::T0 => *self,
            Turns90::T90 => Point3D {
                x: self.z,
                y: self.y,
                z: -self.x,
            },
            Turns90::T180 => Point3D {
                x: -self.x,
                y: self.y,
                z: -self.z,
            },
            Turns90::T270 => Point3D {
                x: -self.z,
                y: self.y,
                z: self.x,
            },
        }
    }

    pub const fn rotate_z_turn90(&self, t: Turns90) -> Self {
        match t {
            Turns90::T0 => *self,
            Turns90::T90 => Point3D {
                x: -self.y,
                y: self.x,
                z: self.z,
            },
            Turns90::T180 => Point3D {
                x: -self.x,
                y: -self.y,
                z: self.z,
            },
            Turns90::T270 => Point3D {
                x: self.y,
                y: -self.x,
                z: self.z,
            },
        }
    }

    pub const fn all_unambiguous_rotation_combinations() -> [(Turns90, Turns90, Turns90); 24] {
        const POINT: Point3D = Point3D { x: 1, y: 2, z: 3 };
        const MAX_COMBINATIONS: usize = 24;
        let mut combinations: [(Turns90, Turns90, Turns90); MAX_COMBINATIONS] =
            [(Turns90::T0, Turns90::T0, Turns90::T0); MAX_COMBINATIONS];
        let mut rotation_results: [Point3D; MAX_COMBINATIONS] =
            [Point3D { x: 0, y: 0, z: 0 }; MAX_COMBINATIONS];
        let mut num_combinations = 0;
        const ROTATIONS: [Turns90; 4] = [Turns90::T0, Turns90::T90, Turns90::T180, Turns90::T270];
        let mut z = 0;
        while z < 4 {
            let rotate_z = ROTATIONS[z];
            let mut y = 0;
            while y < 4 {
                let rotate_y = ROTATIONS[y];
                let mut x = 0;
                while x < 4 {
                    let rotate_x = ROTATIONS[x];
                    let rotated = POINT
                        .rotate_x_turn90(rotate_x)
                        .rotate_y_turn90(rotate_y)
                        .rotate_z_turn90(rotate_z);
                    let mut unique = true;
                    let mut index = 0;
                    while index < num_combinations && unique {
                        unique = unique
                            && (rotated.x != rotation_results[index].x
                                || rotated.y != rotation_results[index].y
                                || rotated.z != rotation_results[index].z);
                        index += 1;
                    }
                    if unique {
                        rotation_results[num_combinations] = rotated;
                        combinations[num_combinations] = (rotate_x, rotate_y, rotate_z);
                        num_combinations += 1;
                    }
                    x += 1;
                }
                y += 1;
            }
            z += 1;
        }
        combinations
    }

    pub const fn apply_rotation_combination(
        &self,
        combination: (Turns90, Turns90, Turns90),
    ) -> Self {
        self.rotate_x_turn90(combination.0)
            .rotate_y_turn90(combination.1)
            .rotate_z_turn90(combination.2)
    }
    pub fn iter_cuboid(
        &self,
        dx_minus: i64,
        dx_plus: i64,
        dy_minus: i64,
        dy_plus: i64,
        dz_minus: i64,
        dz_plus: i64,
    ) -> impl Iterator<Item = Point3D> + '_ {
        (dx_minus <= 0
            && dx_plus >= 0
            && dy_minus <= 0
            && dy_plus >= 0
            && dz_minus <= 0
            && dz_plus >= 0)
            .then(|| {
                (dz_minus..=dz_plus).flat_map(move |dz| {
                    (dy_minus..=dy_plus).flat_map(move |dy| {
                        (dx_minus..=dx_plus).map(move |dx| self.add((dx, dy, dz)))
                    })
                })
            })
            .into_iter()
            .flatten()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_my_point() {
        let mut test_point = Point::new(1, 1);
        test_point = test_point.add(Point::new(2, 3));
        assert_eq!(test_point, Point::new(3, 4));
        assert_eq!(Cylindrical::from(test_point).r as i64, 5);
        let angle = 180.0;
        let on_negative_x_axis = Point::new(-10000, 0);
        let abs_difference = (Cylindrical::from(on_negative_x_axis).angle - angle).abs();
        assert!(abs_difference <= 0.01);
    }

    #[test]
    fn test_quadrant() {
        assert_eq!(Point::new(0, 0).quadrant(), Quadrant::Origin);
        assert_eq!(Point::new(9, 0).quadrant(), Quadrant::PositiveX);
        assert_eq!(Point::new(-8, 0).quadrant(), Quadrant::NegativeX);
        assert_eq!(Point::new(0, 7).quadrant(), Quadrant::PositiveY);
        assert_eq!(Point::new(0, -11).quadrant(), Quadrant::NegativeY);
        assert_eq!(Point::new(12, 13).quadrant(), Quadrant::First);
        assert_eq!(Point::new(-4, 8).quadrant(), Quadrant::Second);
        assert_eq!(Point::new(-4, -8).quadrant(), Quadrant::Third);
        assert_eq!(Point::new(7, -3).quadrant(), Quadrant::Fourth);
    }

    #[test]
    fn all_unambiguous_rotational_combinations() {
        use std::collections::HashSet;
        let mut rotation_results: HashSet<Point3D> = HashSet::new();
        let point = Point3D::new(1, 2, 3);

        // should not panic, which would happen, if there are more than 24 combinations
        let combinations = Point3D::all_unambiguous_rotation_combinations();

        // all combinations are unique?
        for (index, left) in combinations.iter().enumerate() {
            for right in combinations.iter().skip(index + 1) {
                assert_ne!(left, right);
            }
        }

        // all rotation results are unique?
        for combination in combinations {
            let rotated = point.apply_rotation_combination(combination);
            assert!(rotation_results.insert(rotated));
        }
    }
}
