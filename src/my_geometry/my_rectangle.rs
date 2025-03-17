use super::{
    my_circle::Circle,
    my_diamond::Diamond,
    my_line::{Line, LineSegment},
    my_point::Point,
    FormOrdering,
};
use std::cmp::Ordering;

// Rectangle: defined by top-left and bottom-right point
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

impl PartialOrd for Rectangle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rectangle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.surface().cmp(&other.surface())
    }
}

impl PartialEq<Point> for Rectangle {
    // equal if Point is on one segment
    // if corner, point is on 2 segments
    fn eq(&self, other: &Point) -> bool {
        self.sides().iter().filter(|s| *s == other).count() > 0
    }
}

impl PartialOrd<Point> for Rectangle {
    // Greater: Point inside Rectangle
    // Equal: Point is on circumference of Rectangle
    // Less: Point is outside of Rectangle
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.top_left.x < other.x
            && other.x < self.bottom_right.x
            && self.top_left.y > other.y
            && other.y > self.bottom_right.y
        {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Rectangle {
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        assert!(top_left.x < bottom_right.x);
        assert!(top_left.y > bottom_right.y);
        Self {
            top_left,
            bottom_right,
        }
    }
    pub fn size_x(&self) -> i64 {
        self.bottom_right.x - self.top_left.x
    }
    pub fn size_y(&self) -> i64 {
        self.top_left.y - self.bottom_right.y
    }
    pub fn surface(&self) -> i64 {
        self.size_x() * self.size_y()
    }
    pub fn corners(&self) -> [Point; 4] {
        [
            self.top_left,
            // top right
            Point::new(self.bottom_right.x, self.top_left.y),
            // bottom left
            Point::new(self.top_left.x, self.bottom_right.y),
            self.bottom_right,
        ]
    }
    pub fn sides(&self) -> [LineSegment; 4] {
        [
            // top
            LineSegment::new(
                self.top_left,
                Point::new(self.bottom_right.x, self.top_left.y),
            ),
            // right
            LineSegment::new(
                Point::new(self.bottom_right.x, self.top_left.y),
                self.bottom_right,
            ),
            // bottom
            LineSegment::new(
                self.bottom_right,
                Point::new(self.top_left.x, self.bottom_right.y),
            ),
            // left
            LineSegment::new(
                Point::new(self.top_left.x, self.bottom_right.y),
                self.top_left,
            ),
        ]
    }
    pub fn overlapping_corners(&self, other: &Self) -> Vec<Point> {
        let mut oc: Vec<Point> = Vec::new();
        for corner in other.corners().iter() {
            if self >= corner {
                oc.push(*corner);
            }
        }
        oc
    }
    pub fn rectangle_cmp(&self, other: &Self) -> FormOrdering {
        if self == other {
            return FormOrdering::Identical;
        }
        let oc_other_in_self = self.overlapping_corners(other);
        let oc_self_in_other = other.overlapping_corners(self);
        match oc_other_in_self.len().cmp(&oc_self_in_other.len()) {
            Ordering::Greater => {
                if oc_other_in_self.len() == 4 && oc_other_in_self.iter().any(|c| self == c) {
                    FormOrdering::InsideTouching
                } else if oc_other_in_self.len() == 4 {
                    FormOrdering::Inside
                } else if oc_other_in_self.iter().any(|c| self == c) {
                    // side is touching
                    FormOrdering::Touching
                } else {
                    FormOrdering::Overlapping
                }
            }
            Ordering::Less => {
                if oc_self_in_other.len() == 4 && oc_self_in_other.iter().any(|c| other == c) {
                    FormOrdering::InsideTouching
                } else if oc_self_in_other.len() == 4 {
                    FormOrdering::Inside
                } else if oc_self_in_other.iter().any(|c| other == c) {
                    // side is touching
                    FormOrdering::Touching
                } else {
                    FormOrdering::Overlapping
                }
            }
            Ordering::Equal => {
                if oc_other_in_self.is_empty() {
                    FormOrdering::NonOverlapping
                } else if oc_other_in_self.len() == 2
                    && oc_other_in_self
                        .iter()
                        .all(|c| oc_self_in_other.contains(c))
                {
                    // side is touching
                    FormOrdering::Touching
                } else if oc_other_in_self.len() == 2 {
                    // overlapping rectangles with either equal size_x or size_y and 2 corners on sides of other rectangle and vice versa
                    FormOrdering::Overlapping
                } else if oc_other_in_self.iter().any(|c| self == c) {
                    // corner is touching
                    FormOrdering::Touching
                } else {
                    FormOrdering::Overlapping
                }
            }
        }
    }

    pub fn rectangle_intersection(&self, other: &Self) -> Vec<Point> {
        let mut ri: Vec<Point> = Vec::new();
        for self_side in self.sides().iter() {
            for other_side in other.sides().iter() {
                if let Some(ip) = self_side.segment_intersection(other_side) {
                    if !ri.contains(&ip) {
                        ri.push(ip);
                    }
                }
                for op in self_side.segment_overlapping(other_side).iter() {
                    if !ri.contains(op) {
                        ri.push(*op);
                    }
                }
            }
        }
        ri
    }

    pub fn rectangle_line_intersection(&self, line: &Line) -> Vec<Point> {
        let mut rli: Vec<Point> = Vec::new();
        for side in self.sides().iter() {
            if let Some(si) = side.segment_line_intersection(line) {
                if !rli.contains(&si) {
                    rli.push(si);
                }
            }
        }
        rli
    }

    pub fn rectangle_segment_intersection(&self, segment: &LineSegment) -> Vec<Point> {
        let mut rli: Vec<Point> = Vec::new();
        for side in self.sides().iter() {
            if let Some(si) = side.segment_intersection(segment) {
                if !rli.contains(&si) {
                    rli.push(si);
                }
            }
        }
        rli
    }

    pub fn rectangle_diamond_intersection(&self, diamond: &Diamond) -> Vec<Point> {
        let mut rdi: Vec<Point> = Vec::new();
        for rside in self.sides().iter() {
            for dside in diamond.sides().iter() {
                if let Some(si) = rside.segment_intersection(dside) {
                    if !rdi.contains(&si) {
                        rdi.push(si);
                    }
                }
            }
        }
        rdi
    }

    pub fn rectangle_circle_intersection(&self, circle: &Circle) -> Vec<Point> {
        let mut rci: Vec<Point> = Vec::new();
        for side in self.sides().iter() {
            for si in circle.circle_segment_intersection(side).iter() {
                if !rci.contains(si) {
                    rci.push(*si);
                }
            }
        }
        rci
    }
}
