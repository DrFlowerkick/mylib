use crate::my_geometry::{
    FormOrdering,
    my_circle::Circle,
    my_line::{Line, LineSegment},
    my_point::Point,
    my_rectangle::Rectangle,
};
use std::cmp::Ordering;

// Diamond is a "quadratic circle, standing on on tip", which uses as radius the manhattan distance to it circumference points
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
    // radius is manhattan distance
    radius: i64,
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
    pub fn new(center: Point, radius: i64) -> Self {
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
            radius: (self.radius as f32 * factor) as i64,
        }
    }
    pub fn y_of_x(&self, x: i64) -> Vec<Point> {
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
    pub fn x_of_y(&self, y: i64) -> Vec<Point> {
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
    pub fn corners(&self) -> [Point; 4] {
        [
            // top
            Point::new(self.center.x, self.center.y + self.radius),
            // right
            Point::new(self.center.x + self.radius, self.center.y),
            // bottom
            Point::new(self.center.x, self.center.y - self.radius),
            // left
            Point::new(self.center.x - self.radius, self.center.y),
        ]
    }
    pub fn sides(&self) -> [LineSegment; 4] {
        let sc = self.corners();
        [
            // quadrant 1
            LineSegment::new(sc[0], sc[1]),
            // quadrant 2
            LineSegment::new(sc[0], sc[3]),
            // quadrant 3
            LineSegment::new(sc[2], sc[3]),
            // quadrant 4
            LineSegment::new(sc[2], sc[1]),
        ]
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
            // no integer solution for odd difference between delta and sum of radii
            return intersection_points;
        } else {
            // check intersections for each side of each diamond
            // parallel sides are automatically ignored
            for self_side in self.sides().iter() {
                for intersection_point in other
                    .sides()
                    .iter()
                    .filter_map(|os| os.segment_intersection(self_side))
                {
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
        for side in self.sides().iter() {
            if let Some(intersection_point) = side.segment_line_intersection(line) {
                // check for duplicates, which can happen, if corners touch each other
                if !intersection_points.contains(&intersection_point) {
                    intersection_points.push(intersection_point);
                }
            }
        }
        intersection_points
    }

    pub fn diamond_rectangle_intersection(&self, rectangle: &Rectangle) -> Vec<Point> {
        rectangle.rectangle_diamond_intersection(self)
    }

    pub fn diamond_circle_intersection(&self, circle: &Circle) -> Vec<Point> {
        let mut dci: Vec<Point> = Vec::new();
        for side in self.sides().iter() {
            for si in circle.circle_segment_intersection(side).iter() {
                if !dci.contains(si) {
                    dci.push(*si);
                }
            }
        }
        dci
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
