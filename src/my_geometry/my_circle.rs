use super::{
    my_diamond::Diamond,
    my_line::{Line, LineSegment},
    my_point::{Cylindrical, Point},
    my_rectangle::Rectangle,
    FormOrdering,
};
use std::cmp::Ordering;

// Circle: (x - x_c)² + (y - y_c)² = r²
// with center: (x_c, y_c) and r: radius
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Circle {
    center: Point,
    radius: i64,
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
    pub fn new(center: Point, radius: i64) -> Self {
        assert!(radius > 0);
        Self { center, radius }
    }
    pub fn get_center(&self) -> Point {
        self.center
    }
    pub fn get_radius(&self) -> i64 {
        self.radius
    }
    pub fn shift(&self, center: Point) -> Self {
        Self {
            center,
            radius: self.radius,
        }
    }
    pub fn stretch(&self, offset: i64) -> Self {
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
            radius: (self.radius as f32 * factor) as i64,
        }
    }
    pub fn point_of_angle(&self, angle: f32) -> Point {
        let poc = Point::from(Cylindrical::new(self.radius as f32, angle));
        self.center.add(poc)
    }
    pub fn y_of_x(&self, x: i64) -> Vec<Point> {
        // formulas
        // circle: (x - x_c)² + (y - y_c)² = r²
        // y1,2 = y_c +/- sqrt(r² - (x - x_c)²)
        let sqrt_term = self.radius.pow(2) - (x - self.center.x).pow(2);
        let mut y: Vec<Point> = Vec::new();
        match sqrt_term {
            0 => y.push(Point::new(x, self.center.y)),
            1.. => {
                let y_1 = self.center.y - (sqrt_term as f32).sqrt() as i64;
                let y_2 = self.center.y + (sqrt_term as f32).sqrt() as i64;
                y.push(Point::new(x, y_1));
                y.push(Point::new(x, y_2));
            }
            _ => (),
        }
        y
    }
    pub fn x_of_y(&self, y: i64) -> Vec<Point> {
        // formulas
        // circle: (x - x_c)² + (y - y_c)² = r²
        // x1,2 = x_c +/- sqrt(r² - (y - y_c)²)
        let sqrt_term = self.radius.pow(2) - (y - self.center.y).pow(2);
        let mut x: Vec<Point> = Vec::new();
        match sqrt_term {
            0 => x.push(Point::new(self.center.x, y)),
            1.. => {
                let x_1 = self.center.x - (sqrt_term as f32).sqrt() as i64;
                let x_2 = self.center.x + (sqrt_term as f32).sqrt() as i64;
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
            .cmp(&(self.center.distance(other.center) as i64))
        {
            Ordering::Greater => FormOrdering::Inside,
            Ordering::Equal => {
                if self.center == other.center {
                    FormOrdering::Identical
                } else {
                    FormOrdering::InsideTouching
                }
            }
            Ordering::Less => match (self.center.distance(other.center) as i64)
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
                // solve quadratic terms and subtract
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
        if line.a() == 0 {
            // formulas
            // line: y = -a/b * x -c/b
            // circle: (x - x_c)² + (y - y_c)² = r²
            // with a == 0
            // y = -c / b
            // -> y is given, use x_of_y()
            let y_0 = -line.c() / line.b();
            self.x_of_y(y_0)
        } else if line.b() == 0 {
            // formulas
            // line: y = -a/b * x -c/b
            // circle: (x - x_c)² + (y - y_c)² = r²
            // with b == 0
            // x = -c / a
            // x is given, use y_of_x()
            let x_0 = -line.c() / line.a();
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
            let y_d = line.c() as f32 / line.b() as f32 + self.center.y as f32;
            let div = 1. + (line.a() as f32 / line.b() as f32).powi(2);
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
                intersection_result.push(Point::new(x_0 as i64, y_0 as i64));
            } else {
                let x_1 = x_0 - sqrt_term;
                let x_2 = x_0 + sqrt_term;
                let y_1 = line.y_of_x_float(x_1).unwrap();
                let y_2 = line.y_of_x_float(x_2).unwrap();
                intersection_result.push(Point::new(x_1 as i64, y_1 as i64));
                intersection_result.push(Point::new(x_2 as i64, y_2 as i64));
            }
            intersection_result
        }
    }

    pub fn circle_segment_intersection(&self, segment: &LineSegment) -> Vec<Point> {
        let csi: Vec<Point> = self
            .circle_line_intersection(&segment.line())
            .into_iter()
            .filter(|p| segment == p)
            .collect();
        csi
    }

    pub fn circle_rectangle_intersection(&self, rectangle: &Rectangle) -> Vec<Point> {
        rectangle.rectangle_circle_intersection(self)
    }

    pub fn circle_diamond_intersection(&self, diamond: &Diamond) -> Vec<Point> {
        diamond.diamond_circle_intersection(self)
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
}
