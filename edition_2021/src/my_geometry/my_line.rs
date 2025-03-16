use super::my_point::Point;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Line {
    // a*x + b*y + c = 0
    a: i64,
    b: i64,
    c: i64,
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.is_parallel(other) && {
            if self.b == 0 {
                self.c * other.a == other.c * self.a
            } else {
                self.c * other.b == other.c * self.b
            }
        }
    }
}

impl PartialEq<Point> for Line {
    // equal if Point is on Line
    fn eq(&self, other: &Point) -> bool {
        self.a * other.x + self.b * other.y + self.c == 0
    }
}

impl From<(Point, Point)> for Line {
    fn from(value: (Point, Point)) -> Self {
        Self {
            a: value.0.y - value.1.y,
            b: value.1.x - value.0.x,
            c: value.0.x * value.1.y - value.1.x * value.0.y,
        }
    }
}

impl Line {
    pub fn new(a: i64, b: i64, c: i64) -> Self {
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
            a: (m * factor) as i64,
            b: (-1. * factor) as i64,
            c: (q * factor) as i64,
        }
    }
    pub fn get_line_parameter(&self) -> (i64, i64, i64) {
        (self.a, self.b, self.c)
    }
    pub fn a(&self) -> i64 {
        self.a
    }
    pub fn b(&self) -> i64 {
        self.b
    }
    pub fn c(&self) -> i64 {
        self.c
    }
    pub fn get_m_q(&self) -> Option<(f32, f32)> {
        if self.b == 0 {
            return None;
        }
        let m = (self.a as f32) / (-self.b as f32);
        let q = (self.c as f32) / (-self.b as f32);
        Some((m, q))
    }
    pub fn y_of_x(&self, x: i64) -> Option<i64> {
        if self.b == 0 {
            None
        } else {
            Some((self.a * x + self.c) / -self.b)
        }
    }
    pub fn x_of_y(&self, y: i64) -> Option<i64> {
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
    pub fn is_parallel(&self, other: &Self) -> bool {
        // check if parallel: m_self == m_other
        // m_self = -a_self / b_self
        // m_other = -a_other / b_other
        // parallel, if a_self * b_other == a_other * b_self
        self.a * other.b == other.a * self.b
    }
    pub fn line_intersection(&self, other: &Self) -> Option<Point> {
        // check if parallel
        if self.is_parallel(other) {
            return None;
        }
        // intersection: y_self == y_other
        // y_self = -a_self/b_self * x - c_self/b_self
        // y_other = -a_other/b_other * x - c_other/b_other
        // a_self/b_self * x + c_self/b_self = a_other/b_other * x + c_other/b_other
        // x * (a_self * b_other - a_other * b_self) = c_other * b_self - c_self * b_other
        // x = (c_other * b_self - c_self * b_other) / (a_self * b_other - a_other * b_self)
        // Insert x in one line to yield y
        let (x, y) =
            if (other.c * self.b - self.c * other.b) % (self.a * other.b - other.a * self.b) == 0 {
                let x =
                    (other.c * self.b - self.c * other.b) / (self.a * other.b - other.a * self.b);
                // check if self is vertical
                let y = match self.y_of_x(x) {
                    Some(y) => y,
                    None => other.y_of_x(x).unwrap(),
                };
                (x, y)
            } else {
                let x = (other.c * self.b - self.c * other.b) as f32
                    / (self.a * other.b - other.a * self.b) as f32;
                // check if self is vertical
                let y = match self.y_of_x_float(x) {
                    Some(y) => y,
                    None => other.y_of_x_float(x).unwrap(),
                };
                (x as i64, y as i64)
            };
        Some(Point::new(x, y))
    }
    pub fn line_segment_intersection(&self, segment: &LineSegment) -> Option<Point> {
        segment.segment_line_intersection(self)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LineSegment {
    a: Point,
    b: Point,
}

impl PartialOrd for LineSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.delta().cmp(&other.delta()))
    }
}

impl PartialEq<Point> for LineSegment {
    fn eq(&self, other: &Point) -> bool {
        self.line() == *other && self.min_x() <= other.x && other.x <= self.max_x()
    }
}

impl LineSegment {
    pub fn new(a: Point, b: Point) -> Self {
        assert!(a != b);
        Self { a, b }
    }
    pub fn end_points(&self) -> [Point; 2] {
        [self.a, self.b]
    }
    pub fn line(&self) -> Line {
        Line::from((self.a, self.b))
    }
    pub fn min_x(&self) -> i64 {
        self.a.x.min(self.b.x)
    }
    pub fn max_x(&self) -> i64 {
        self.a.x.max(self.b.x)
    }
    pub fn min_y(&self) -> i64 {
        self.a.y.min(self.b.y)
    }
    pub fn max_y(&self) -> i64 {
        self.a.y.max(self.b.y)
    }
    pub fn len(&self) -> f32 {
        self.a.distance(self.b)
    }
    pub fn delta(&self) -> i64 {
        self.a.delta(self.b)
    }
    pub fn is_parallel(&self, other: &Self) -> bool {
        self.line().is_parallel(&other.line())
    }
    pub fn segment_intersection(&self, other: &Self) -> Option<Point> {
        if let Some(si) = self.line().line_intersection(&other.line()) {
            if self == &si && other == &si {
                return Some(si);
            }
        }
        None
    }
    pub fn segment_line_intersection(&self, line: &Line) -> Option<Point> {
        if let Some(si) = self.line().line_intersection(line) {
            if self == &si {
                return Some(si);
            }
        }
        None
    }
    pub fn segment_overlapping(&self, other: &Self) -> Vec<Point> {
        let mut so: Vec<Point> = Vec::with_capacity(2);
        for ep in self.end_points().iter().filter(|p| other == *p) {
            so.push(*ep);
        }
        for ep in other.end_points().iter().filter(|p| self == *p) {
            if !so.contains(ep) {
                so.push(*ep);
            }
        }
        so
    }
}
