use std::fmt::Display;

use crate::my_array::*;
use crate::my_compass::*;
use crate::my_map_point::*;

/* use filter_fn as follows (use "_" for unused variables):
        let filter_fn = Box::new(|
            point_of_next_cell: MapPoint<X, Y>,
            value_of_next_cell: &T,
            orientation_of_next_cell: Compass,
            current_point: MapPoint<X, Y>,
            value_of_current_cell: &T,
            current_distance: usize| {
                point_of_next_cell.use_it_somehow() &&
                value_of_next_cell.use_it_somehow() &&
                orientation_of_next_cell.use_it_somehow() &&
                current_point.use_it_somehow() &&
                value_of_current_cell.use_it_somehow() &&
                current_distance.use_it_somehow()
        });
*/
pub type FilterFn<T, const X: usize, const Y: usize> =
    Box<dyn Fn(MapPoint<X, Y>, &T, Compass, MapPoint<X, Y>, &T, usize) -> bool>;

// use is_cell_free_fn as follows (use "_" for unused variables):
// let is_cell_free_fn = Box::new(|current_point: MapPoint<X, Y>, current_cell_value: &T| current_point.use_it_somehow() || current_cell_value.use_it_somehow() );
pub type IsCellFreeFn<T, const X: usize, const Y: usize> = Box<dyn Fn(MapPoint<X, Y>, &T) -> bool>;

// use MyMap2D if compilation time is suffice, because it is more efficient and has cleaner interface
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MyMap2D<T, const X: usize, const Y: usize> {
    // X: number of columns, Y: number of rows
    items: [[T; X]; Y], //outer array rows, inner array columns -> first index chooses row (y), second index chooses column (x)
}

impl<T: Copy + Clone + Default + From<char>, const X: usize, const Y: usize> From<&str>
    for MyMap2D<T, X, Y>
{
    fn from(value: &str) -> Self {
        let mut map = MyMap2D::default();
        for (y, line) in value.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                map.set(MapPoint::<X, Y>::new(x, y), T::from(c));
            }
        }
        map
    }
}

impl<T: Copy + Clone + Default + Display, const X: usize, const Y: usize> Display
    for MyMap2D<T, X, Y>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::new();
        for (p, v) in self.iter() {
            line = format!("{}{}", line, v);
            if (p.x() + 1) % X == 0 && !line.is_empty() {
                writeln!(f, "{}", line)?;
                line = "".into();
            }
        }
        Ok(())
    }
}

impl<T: Copy + Clone + Default, const X: usize, const Y: usize> MyMap2D<T, X, Y> {
    pub fn new() -> Self {
        if X == 0 {
            panic!("line {}, minimum one column", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum one row", line!());
        }
        Self {
            items: [[T::default(); X]; Y],
        }
    }
    pub fn init(init_element: T) -> Self {
        if X == 0 {
            panic!("line {}, minimum one column", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum one row", line!());
        }
        Self {
            items: [[init_element; X]; Y],
        }
    }
    pub fn get(&self, coordinates: MapPoint<X, Y>) -> &T {
        &self.items[coordinates.y()][coordinates.x()]
    }
    pub fn get_mut(&mut self, coordinates: MapPoint<X, Y>) -> &mut T {
        &mut self.items[coordinates.y()][coordinates.x()]
    }
    pub fn set(&mut self, coordinates: MapPoint<X, Y>, value: T) -> &T {
        self.items[coordinates.y()][coordinates.x()] = value;
        &self.items[coordinates.y()][coordinates.x()]
    }
    pub fn swap_value(&mut self, coordinates: MapPoint<X, Y>, value: T) -> T {
        let old_value = self.items[coordinates.y()][coordinates.x()];
        self.items[coordinates.y()][coordinates.x()] = value;
        old_value
    }
    pub fn swap_cell_values(&mut self, cell_1: MapPoint<X, Y>, cell_2: MapPoint<X, Y>) {
        if cell_1 == cell_2 {
            return;
        }
        unsafe {
            // this is safe, since cell_1 != cell_2 and both always point to legit items of self
            let p1: *mut T = self.get_mut(cell_1);
            let p2: *mut T = self.get_mut(cell_2);
            std::ptr::swap(p1, p2);
        }
    }
    pub fn get_row(&self, row: usize) -> &[T] {
        if row >= Y {
            panic!("line {}, row out of range", line!());
        }
        &self.items[row][..]
    }
    pub fn get_row_mut(&mut self, row: usize) -> &mut [T] {
        if row >= Y {
            panic!("line {}, row out of range", line!());
        }
        &mut self.items[row][..]
    }
    // Since you cannot get a slice on a column with indexing (as you can with a row, since a row is an array of [T; X]),
    // you have to work around it by returning a column as copy of values
    pub fn get_column(&self, col: usize) -> [T; Y] {
        if col >= X {
            panic!("line {}, column out of range", line!());
        }
        let mut column = [T::default(); Y];
        self.iter_column(col).for_each(|(p, v)| column[p.y()] = *v);
        column
    }
    // If you modify values of a column outside of MyMap2D, you can use this function to apply these new values
    pub fn apply_column(&mut self, col: usize, column: [T; Y]) {
        if col >= X {
            panic!("line {}, column out of range", line!());
        }
        self.iter_column_mut(col)
            .for_each(|(p, v)| *v = column[p.y()]);
    }
    pub fn is_cut_off_cell(
        &self,
        map_point: MapPoint<X, Y>,
        is_cell_free_fn: IsCellFreeFn<T, X, Y>,
    ) -> bool {
        let (mut last_free, initial_orientation) = match map_point.map_position() {
            Compass::NW | Compass::N => (false, Compass::E),
            Compass::NE | Compass::E => (false, Compass::S),
            Compass::SE | Compass::S => (false, Compass::W),
            Compass::SW | Compass::W => (false, Compass::N),
            Compass::Center => {
                let nw = map_point.neighbor(Compass::NW).unwrap();
                (is_cell_free_fn(nw, self.get(nw)), Compass::N)
            }
        };
        let mut free_zones = 0;
        for (is_free, is_side) in map_point
            .iter_neighbors(initial_orientation, true, false, true)
            .map(|(p, o)| (is_cell_free_fn(p, self.get(p)), o.is_cardinal()))
        {
            if !last_free && is_free && is_side {
                // new free zones start always at a side of map_point, since movement over corners is not allowed
                free_zones += 1;
            }
            last_free = if is_side || !is_free {
                // side or blocked corner -> apply is_free to last_free
                is_free
            } else {
                // free corner -> keep old value of last_free
                last_free
            };
        }
        free_zones > 1
    }
    pub fn iter(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        self.items.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, column)| (MapPoint::<X, Y>::new(x, y), column))
        })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (MapPoint<X, Y>, &mut T)> {
        self.items.iter_mut().enumerate().flat_map(|(y, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(x, column)| (MapPoint::<X, Y>::new(x, y), column))
        })
    }
    pub fn iter_row(&self, r: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        // dimension check of r is done in get_row()
        self.get_row(r)
            .iter()
            .enumerate()
            .map(move |(x, column)| (MapPoint::new(x, r), column))
    }
    pub fn iter_row_mut(&mut self, r: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &mut T)> {
        // dimension check of r is done in get_row_mut()
        self.get_row_mut(r)
            .iter_mut()
            .enumerate()
            .map(move |(x, column)| (MapPoint::new(x, r), column))
    }
    pub fn iter_column(&self, c: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        if c >= X {
            panic!("line {}, column index is out of range", line!());
        }
        self.items.iter().enumerate().flat_map(move |(y, row)| {
            row.iter()
                .enumerate()
                .filter(move |(x, _)| *x == c)
                .map(move |(x, column)| (MapPoint::new(x, y), column))
        })
    }
    pub fn iter_column_mut(&mut self, c: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &mut T)> {
        if c >= X {
            panic!("line {}, column index is out of range", line!());
        }
        self.items.iter_mut().enumerate().flat_map(move |(y, row)| {
            row.iter_mut()
                .enumerate()
                .filter(move |(x, _)| *x == c)
                .map(move |(x, column)| (MapPoint::new(x, y), column))
        })
    }
    pub fn iter_edge(
        &self,
        start_point: MapPoint<X, Y>,
        counterclockwise: bool,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        start_point
            .iter_edge(counterclockwise)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_neighbors(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, Compass, &T)> {
        center_point
            .iter_neighbors(Compass::N, true, false, false)
            .map(move |(p, o)| (p, o, self.get(p)))
    }
    pub fn iter_neighbors_mut(
        &mut self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, Compass, &mut T)> {
        center_point
            .iter_neighbors(Compass::N, true, false, false)
            .map(move |(p, o)| unsafe { (p, o, &mut *(self.get_mut(p) as *mut _)) })
    }
    pub fn iter_neighbors_with_center(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, Compass, &T)> {
        center_point
            .iter_neighbors(Compass::N, true, true, false)
            .map(move |(p, o)| (p, o, self.get(p)))
    }
    pub fn iter_neighbors_with_corners(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, Compass, &T)> {
        center_point
            .iter_neighbors(Compass::N, true, false, true)
            .map(move |(p, o)| (p, o, self.get(p)))
    }
    pub fn iter_neighbors_with_center_and_corners(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, Compass, &T)> {
        center_point
            .iter_neighbors(Compass::N, true, true, true)
            .map(move |(p, o)| (p, o, self.get(p)))
    }
    pub fn iter_orientation(
        &self,
        start_point: MapPoint<X, Y>,
        orientation: Compass,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        start_point
            .iter_orientation(orientation)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_diagonal_top_left(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(0, 0)
            .iter_orientation(Compass::SE)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_diagonal_top_right(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(X - 1, 0)
            .iter_orientation(Compass::SW)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_diagonal_bottom_left(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(0, Y - 1)
            .iter_orientation(Compass::NE)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_diagonal_bottom_right(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(X - 1, Y - 1)
            .iter_orientation(Compass::NW)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_distance(
        &self,
        start_point: MapPoint<X, Y>,
        filter_fn: FilterFn<T, X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T, usize)> {
        let start_points: Vec<MapPoint<X, Y>> = vec![start_point];
        DistanceIter::new(self, start_points, filter_fn)
    }
    pub fn iter_distance_area(
        &self,
        start_points: Vec<MapPoint<X, Y>>,
        filter_fn: FilterFn<T, X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T, usize)> {
        DistanceIter::new(self, start_points, filter_fn)
    }
}

impl<T: Copy + Clone + Default, const X: usize, const Y: usize> Default for MyMap2D<T, X, Y> {
    fn default() -> Self {
        Self::new()
    }
}

struct DistanceIter<'a, T, const X: usize, const Y: usize> {
    data_map: &'a MyMap2D<T, X, Y>,
    // input for filter_fn in stated order:
    // MapPoint<X, Y>: next possible point
    // &T: data from data_map of next possible point
    // Compass: orientation of next possible point from current point
    // MapPoint<X, Y>: current point,
    // &T: value of current point
    // usize: distance of current point to start_points
    filter_fn: FilterFn<T, X, Y>,
    next_cells: Vec<(MapPoint<X, Y>, usize)>,
    index: usize,
}

impl<'a, T: Copy + Clone, const X: usize, const Y: usize> DistanceIter<'a, T, X, Y> {
    fn new(
        data_map: &'a MyMap2D<T, X, Y>,
        start_points: Vec<MapPoint<X, Y>>,
        filter_fn: FilterFn<T, X, Y>,
    ) -> Self {
        let mut result = DistanceIter {
            data_map,
            filter_fn,
            next_cells: Vec::with_capacity((X + Y) * 2),
            index: 0,
        };
        for sp in start_points.iter() {
            result.next_cells.push((*sp, 0));
        }
        result
    }
}

impl<'a, T: Copy + Clone + Default, const X: usize, const Y: usize> Iterator
    for DistanceIter<'a, T, X, Y>
{
    type Item = (MapPoint<X, Y>, &'a T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.next_cells.len() || self.next_cells.is_empty() {
            return None;
        }
        let (map_point, distance) = *self.next_cells.get(self.index).unwrap();
        let mut local_next_cells: MyArray<(MapPoint<X, Y>, usize), 4> = MyArray::new();
        for (next_cell, ..) in self.data_map.iter_neighbors(map_point).filter(|(p, o, c)| {
            !self.next_cells.iter().any(|(n, _)| n == p)
                && (self.filter_fn)(
                    *p,
                    *c,
                    *o,
                    map_point,
                    self.data_map.get(map_point),
                    distance,
                )
        }) {
            local_next_cells.push((next_cell, distance + 1));
        }
        self.next_cells
            .extend_from_slice(local_next_cells.as_slice());
        self.index += 1;
        Some((map_point, self.data_map.get(map_point), distance))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cut_off() {
        const X: usize = 20;
        const Y: usize = 10;

        let mut cut_off_map: MyMap2D<bool, X, Y> = MyMap2D::new();
        let mut game_map: MyMap2D<bool, X, Y> = MyMap2D::init(true);
        game_map.set(MapPoint::<X, Y>::new(1, 1), false);
        for (p, _) in game_map.iter().filter(|(_, c)| **c) {
            let is_cell_free_fn = Box::new(|_: MapPoint<X, Y>, c: &bool| *c);
            *cut_off_map.get_mut(p) = game_map.is_cut_off_cell(p, is_cell_free_fn);
        }
        assert_eq!(cut_off_map.iter().filter(|(_, c)| **c == true).count(), 5);

        game_map.set(MapPoint::<X, Y>::new(8, 2), false);
        for (p, _) in game_map.iter().filter(|(_, c)| **c) {
            let is_cell_free_fn = Box::new(|_: MapPoint<X, Y>, c: &bool| *c);
            *cut_off_map.get_mut(p) = game_map.is_cut_off_cell(p, is_cell_free_fn);
        }
        assert_eq!(cut_off_map.iter().filter(|(_, c)| **c == true).count(), 5);

        game_map.set(MapPoint::<X, Y>::new(7, 4), false);
        for (p, _) in game_map.iter().filter(|(_, c)| **c) {
            let is_cell_free_fn = Box::new(|_: MapPoint<X, Y>, c: &bool| *c);
            *cut_off_map.get_mut(p) = game_map.is_cut_off_cell(p, is_cell_free_fn);
        }
        assert_eq!(cut_off_map.iter().filter(|(_, c)| **c == true).count(), 7);

        game_map.set(MapPoint::<X, Y>::new(7, 6), false);
        for (p, _) in game_map.iter().filter(|(_, c)| **c) {
            let is_cell_free_fn = Box::new(|_: MapPoint<X, Y>, c: &bool| *c);
            *cut_off_map.get_mut(p) = game_map.is_cut_off_cell(p, is_cell_free_fn);
        }
        assert_eq!(cut_off_map.iter().filter(|(_, c)| **c == true).count(), 10);

        game_map.set(MapPoint::<X, Y>::new(9, 8), false);
        for (p, _) in game_map.iter().filter(|(_, c)| **c) {
            let is_cell_free_fn = Box::new(|_: MapPoint<X, Y>, c: &bool| *c);
            *cut_off_map.get_mut(p) = game_map.is_cut_off_cell(p, is_cell_free_fn);
        }
        assert_eq!(cut_off_map.iter().filter(|(_, c)| **c == true).count(), 14);
        assert!(*cut_off_map.get(MapPoint::<X, Y>::new(8, 7)));
    }

    #[test]
    fn test_get_column() {
        const X: usize = 20;
        const Y: usize = 10;
        let mut map: MyMap2D<usize, X, Y> = MyMap2D::default();
        let mut count = 0;
        for y in 0..Y {
            for x in 0..X {
                map.set((x, y).into(), count);
                count += 1;
            }
        }
        eprintln!("{}", map);
        for col in 0..X {
            eprint!("col {:02}: ", col);
            for cell in map.get_column(col).iter() {
                eprint!("{:03} ", cell);
            }
            eprintln!("");
        }
    }
}
