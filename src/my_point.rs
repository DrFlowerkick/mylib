// Point is a simple two dimensional point with positive and negative x and y dimension.
// It can also be used as a vector, e.g. see offset().

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
    pub fn new_angle_len(alpha: f32, len: f32) -> Point {
        Point {
            x: (alpha.to_radians().cos() * len) as i32,
            y: (alpha.to_radians().sin() * len) as i32,
        }
    }
    pub fn switch_xy(&self) -> Point {
        Point {
            x: self.y,
            y: self.x,
        }
    }
    pub fn distance_x(&self, target: Point) -> i32 {
        self.x - target.x
    }
    pub fn distance_y(&self, target: Point) -> i32 {
        self.y - target.y
    }
    pub fn distance(&self, target: Point) -> f32 {
        let result = (self.x - target.x).pow(2) + (self.y - target.y).pow(2);
        let result = (result as f32).sqrt();
        result.abs()
    }
    pub fn len(&self) -> f32 {
        self.distance(Point::new(0, 0))
    }
    pub fn angle(&self) -> f32 {
        let len = self.len();
        if (len as i32) == 0 {
            return 0.0; // return 0, if zero len vector
        }
        let alpha = ((self.x as f32) / self.len()).acos().to_degrees();
        if self.y < 0 {
            360.0 - alpha
        } else {
            alpha
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
    pub fn scale(&self, scale_factor: f32) -> Point {
        Point {
            x: (self.x as f32 * scale_factor) as i32,
            y: (self.y as f32 * scale_factor) as i32,
        }
    }
    pub fn scale_toward_point_with_len(&self, target: Point, len: f32) -> Point {
        let mut vector = target.subtract(*self);
        vector = vector.scale(len / vector.len());
        self.add(vector)
    }
    pub fn scale_toward_point_with_factor(&self, target: Point, factor: f32) -> Point {
        let mut vector = target.subtract(*self);
        vector = vector.scale(factor);
        self.add(vector)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_my_signed_point() {
        let mut test_point = Point::new(1, 1);
        test_point = test_point.add(Point::new(2, 3));
        assert_eq!(test_point, Point::new(3, 4));
        assert_eq!(test_point.len() as i32, 5);
        let angle = 180.0;
        let vector = Point::new_angle_len(angle, 10000.0);
        let abs_difference = (vector.angle() - angle).abs();
        assert!(abs_difference <= 0.01);
    }
}
