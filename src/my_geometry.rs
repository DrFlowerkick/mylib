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
    pub fn line_intersection(&self, other: &Self) -> Option<Point> {
        // check if parallel: m_self == m_other
        // m_self = -a_self / b_self
        // m_other = -a_other / b_other
        // parallel, if a_self * b_other == a_other * b_self
        if self.a * other.b == other.a * self.b {
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
                (x as i32, y as i32)
            };
        Some(Point::new(x, y))
    }
}

pub enum FormOrdering {
    Identical,
    Inside,
    InsideTouching,
    Overlapping,
    Touching,
    NonOverlapping,
}

// Circle: (x - x_c)² + (y - y_c)² = r²
// with center: (x_c, y_c) and r: radius
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
        self.radius.pow(2) == (other.x - self.center.x).pow(2) + (other.y - self.center.y).pow(2)
    }
}

impl PartialOrd<Point> for Circle {
    // Greater: Point inside Circle
    // Equal: Point is on circumference of Circle
    // Less: Point is outside of Circle
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(
            self.radius
                .pow(2)
                .cmp(&((other.x - self.center.x).pow(2) + (other.y - self.center.y).pow(2))),
        )
    }
}

impl Circle {
    pub fn new(center: Point, radius: i32) -> Self {
        assert!(radius > 0);
        Self { center, radius }
    }
    pub fn get_center(&self) -> Point {
        self.center
    }
    pub fn get_radius(&self) -> i32 {
        self.radius
    }
    pub fn shift(&self, center: Point) -> Self {
        Self {
            center,
            radius: self.radius,
        }
    }
    pub fn stretch(&self, offset: i32) -> Self {
        assert!(self.radius + offset > 0);
        Self {
            center: self.center,
            radius: self.radius + offset,
        }
    }
    pub fn scale(&self, factor: f32) -> Self {
        assert!(factor > 0.0);
        Self {
            center: self.center,
            radius: (self.radius as f32 * factor) as i32,
        }
    }
    pub fn point_of_angle(&self, angle: f32) -> Point {
        let poc = Point::from(Cylindrical::new(self.radius as f32, angle));
        self.center.add(poc)
    }
    pub fn y_of_x(&self, x: i32) -> Vec<Point> {
        // formulas
        // circle: (x - x_c)² + (y - y_c)² = r²
        // y1,2 = y_c +/- sqrt(r² - (x - x_c)²)
        let sqrt_term = self.radius.pow(2) - (x - self.center.x).pow(2);
        let mut y: Vec<Point> = Vec::new();
        match sqrt_term {
            0 => y.push(Point::new(x, self.center.y)),
            1.. => {
                let y_1 = self.center.y - (sqrt_term as f32).sqrt() as i32;
                let y_2 = self.center.y + (sqrt_term as f32).sqrt() as i32;
                y.push(Point::new(x, y_1));
                y.push(Point::new(x, y_2));
            }
            _ => (),
        }
        y
    }
    pub fn x_of_y(&self, y: i32) -> Vec<Point> {
        // formulas
        // circle: (x - x_c)² + (y - y_c)² = r²
        // x1,2 = x_c +/- sqrt(r² - (y - y_c)²)
        let sqrt_term = self.radius.pow(2) - (y - self.center.y).pow(2);
        let mut x: Vec<Point> = Vec::new();
        match sqrt_term {
            0 => x.push(Point::new(self.center.x, y)),
            1.. => {
                let x_1 = self.center.x - (sqrt_term as f32).sqrt() as i32;
                let x_2 = self.center.x + (sqrt_term as f32).sqrt() as i32;
                x.push(Point::new(x_1, y));
                x.push(Point::new(x_2, y));
            }
            _ => (),
        }
        x
    }
    pub fn circle_cmp(&self, other: &Self) -> FormOrdering {
        match (self.radius - other.radius)
            .abs()
            .cmp(&(self.center.distance(other.center) as i32))
        {
            Ordering::Greater => FormOrdering::Inside,
            Ordering::Equal => {
                if self.center == other.center {
                    FormOrdering::Identical
                } else {
                    FormOrdering::InsideTouching
                }
            }
            Ordering::Less => match (self.center.distance(other.center) as i32)
                .cmp(&(self.radius + other.radius))
            {
                Ordering::Greater => FormOrdering::NonOverlapping,
                Ordering::Equal => FormOrdering::Touching,
                Ordering::Less => FormOrdering::Overlapping,
            },
        }
    }
    pub fn circle_intersection(&self, other: &Self) -> Vec<Point> {
        match self.circle_cmp(other) {
            FormOrdering::InsideTouching | FormOrdering::Overlapping | FormOrdering::Touching => {
                // formulas
                // circle c_1: (x - x_c1)² + (y - y_c1)² = r1²
                // circle c_2: (x - x_c2)² + (y - y_c2)² = r2²
                // solve quadratic terms and substract
                // x * 2 *(x_c1 - x_c2) + y * 2 * (y_c1 - y_c2) + x_c2² - x_c1² + y_c2² - y_c1² + r1² - r2² = 0
                // line: a*x + b*y + c = 0
                let a = 2 * (self.center.x - other.center.x);
                let b = 2 * (self.center.y - other.center.y);
                let c = other.center.x.pow(2) - self.center.x.pow(2) + other.center.y.pow(2)
                    - self.center.y.pow(2)
                    + self.radius.pow(2)
                    - other.radius.pow(2);
                let intersection_line = Line::new(a, b, c);
                self.circle_line_intersection(&intersection_line)
            }
            _ => Vec::new(),
        }
    }
    pub fn circle_line_intersection(&self, line: &Line) -> Vec<Point> {
        if line.a == 0 {
            // formulas
            // line: y = -a/b * x -c/b
            // circle: (x - x_c)² + (y - y_c)² = r²
            // with a == 0
            // y = -c / b
            // -> y is given, use x_of_y()
            let y_0 = -line.c / line.b;
            self.x_of_y(y_0)
        } else if line.b == 0 {
            // formulas
            // line: y = -a/b * x -c/b
            // circle: (x - x_c)² + (y - y_c)² = r²
            // with b == 0
            // x = -c / a
            // x is given, use y_of_x()
            let x_0 = -line.c / line.a;
            self.y_of_x(x_0)
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
            let sqrt_term = ((p / 2.).powi(2) - q).sqrt();
            let mut intersection_result: Vec<Point> = Vec::new();
            if sqrt_term.is_nan() {
                return intersection_result;
            }
            if sqrt_term < f32::EPSILON {
                let y_0 = line.y_of_x_float(x_0).unwrap();
                intersection_result.push(Point::new(x_0 as i32, y_0 as i32));
            } else {
                let x_1 = x_0 - sqrt_term;
                let x_2 = x_0 + sqrt_term;
                let y_1 = line.y_of_x_float(x_1).unwrap();
                let y_2 = line.y_of_x_float(x_2).unwrap();
                intersection_result.push(Point::new(x_1 as i32, y_1 as i32));
                intersection_result.push(Point::new(x_2 as i32, y_2 as i32));
            }
            intersection_result
        }
    }
}

// Diamond is a "quadrattic circle, standing on on tip", which uses as radius the manhatten distance to it circumference points
//                                                  3
//                               2                 323
//            1                 212               32123
// r=1 ->    1c1      r=2 ->   21c12     r=3 ->  321c123  a.s.f.
//            1                 212               32123
//                               2                 323
//                                                  3
// formulas for x and y on circumference of diamond depend on quadrant
// 1. quadrant (x>x_c, y>y_c): y - y_c + x - x_c = r
// 2. quadrant (x<x_c, y>y_c): y - y_c + x_c - x = r
// 3. quadrant (x<x_c, y<y_c): y_c - y + x_c - x = r
// 4. quadrant (x>x_c, y<y_c): y_c - y + x - x_c = r
// if x == x_c: y_1 = y_c + r and y_2 = y_c - r
// if y == y_c: x_1 = x_c + r and x_2 = x_c - r

#[derive(Debug, Clone, Copy, Eq)]
pub struct Diamond {
    center: Point,
    // radius is manhatten distance
    radius: i32,
}

impl PartialEq for Diamond {
    fn eq(&self, other: &Self) -> bool {
        self.radius == other.radius
    }
}

impl PartialOrd for Diamond {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Diamond {
    fn cmp(&self, other: &Self) -> Ordering {
        self.radius.cmp(&other.radius)
    }
}

impl PartialEq<Point> for Diamond {
    // equal if Point is on circumference
    fn eq(&self, other: &Point) -> bool {
        self.center.delta(*other) == self.radius
    }
}

impl PartialOrd<Point> for Diamond {
    // Greater: Point inside Diamond
    // Equal: Point is on circumference of Diamond
    // Less: Point is outside of Diamond
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.radius.cmp(&self.center.delta(*other)))
    }
}

impl Diamond {
    pub fn new(center: Point, radius: i32) -> Self {
        assert!(radius > 0);
        Self { center, radius }
    }
    pub fn get_center(&self) -> Point {
        self.center
    }
    pub fn get_radius(&self) -> i32 {
        self.radius
    }
    pub fn shift(&self, center: Point) -> Self {
        Self {
            center,
            radius: self.radius,
        }
    }
    pub fn stretch(&self, offset: i32) -> Self {
        assert!(self.radius - offset >= 0);
        Self {
            center: self.center,
            radius: self.radius + offset,
        }
    }
    pub fn scale(&self, factor: f32) -> Self {
        assert!(factor > 0.0);
        Self {
            center: self.center,
            radius: (self.radius as f32 * factor) as i32,
        }
    }
    pub fn y_of_x(&self, x: i32) -> Vec<Point> {
        let delta_x = (self.center.x - x).abs();
        let mut y: Vec<Point> = Vec::new();
        match delta_x.cmp(&self.radius) {
            Ordering::Equal => y.push(Point::new(x, self.center.y)),
            Ordering::Less => {
                let delta_y = self.radius - delta_x;
                y.push(Point::new(x, self.center.y - delta_y));
                y.push(Point::new(x, self.center.y + delta_y));
            }
            Ordering::Greater => (),
        }
        y
    }
    pub fn x_of_y(&self, y: i32) -> Vec<Point> {
        let delta_y = (self.center.y - y).abs();
        let mut x: Vec<Point> = Vec::new();
        match delta_y.cmp(&self.radius) {
            Ordering::Equal => x.push(Point::new(self.center.x, y)),
            Ordering::Less => {
                let delta_x = self.radius - delta_y;
                x.push(Point::new(self.center.x - delta_x, y));
                x.push(Point::new(self.center.x + delta_x, y));
            }
            Ordering::Greater => (),
        }
        x
    }
    pub fn diamond_cmp(&self, other: &Self) -> FormOrdering {
        match (self.radius - other.radius)
            .abs()
            .cmp(&self.center.delta(other.center))
        {
            Ordering::Greater => FormOrdering::Inside,
            Ordering::Equal => {
                if self.center == other.center {
                    FormOrdering::Identical
                } else {
                    FormOrdering::InsideTouching
                }
            }
            Ordering::Less => match self
                .center
                .delta(other.center)
                .cmp(&(self.radius + other.radius))
            {
                Ordering::Greater => FormOrdering::NonOverlapping,
                Ordering::Equal => FormOrdering::Touching,
                Ordering::Less => FormOrdering::Overlapping,
            },
        }
    }
    pub fn diamond_intersection(&self, other: &Self) -> Vec<Point> {
        // If a diamond corner is on circumference of other diamond, both
        // circumference overlap. In this case, the only Points returned are
        // points which match at least one corner of a diamond
        let mut intersection_points: Vec<Point> = Vec::new();
        if matches!(
            self.diamond_cmp(other),
            FormOrdering::Identical | FormOrdering::Inside | FormOrdering::NonOverlapping
        ) || (self.radius + other.radius - self.center.delta(other.center)) % 2 != 0
        {
            // no touching or overlapping
            // no integer solution for odd difference between delta and sum of radi
            return intersection_points;
        } else {
            // lines of sides
            // 1. quadrant (x>x_c, y>y_c): y - y_c + x - x_c = r
            // 2. quadrant (x<x_c, y>y_c): y - y_c + x_c - x = r
            // 3. quadrant (x<x_c, y<y_c): y_c - y + x_c - x = r
            // 4. quadrant (x>x_c, y<y_c): y_c - y + x - x_c = r
            // When checking for intersections of diamonds, you have to check for intersection of each
            // side of each diamond. Since sides of a diamond are lines with m=1 or m=-1, parallel
            // sides of diamonds cannot intersect. Therefore we compare every line with different m.
            // This results in 8 formulas to check (q: quadrant, d: diamond):
            // q1d1::q2d2 and q1d2::q2d1
            // q2d1::q3d2 and q2d2::q3d1
            // q3d1::q4d2 and q3d2::q4d1
            // q4d1::q1d2 and q4d2::q1d1
            // all these combinations result in a intersection point, but only points, which are on
            // circumferences of both diamonds are valid
            let q1d1 = Line::new(1, 1, -self.center.y - self.center.x - self.radius);
            let q2d1 = Line::new(-1, 1, -self.center.y + self.center.x - self.radius);
            let q3d1 = Line::new(-1, -1, self.center.y + self.center.x - self.radius);
            let q4d1 = Line::new(1, -1, self.center.y - self.center.x - self.radius);
            let q1d2 = Line::new(1, 1, -other.center.y - other.center.x - other.radius);
            let q2d2 = Line::new(-1, 1, -other.center.y + other.center.x - other.radius);
            let q3d2 = Line::new(-1, -1, other.center.y + other.center.x - other.radius);
            let q4d2 = Line::new(1, -1, other.center.y - other.center.x - other.radius);
            let line_pairs = [
                (q1d1, q2d2),
                (q1d2, q2d1),
                (q2d1, q3d2),
                (q2d2, q3d1),
                (q3d1, q4d2),
                (q3d2, q4d1),
                (q4d1, q1d2),
                (q4d2, q1d1),
            ];
            for (side_1, side_2) in line_pairs.iter() {
                let intersection_point = side_1.line_intersection(side_2).unwrap();
                if *self == intersection_point && *other == intersection_point {
                    // check for duplicates, which can happen, if corners touch each other
                    if !intersection_points.contains(&intersection_point) {
                        intersection_points.push(intersection_point);
                    }
                }
            }
        }
        intersection_points
    }
    pub fn diamond_line_intersection(&self, line: &Line) -> Vec<Point> {
        let mut intersection_points: Vec<Point> = Vec::new();
        let q1 = Line::new(1, 1, -self.center.y - self.center.x - self.radius);
        let q2 = Line::new(-1, 1, -self.center.y + self.center.x - self.radius);
        let q3 = Line::new(-1, -1, self.center.y + self.center.x - self.radius);
        let q4 = Line::new(1, -1, self.center.y - self.center.x - self.radius);
        let diamond_sides = [q1, q2, q3, q4];
        for diamond_side in diamond_sides.iter() {
            if let Some(intersection_point) = diamond_side.line_intersection(line) {
                if *self == intersection_point {
                    // check for duplicates, which can happen, if corners touch each other
                    if !intersection_points.contains(&intersection_point) {
                        intersection_points.push(intersection_point);
                    }
                }
            }
        }
        intersection_points
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_circle_point_cmp() {
        let circle = Circle::new(Point::default(), 1_000);
        let outside = Point::new(1_000, 1_000);
        assert_eq!(circle.partial_cmp(&outside), Some(Ordering::Less));
        assert!(circle < outside);
        let on_circumference = Point::new(0, 1_000);
        assert_eq!(circle.partial_cmp(&on_circumference), Some(Ordering::Equal));
        assert!(circle == on_circumference);
        let inside = Point::new(500, 500);
        assert_eq!(circle.partial_cmp(&inside), Some(Ordering::Greater));
        assert!(circle > inside);
    }

    #[test]
    fn test_circle_intersection() {
        let c1 = Circle::new(Point::default(), 1_000);
        let c2 = Circle::new(Point::new(1_000, 1_000), 1_000);
        let intersection_result = c1.circle_intersection(&c2);
        assert_eq!(intersection_result.len(), 2);
        if let Some(p1) = intersection_result.get(0) {
            eprintln!("p1: {}", p1);
            assert_eq!(*p1, Point::new(0, 1_000));
        }
        if let Some(p2) = intersection_result.get(1) {
            eprintln!("p2: {}", p2);
            assert_eq!(*p2, Point::new(1_000, 0));
        }
        // touching
        let c2 = Circle::new(Point::new(2_000, 0), 1_000);
        let intersection_result = c1.circle_intersection(&c2);
        assert_eq!(intersection_result.len(), 1);
        if let Some(p) = intersection_result.get(0) {
            eprintln!("p: {}", p);
            assert_eq!(*p, Point::new(1_000, 0));
        }
        let c2 = Circle::new(Point::new(0, 2_000), 1_000);
        let intersection_result = c1.circle_intersection(&c2);
        assert_eq!(intersection_result.len(), 1);
        if let Some(p) = intersection_result.get(0) {
            eprintln!("p: {}", p);
            assert_eq!(*p, Point::new(0, 1_000));
        }
    }

    #[test]
    fn test_diamond_point_cmp() {
        let diamond = Diamond::new(Point::default(), 1_000);
        let outside = Point::new(1_000, 1_000);
        assert_eq!(diamond.partial_cmp(&outside), Some(Ordering::Less));
        assert!(diamond < outside);
        let on_circumference = Point::new(0, 1_000);
        assert_eq!(
            diamond.partial_cmp(&on_circumference),
            Some(Ordering::Equal)
        );
        assert!(diamond == on_circumference);
        let inside = Point::new(50, 50);
        assert_eq!(diamond.partial_cmp(&inside), Some(Ordering::Greater));
        assert!(diamond > inside);
    }
    #[test]
    fn test_diamond_intersection() {
        let d1 = Diamond::new(Point::default(), 5);
        let d2 = Diamond::new(Point::new(2, -6), 5);
        let intersection_result = d1.diamond_intersection(&d2);
        assert_eq!(intersection_result.len(), 2);
        if let Some(p1) = intersection_result.get(0) {
            eprintln!("p1: {}", p1);
            assert_eq!(*p1, Point::new(-1, -4));
        }
        if let Some(p2) = intersection_result.get(1) {
            eprintln!("p2: {}", p2);
            assert_eq!(*p2, Point::new(3, -2));
        }
        // with overlapping side
        let d1 = Diamond::new(Point::default(), 8);
        let d2 = Diamond::new(Point::new(3, 6), 5);
        let mut intersection_result = d1.diamond_intersection(&d2);
        intersection_result.sort_by(|a, b| a.x.cmp(&b.x));
        assert_eq!(intersection_result.len(), 3);
        if let Some(p1) = intersection_result.get(0) {
            eprintln!("p1: {}", p1);
            assert_eq!(*p1, Point::new(-2, 6));
        }
        if let Some(p2) = intersection_result.get(1) {
            eprintln!("p2: {}", p2);
            assert_eq!(*p2, Point::new(0, 8));
        }
        if let Some(p3) = intersection_result.get(2) {
            eprintln!("p3: {}", p3);
            assert_eq!(*p3, Point::new(5, 3));
        }
        // touching corner
        let d1 = Diamond::new(Point::default(), 8);
        let d2 = Diamond::new(Point::new(10, 0), 2);
        let intersection_result = d1.diamond_intersection(&d2);
        eprintln!("{:?}", intersection_result);
        assert_eq!(intersection_result.len(), 1);
        if let Some(p) = intersection_result.get(0) {
            eprintln!("p: {}", p);
            assert_eq!(*p, Point::new(8, 0));
        }
    }
}
