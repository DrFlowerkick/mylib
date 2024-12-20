use crate::my_compass::*;
use crate::my_geometry::my_line::Line;
use crate::my_geometry::my_point::Point;
use crate::my_geometry::my_rectangle::Rectangle;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Hash)]
pub struct MapPoint<const X: usize, const Y: usize> {
    // X: size of dimension x
    // Y: size of dimension Y
    // x and y are not public, because changing them without the provided functions can result in unwanted panics!
    x: usize,
    y: usize,
}

impl<const X: usize, const Y: usize> PartialOrd for MapPoint<X, Y> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const X: usize, const Y: usize> Ord for MapPoint<X, Y> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.y.cmp(&other.y) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.x.cmp(&other.x),
        }
    }
}

impl<const X: usize, const Y: usize> Display for MapPoint<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<const X: usize, const Y: usize> From<(usize, usize)> for MapPoint<X, Y> {
    fn from(value: (usize, usize)) -> Self {
        MapPoint::<X, Y>::new(value.0, value.1)
    }
}

impl<const X: usize, const Y: usize> From<MapPoint<X, Y>> for (usize, usize) {
    fn from(value: MapPoint<X, Y>) -> Self {
        (value.x(), value.y())
    }
}

impl<const X: usize, const Y: usize> TryFrom<Point> for MapPoint<X, Y> {
    type Error = &'static str;

    fn try_from(value: Point) -> Result<Self, Self::Error> {
        if value.x < 0 || value.x as usize >= X || value.y < 0 || value.y as usize >= Y {
            Err("Point values outside of map range")
        } else {
            Ok(MapPoint::new(value.x as usize, value.y as usize))
        }
    }
}



impl<const X: usize, const Y: usize> MapPoint<X, Y> {
    pub const NW: MapPoint<X, Y> = MapPoint { x: 0, y: 0 };
    pub const NE: MapPoint<X, Y> = MapPoint { x: X - 1, y: 0 };
    pub const SW: MapPoint<X, Y> = MapPoint { x: 0, y: Y - 1 };
    pub const SE: MapPoint<X, Y> = MapPoint { x: X - 1, y: Y - 1 };
    pub const fn new_const(x: usize, y: usize) -> MapPoint<X, Y> {
        MapPoint { x, y }
    }
    pub fn new(x: usize, y: usize) -> Self {
        if X == 0 {
            panic!("line {}, minimum size of dimension X is 1", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum size of dimension Y is 1", line!());
        }
        let result = MapPoint { x, y };
        if !result.is_in_map() {
            panic!("line {}, coordinates are out of range", line!());
        }
        result
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn distance_x(&self, target: MapPoint<X, Y>) -> usize {
        match self.x.cmp(&target.x) {
            Ordering::Equal => 0,
            Ordering::Greater => self.x - target.x,
            Ordering::Less => target.x - self.x,
        }
    }
    pub fn distance_y(&self, target: MapPoint<X, Y>) -> usize {
        match self.y.cmp(&target.y) {
            Ordering::Equal => 0,
            Ordering::Greater => self.y - target.y,
            Ordering::Less => target.y - self.y,
        }
    }
    pub fn distance(&self, target: MapPoint<X, Y>) -> usize {
        self.distance_x(target) + self.distance_y(target)
    }
    pub fn delta_xy(&self, target: MapPoint<X, Y>) -> usize {
        let dist_x = self.distance_x(target);
        let dist_y = self.distance_y(target);
        match dist_x.cmp(&dist_y) {
            Ordering::Equal => 0,
            Ordering::Greater => dist_x - dist_y,
            Ordering::Less => dist_y - dist_x,
        }
    }
    pub fn is_in_map(&self) -> bool {
        self.x < X && self.y < Y
    }
    pub fn map_position(&self) -> Compass {
        match (self.x, self.y) {
            (0, 0) => Compass::NW,
            (x, 0) if x == X - 1 => Compass::NE,
            (0, y) if y == Y - 1 => Compass::SW,
            (x, y) if x == X - 1 && y == Y - 1 => Compass::SE,
            (x, 0) if x < X - 1 => Compass::N,
            (0, y) if y < Y - 1 => Compass::W,
            (x, y) if y == Y - 1 && x < X - 1 => Compass::S,
            (x, y) if x == X - 1 && y < Y - 1 => Compass::E,
            _ => Compass::Center,
        }
    }
    pub fn available_cardinal_directions(&self) -> Vec<Compass> {
        match self.map_position() {
            Compass::Center => vec![Compass::N, Compass::E, Compass::S, Compass::W],
            Compass::N => vec![Compass::E, Compass::S, Compass::W],
            Compass::E => vec![Compass::N, Compass::S, Compass::W],
            Compass::S => vec![Compass::N, Compass::E, Compass::W],
            Compass::W => vec![Compass::N, Compass::E, Compass::S],
            Compass::NE => vec![Compass::S, Compass::W],
            Compass::SE => vec![Compass::N, Compass::W],
            Compass::SW => vec![Compass::N, Compass::E],
            Compass::NW => vec![Compass::E, Compass::S],
        }
    }
    pub fn forward_x(&self) -> Option<MapPoint<X, Y>> {
        // increments x, if x reaches row end, move to start of next row; if x reaches end of map, return None
        let mut result = *self;
        result.x += 1;
        if result.x == X {
            result.y += 1;
            if result.y == Y {
                return None;
            }
            result.x = 0;
        }
        Some(result)
    }
    pub fn backward_x(&self) -> Option<MapPoint<X, Y>> {
        // decrements x, if x reaches row start, move to end of previous row; if x reaches start of map, return None
        let mut result = *self;
        if result.x == 0 {
            if result.y == 0 {
                return None;
            }
            result.y -= 1;
            result.x = X - 1;
        } else {
            result.x -= 1;
        }
        Some(result)
    }
    pub fn forward_y(&self) -> Option<MapPoint<X, Y>> {
        // increments y, if y reaches column end, move to end of next column; if y reaches end of map, return None
        let mut result = *self;
        result.y += 1;
        if result.y == Y {
            result.x += 1;
            if result.x == X {
                return None;
            }
            result.y = 0;
        }
        Some(result)
    }
    pub fn backward_y(&self) -> Option<MapPoint<X, Y>> {
        // decrements y, if y reaches column start, move to end of previous column; if y reaches start of map, return None
        let mut result = *self;
        if result.y == 0 {
            if result.x == 0 {
                return None;
            }
            result.x -= 1;
            result.y = Y - 1;
        } else {
            result.y -= 1;
        }
        Some(result)
    }
    pub fn offset_pp(&self, offset: (usize, usize)) -> Option<MapPoint<X, Y>> {
        let result = MapPoint {
            x: self.x + offset.0,
            y: self.y + offset.1,
        };
        if result.is_in_map() {
            Some(result)
        } else {
            None
        }
    }
    pub fn offset_mm(&self, offset: (usize, usize)) -> Option<MapPoint<X, Y>> {
        if offset.0 > self.x || offset.1 > self.y {
            return None;
        }
        let result = MapPoint {
            x: self.x - offset.0,
            y: self.y - offset.1,
        };
        if result.is_in_map() {
            Some(result)
        } else {
            None
        }
    }
    pub fn invert_x(&self) -> MapPoint<X, Y> {
        Self {
            x: X - 1 - self.x,
            y: self.y,
        }
    }
    pub fn invert_y(&self) -> MapPoint<X, Y> {
        Self {
            x: self.x,
            y: Y - 1 - self.y,
        }
    }
    pub fn invert_x_and_y(&self) -> MapPoint<X, Y> {
        Self {
            x: X - 1 - self.x,
            y: Y - 1 - self.y,
        }
    }
    pub fn neighbor(&self, orientation: Compass) -> Option<MapPoint<X, Y>> {
        match orientation {
            Compass::Center => Some(*self),
            Compass::N => self.offset_mm((0, 1)),
            Compass::NE => self.offset_mm((0, 1)).and_then(|n| n.offset_pp((1, 0))),
            Compass::E => self.offset_pp((1, 0)),
            Compass::SE => self.offset_pp((1, 1)),
            Compass::S => self.offset_pp((0, 1)),
            Compass::SW => self.offset_pp((0, 1)).and_then(|s| s.offset_mm((1, 0))),
            Compass::W => self.offset_mm((1, 0)),
            Compass::NW => self.offset_mm((1, 1)),
        }
    }
    pub fn orientation_of_neighbor(&self, neighbor: MapPoint<X, Y>) -> Option<Compass> {
        self.iter_neighbors(Compass::N, true, false, true)
            .find(|(p, _)| *p == neighbor)
            .map(|(_, o)| o)
    }
    pub fn iter_neighbors(
        &self,
        initial_orientation: Compass,
        rotation_direction: bool,
        include_center: bool,
        include_corners: bool,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, Compass)> {
        NeighborIter::new(
            *self,
            initial_orientation,
            rotation_direction,
            include_center,
            include_corners,
        )
    }
    pub fn iter_orientation(&self, orientation: Compass) -> impl Iterator<Item = MapPoint<X, Y>> {
        OrientationIter::new(*self, orientation, false, Compass::Center)
    }
    pub fn iter_orientation_wrap_around(
        &self,
        orientation: Compass,
        offset: Compass,
    ) -> impl Iterator<Item = MapPoint<X, Y>> {
        OrientationIter::new(*self, orientation, true, offset)
    }
    pub fn iter_edge(&self, counterclockwise: bool) -> impl Iterator<Item = MapPoint<X, Y>> {
        EdgeIter::new(*self, counterclockwise)
    }
}

struct NeighborIter<const X: usize, const Y: usize> {
    include_center: bool,
    include_corners: bool,
    center_point: MapPoint<X, Y>,
    initial_orientation: Compass,
    current_orientation: Compass,
    // true: rotate clockwise, false: rotate counterclockwise
    rotation_direction: bool,
    finished: bool,
}

impl<const X: usize, const Y: usize> NeighborIter<X, Y> {
    fn new(
        center_point: MapPoint<X, Y>,
        initial_orientation: Compass,
        rotation_direction: bool,
        include_center: bool,
        include_corners: bool,
    ) -> Self {
        if initial_orientation.is_center() {
            panic!("line {}, need direction", line!());
        }
        if !include_corners && initial_orientation.is_ordinal() {
            panic!("line {}, need side direction", line!());
        }
        NeighborIter {
            include_center,
            include_corners,
            center_point,
            initial_orientation,
            current_orientation: initial_orientation,
            rotation_direction,
            finished: false,
        }
    }
    fn rotate_orientation(&mut self) {
        if self.include_center {
            self.include_center = false;
        } else if self.rotation_direction {
            // rotate clockwise
            self.current_orientation = if self.include_corners {
                self.current_orientation.clockwise()
            } else {
                self.current_orientation.clockwise().clockwise()
            };
            self.finished = self.current_orientation == self.initial_orientation;
        } else {
            // rotate counterclockwise
            self.current_orientation = if self.include_corners {
                self.current_orientation.counterclockwise()
            } else {
                self.current_orientation
                    .counterclockwise()
                    .counterclockwise()
            };
            self.finished = self.current_orientation == self.initial_orientation;
        }
    }
}

impl<const X: usize, const Y: usize> Iterator for NeighborIter<X, Y> {
    type Item = (MapPoint<X, Y>, Compass);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.finished {
            let result = if self.include_center {
                Some((self.center_point, Compass::Center))
            } else {
                self.center_point
                    .neighbor(self.current_orientation)
                    .map(|n| (n, self.current_orientation))
            };
            match result {
                Some(map_point) => {
                    self.rotate_orientation();
                    return Some(map_point);
                }
                None => self.rotate_orientation(),
            }
        }
        None
    }
}

struct OrientationIter<const X: usize, const Y: usize> {
    current_point: MapPoint<X, Y>,
    orientation: Compass,
    wrap_around: bool,
    offset: Compass,
    finished: bool,
}

impl<const X: usize, const Y: usize> OrientationIter<X, Y> {
    fn new(
        start_point: MapPoint<X, Y>,
        orientation: Compass,
        wrap_around: bool,
        offset: Compass,
    ) -> Self {
        if orientation.is_center() {
            panic!("line {}, need direction", line!());
        }
        if wrap_around && (orientation == offset || orientation == offset.flip()) {
            panic!("line {}, offset on same axis as orientation", line!());
        }
        OrientationIter {
            current_point: start_point,
            orientation,
            wrap_around,
            offset,
            finished: false,
        }
    }
    fn wrap_around(&mut self) {
        // using my_geometry rectangle and line to find wrap around point
        let i_current = Point::from(self.current_point);
        let delta = Point::from(self.orientation);
        let offset = Point::from(self.offset);
        let a = i_current.add(offset);
        let b = a.add(delta);
        let line = Line::from((a, b));
        // to find top_left and bottom_right of rectangle, map has to be mirror an x-axis
        let rectangle = Rectangle::new(MapPoint::<X, Y>::SW.into(), MapPoint::<X, Y>::NE.into());
        let rli: Vec<MapPoint<X, Y>> = rectangle
            .rectangle_line_intersection(&line)
            .iter()
            .filter_map(|p| MapPoint::<X, Y>::try_from(*p).ok())
            .collect();
        match rli.len() {
            0 => self.current_point = match self.current_point.map_position() {
                Compass::NW => MapPoint::<X, Y>::SE,
                Compass::NE => MapPoint::<X, Y>::SW,
                Compass::SW => MapPoint::<X, Y>::NE,
                Compass::SE => MapPoint::<X, Y>::NW,
                _ => panic!("line {}, wrap around fails to find new current_point while not being at cardinal point of map.", line!())
            },
            1 => self.current_point = rli[0],
            2 => {
                let orientation = self.orientation;
                let mut rli_iter = rli.iter().filter(|p| p.neighbor(orientation).is_some());
                self.current_point = *rli_iter.next().unwrap_or_else(|| panic!("line {}, wrap around fails to find neighbor in map at edge point", line!()));
                assert!(rli_iter.next().is_none());
            },
            _ => panic!("line {}, internal error. this should never happen.", line!()),
        }
    }
}

impl<const X: usize, const Y: usize> Iterator for OrientationIter<X, Y> {
    type Item = MapPoint<X, Y>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result = self.current_point;
        match self.current_point.neighbor(self.orientation) {
            Some(map_point) => self.current_point = map_point,
            None => {
                if self.wrap_around {
                    self.wrap_around();
                } else {
                    self.finished = true;
                }
            }
        }
        Some(result)
    }
}

struct EdgeIter<const X: usize, const Y: usize> {
    current_point: MapPoint<X, Y>,
    start_point: MapPoint<X, Y>,
    orientation: Compass,
    counterclockwise: bool,
    finished: bool,
}

impl<const X: usize, const Y: usize> EdgeIter<X, Y> {
    fn new(start_point: MapPoint<X, Y>, counterclockwise: bool) -> Self {
        let mut edge_iter = EdgeIter {
            current_point: start_point,
            start_point,
            orientation: start_point.map_position(),
            counterclockwise,
            finished: start_point.map_position().is_center(),
        };
        // initialize orientation
        edge_iter.orientation = match edge_iter.orientation {
            Compass::Center => Compass::Center,
            Compass::N | Compass::E | Compass::S | Compass::W => {
                if counterclockwise {
                    edge_iter.orientation.counterclockwise().counterclockwise()
                } else {
                    edge_iter.orientation.clockwise().clockwise()
                }
            }
            Compass::NE | Compass::NW | Compass::SE | Compass::SW => {
                if counterclockwise {
                    edge_iter
                        .orientation
                        .counterclockwise()
                        .counterclockwise()
                        .counterclockwise()
                } else {
                    edge_iter.orientation.clockwise().clockwise().clockwise()
                }
            }
        };
        edge_iter
    }
    fn turn_orientation(&mut self) {
        self.orientation = if self.counterclockwise {
            self.orientation.counterclockwise().counterclockwise()
        } else {
            self.orientation.clockwise().clockwise()
        };
    }
}

impl<const X: usize, const Y: usize> Iterator for EdgeIter<X, Y> {
    type Item = MapPoint<X, Y>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result = self.current_point;
        loop {
            match self.current_point.neighbor(self.orientation) {
                Some(map_point) => {
                    self.current_point = map_point;
                    break;
                }
                None => self.turn_orientation(),
            }
        }
        self.finished = self.current_point == self.start_point;
        Some(result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn iter_map_test() {
        const X: usize = 3;
        const Y: usize = 3;
        let mut point = MapPoint::<X, Y>::new(0, 0);
        assert_eq!(point, MapPoint::<X, Y>::new(0, 0));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(1, 0));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(2, 0));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(0, 1));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(1, 1));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(2, 1));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(0, 2));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(1, 2));
        point = point.forward_x().unwrap();
        assert_eq!(point, MapPoint::<X, Y>::new(2, 2));
        assert_eq!(point.forward_x(), None);
    }

    #[test]
    fn iter_wrap_around_test() {
        const X: usize = 20;
        const Y: usize = 10;
        eprintln!("start NW, orientation NE, offset S");
        let start = MapPoint::<X, Y>::NW;
        assert_eq!(
            start
                .iter_orientation_wrap_around(Compass::NE, Compass::S)
                .take_while(|p| *p != MapPoint::<X, Y>::SE)
                .count(),
            X * Y - 1
        );
        eprintln!("start NE, orientation NW, offset S");
        let start = MapPoint::<X, Y>::NE;
        assert_eq!(
            start
                .iter_orientation_wrap_around(Compass::NW, Compass::S)
                .take_while(|p| *p != MapPoint::<X, Y>::SW)
                .count(),
            X * Y - 1
        );
        eprintln!("start SW, orientation E, offset N");
        let start = MapPoint::<X, Y>::SW;
        assert_eq!(
            start
                .iter_orientation_wrap_around(Compass::E, Compass::N)
                .take_while(|p| *p != MapPoint::<X, Y>::NE)
                .count(),
            X * Y - 1
        );
        eprintln!("start NE, orientation S, offset W");
        let start = MapPoint::<X, Y>::NE;
        assert_eq!(
            start
                .iter_orientation_wrap_around(Compass::S, Compass::W)
                .take_while(|p| *p != MapPoint::<X, Y>::SW)
                .count(),
            X * Y - 1
        );
        eprintln!("start NW, orientation E, offset Center");
        let start = MapPoint::<X, Y>::NW;
        assert_eq!(
            start
                .iter_orientation_wrap_around(Compass::E, Compass::Center)
                .take_while(|p| *p != MapPoint::<X, Y>::NE)
                .count(),
            X - 1
        );
    }

    #[test]
    fn side_and_corner_test() {
        const X: usize = 20;
        const Y: usize = 10;
        let a = MapPoint::<X, Y>::new(0, 0);
        assert!(a.map_position().is_ordinal());
        let a = MapPoint::<X, Y>::new(19, 0);
        assert!(a.map_position().is_ordinal());
        let a = MapPoint::<X, Y>::new(0, 9);
        assert!(a.map_position().is_ordinal());
        let a = MapPoint::<X, Y>::new(19, 9);
        assert!(a.map_position().is_ordinal());
        let a = MapPoint::<X, Y>::new(0, 5);
        assert!(a.map_position().is_cardinal());
        let a = MapPoint::<X, Y>::new(19, 3);
        assert!(a.map_position().is_cardinal());
        let a = MapPoint::<X, Y>::new(7, 0);
        assert!(a.map_position().is_cardinal());
        let a = MapPoint::<X, Y>::new(18, 9);
        assert!(a.map_position().is_cardinal());
        let a = MapPoint::<X, Y>::new(18, 8);
        assert!(a.map_position().is_center());
    }
}
