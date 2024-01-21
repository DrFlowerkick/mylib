// library for some basic geometric functions
// at the moment I use integer, if possible, since a lot of challenges
// use integers or pixels

use crate::my_point::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Line {
    // a*x + b*y + c = 0
    a: i32,
    b: i32,
    c: i32,
}

impl Line {
    pub fn new(a: i32, b: i32, c: i32) -> Self {
        assert!(a != 0 || b != 0);
        Self { a, b, c }
    }
    pub fn new_linear_function(m: f32, q: f32, precision: u8) -> Self {
        // y = m*x + q
        assert!(m != 0. || q != 0.);
        let mut initial_precision = 0;
        let mut factor = 1.;
        while (m * factor).fract() > f32::EPSILON && initial_precision < precision {
            factor *= 10.;
            initial_precision += 1;
        }
        Self {
            a: (m * factor) as i32,
            b: (-1. * factor) as i32,
            c: (q * factor) as i32,
        }
    }
    pub fn get_m_q(&self) -> Option<(f32, f32)> {
        if self.b == 0 {
            return None;
        }
        let m = (self.a as f32) / (-self.b as f32);
        let q = (self.c as f32) / (-self.b as f32);
        Some((m, q))
    }
    pub fn y_of_x(&self, x: i32) -> Option<i32> {
        if self.b == 0 {
            None
        } else {
            Some((self.a * x + self.c) / -self.b)
        }
    }
    pub fn x_of_y(&self, y: i32) -> Option<i32> {
        if self.a == 0 {
            None
        } else {
            Some((self.b * y + self.c) / -self.a)
        }
    }
    pub fn y_of_x_float(&self, x: f32) -> Option<f32> {
        if let Some((m, q)) = self.get_m_q() {
            return Some(m * x + q);
        }
        None
    }
    pub fn x_of_y_float(&self, y: f32) -> Option<f32> {
        if self.a == 0 {
            return None;
        }
        match self.get_m_q() {
            Some((m, q)) => Some((y - q) / m),
            None => Some((self.c as f32) / (-self.a as f32)),
        }
    }
}

pub enum CrossingResult {
    None,
    Touching(Point),
    Crossing(Point, Point),
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct Circle {
    center: Point,
    radius: i32,
}

impl PartialEq for Circle {
    fn eq(&self, other: &Self) -> bool {
        self.radius == other.radius
    }
}

impl PartialOrd for Circle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Circle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.radius.cmp(&other.radius)
    }
}

impl PartialEq<Point> for Circle {
    // equal if Point is on circumference
    fn eq(&self, other: &Point) -> bool {
        (other.x - self.center.x).pow(2) + (other.y - self.center.y).pow(2) == self.radius.pow(2)
    }
}

impl PartialOrd<Point> for Circle {
    // Less: Point inside Circle
    // Equal: Point is on circumference of Circle
    // Greater: Point is outside of Circle
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(((other.x - self.center.x).pow(2) + (other.y - self.center.y).pow(2)).cmp(&self.radius.pow(2)))
    }
}

impl Circle {
    pub fn new(center: Point, radius: i32) -> Self {
        Self { center, radius }
    }
    pub fn shift(&self, center: Point) -> Self {
        Self {
            center,
            radius: self.radius,
        }
    }
    pub fn stretch(&self, factor: f32) -> Self {
        assert!(factor >= 0.0);
        Self {
            center: self.center,
            radius: (self.radius as f32 * factor) as i32,
        }
    }
    pub fn point_on_circumference(&self, angle: f32) -> Point {
        let poc = Point::from(Cylindrical::new(self.radius as f32, angle));
        self.center.add(poc)
    }
    // bool: true if a circle is inside of a circle
    pub fn circle_crossing(&self, other: &Self) -> (CrossingResult, bool) {
        match (self.radius - other.radius).abs().cmp(&(self.center.distance(other.center) as i32)) {
            Ordering::Greater => (CrossingResult::None, true),
            Ordering::Equal => if self.center == other.center {
                (CrossingResult::None, true)
            } else {
                (self.calc_crossing(other), true)
            },
            Ordering::Less => match (self.center.distance(other.center) as i32).cmp(&(self.radius + other.radius)) {
                Ordering::Greater => (CrossingResult::None, false),
                _ => (self.calc_crossing(other), false),
            }
        }
    }
    fn calc_crossing(&self, other: &Self) -> CrossingResult {
        assert!(self.center != other.center);
        // formulas
        // circle c_1: (x - x_c1)² + (y - y_c1)² = r1²
        // circle c_2: (x - x_c2)² + (y - y_c2)² = r2²
        // solve quadratic terms and substract
        // x * 2 *(x_c1 - x_c2) + y * 2 * (y_c1 - y_c2) + x_c2² - x_c1² + y_c2² - y_c1² + r1² - r2² = 0
        // line: a*x + b*y + c = 0
        let a = 2 * (self.center.x - other.center.x);
        let b = 2 * (self.center.y - other.center.y);
        let c = other.center.x.pow(2) - self.center.x.pow(2) + other.center.y.pow(2) - self.center.y.pow(2) + self.radius.pow(2) - other.radius.pow(2);
        let crossing_line = Line::new(a, b, c);
        self.line_crossing(crossing_line)
    }
    pub fn line_crossing(&self, line: Line) -> CrossingResult {
        if line.a == 0 {
            let sqrt_term = self.radius.pow(2) as f32 - (line.c as f32 / line.b as f32 + self.center.y as f32).powi(2);
            // formulas
            // line: y = -a/b * x -c/b
            // circle: (x - x_c)² + (y - y_c)² = r²
            // with a == 0
            // y = -c / b
            // (x - x_c)² + (-c/b - y_c)² = r²
            // x² - x * 2x_c + x_c² + (c/b + y_c)² - r² = 0
            // with p-q-formula
            // x1,2 = x_c +/- sqrt(r² - (c/b + y_c)²)
            let y_0 = -line.c / line.b;
            if sqrt_term < 0. {
                CrossingResult::None
            } else if sqrt_term < f32::EPSILON {
                CrossingResult::Touching(Point::new(self.center.x, y_0))
            } else {
                let x_1 = self.center.x - sqrt_term.sqrt() as i32;
                let x_2 = self.center.x + sqrt_term.sqrt() as i32;
                CrossingResult::Crossing(Point::new(x_1, y_0), Point::new(x_2, y_0))
            }
        } else if line.b == 0 {
            let sqrt_term = self.radius.pow(2) as f32 - (line.c as f32 / line.a as f32 + self.center.x as f32).powi(2);
            // formulas
            // line: y = -a/b * x -c/b
            // circle: (x - x_c)² + (y - y_c)² = r²
            // with b == 0
            // x = -c / a
            // (-c/a - x_c)² + (y - y_c)² = r²
            // y² - y * 2y_c + y_c² + (c/a + x_c)² - r² = 0
            // with p-q-formula
            // y1,2 = y_c +/- sqrt(r² - (c/a + x_c)²)
            let x_0 = -line.c / line.a;
            if sqrt_term < 0. {
                CrossingResult::None
            } else if sqrt_term < f32::EPSILON {
                CrossingResult::Touching(Point::new(x_0, self.center.y))
            } else {
                let y_1 = self.center.y - sqrt_term.sqrt() as i32;
                let y_2 = self.center.y + sqrt_term.sqrt() as i32;
                CrossingResult::Crossing(Point::new(x_0, y_1), Point::new(x_0, y_2))
            }
        } else {
            // formulas
            // line: y = -a/b * x -c/b
            // circle: (x - x_c)² + (y - y_c)² = r²
            // (x - x_c)² + (-a/b * x -c/b - y_c)² = r²
            // with y_d = c/b + yc
            // (x - x_c)² + (a/b * x + y_d)² = r²
            // with div = 1 + (a/b)²
            // x² + x * 2 * (y_d - x_c) / div + (x_c² + y_d² - r²) / div = 0
            // with p-q-formula
            // p = 2 * (y_d - x_c) / div
            // q = (x_c² + y_d² - r²) / div
            // x_0 = -p / 2
            // sqrt_term = (p/2)² - q
            let y_d = line.c as f32 / line.b as f32 + self.center.y as f32;
            let div = 1. + (line.a as f32 / line.b as f32).powi(2);
            let p = 2. * (y_d - self.center.x as f32) / div;
            let q = (self.center.x.pow(2) as f32 + y_d.powi(2) - self.radius.pow(2) as f32) / div;
            let x_0 = -p / 2.;
            let sqrt_term = (p / 2.).powi(2) - q;
            if sqrt_term < 0. {
                CrossingResult::None
            } else if sqrt_term < f32::EPSILON {
                let y_0 = line.y_of_x_float(x_0).unwrap();
                CrossingResult::Touching(Point::new(x_0 as i32, y_0 as i32))
            } else {
                let x_1 = x_0 - sqrt_term.sqrt();
                let x_2 = x_0 + sqrt_term.sqrt();
                let y_1 = line.y_of_x_float(x_1).unwrap();
                let y_2 = line.y_of_x_float(x_2).unwrap();
                CrossingResult::Crossing(Point::new(x_1 as i32, y_1 as i32), Point::new(x_2 as i32, y_2 as i32))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_circle_point_cmp() {
        let circle = Circle::new(Point::default(), 1_000);
        let outside = Point::new(1_000, 1_000);
        assert_eq!(circle.partial_cmp(&outside), Some(Ordering::Greater));
        let on_circumference = Point::new(0, 1_000);
        assert_eq!(circle.partial_cmp(&on_circumference), Some(Ordering::Equal));
        let inside = Point::new(500, 500);
        assert_eq!(circle.partial_cmp(&inside), Some(Ordering::Less));
    }

    #[test]
    fn test_circle_crossing() {
        let c1 = Circle::new(Point::default(), 1_000);
        let c2 = Circle::new(Point::new(1_000, 1_000), 1_000);
        if let (CrossingResult::Crossing(p1, p2), false) = c1.circle_crossing(&c2) {
            eprintln!("p1: {}, p2: {}", p1, p2);
            assert_eq!(p1, Point::new(0, 1_000));
            assert_eq!(p2, Point::new(1_000, 0));
        } else {
            panic!("wrong crossing result")
        }
        let c2 = Circle::new(Point::new(2_000, 0), 1_000);
        if let (CrossingResult::Touching(p), false) = c1.circle_crossing(&c2) {
            eprintln!("p: {}", p);
            assert_eq!(p, Point::new(1_000, 0));
        } else {
            panic!("wrong crossing result")
        }
    }
}
