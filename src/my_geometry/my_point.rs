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
    pub fn distance_x(&self, target: Point) -> i64 {
        self.x - target.x
    }
    pub fn delta_x(&self, target: Point) -> i64 {
        (self.x - target.x).abs()
    }
    pub fn distance_y(&self, target: Point) -> i64 {
        self.y - target.y
    }
    pub fn delta_y(&self, target: Point) -> i64 {
        (self.y - target.y).abs()
    }
    pub fn distance(&self, target: Point) -> f32 {
        let result = self.distance_x(target).pow(2) + self.distance_y(target).pow(2);
        (result as f32).sqrt()
    }
    pub fn delta(&self, target: Point) -> i64 {
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
    pub fn add(&self, offset: Point) -> Point {
        Point {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }
    pub fn subtract(&self, offset: Point) -> Point {
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
    pub fn scale_toward_point_with_len(&self, target: Point, len: f32) -> Point {
        let vector = Cylindrical::from(target.subtract(*self));
        self.add(vector.set_radius(len).into())
    }
    pub fn scale_toward_point_with_factor(&self, target: Point, factor: f32) -> Point {
        let vector = Cylindrical::from(target.subtract(*self));
        self.add(vector.stretch(factor).into())
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

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Hash)]
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

    pub fn add(&self, other: &Self) -> Self {
        Point3D {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        Point3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn cross_product(&self, other: &Self) -> Self {
        Point3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
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
}
