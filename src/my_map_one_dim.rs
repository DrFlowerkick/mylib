use crate::my_array::*;
use crate::my_compass::*;
use crate::my_map_point::*;

#[derive(Copy, Clone, PartialEq)]
pub struct MyMap1D<T, const X: usize, const Y: usize, const N: usize> {
    // Saving the map data of a 2-dim map in a 1-dim array saves a substantial amount of compilation time at the
    // cost of a small overhead in run time to calculate the indizes of map into an index and vice versa.
    // X: size of dimension x
    // Y: size of dimension Y
    // N: total Numer if items in Map. Since N must be constant in type definition, you must give the result of X*Y manually as value of N
    items: MyArray<T, N>,
}

impl<T: Copy + Clone + Default, const X: usize, const Y: usize, const N: usize>
    MyMap1D<T, X, Y, N>
{
    fn coordinates_to_index(coordinates: MapPoint<X, Y>) -> usize {
        coordinates.x() + coordinates.y() * X
    }
    fn index_to_coordinates(index: usize) -> MapPoint<X, Y> {
        MapPoint::new(index % X, index / X)
    }
    pub fn new() -> Self {
        if X == 0 {
            panic!("line {}, minimum one column", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum one row", line!());
        }
        if X * Y != N {
            panic!(
                "line {}, number of elements does not fit to X and Y",
                line!()
            );
        }
        Self {
            items: MyArray::init(T::default(), N),
        }
    }
    pub fn init(init_item: T) -> Self {
        if X == 0 {
            panic!("line {}, minimum one column", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum one row", line!());
        }
        if X * Y != N {
            panic!(
                "line {}, number of elements does not fit to X and Y",
                line!()
            );
        }
        Self {
            items: MyArray::init(init_item, N),
        }
    }
    pub fn get(&self, coordinates: MapPoint<X, Y>) -> &T {
        self.items
            .get(MyMap1D::<T, X, Y, N>::coordinates_to_index(coordinates))
            .unwrap() // if coordinates are out of index, this will throw an error
    }
    pub fn get_mut(&mut self, coordinates: MapPoint<X, Y>) -> &mut T {
        self.items
            .get_mut(MyMap1D::<T, X, Y, N>::coordinates_to_index(coordinates))
            .unwrap() // if coordinates are out of index, this will throw an error
    }
    pub fn set(&mut self, coordinates: MapPoint<X, Y>, item: T) -> &T {
        self.items
            .set(
                MyMap1D::<T, X, Y, N>::coordinates_to_index(coordinates),
                item,
            )
            .unwrap() // if coordinates are out of index, this will throw an error
    }
    pub fn is_cut_off_cell(
        &self,
        map_point: MapPoint<X, Y>,
        is_cell_free_fn: Box<dyn Fn(MapPoint<X, Y>, &T) -> bool>,
    ) -> bool {
        // use is_cell_free_fn as follows (use "_" for unused variables):
        // let is_cell_free_fn = Box::new(|current_point: MapPoint<X, Y>, current_cell_value: &T| current_point.use_it_somehow() || current_cell_value.use_it_somehow() );
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
            if !last_free {
                if is_free && is_side {
                    // new free zones start always at a side of map_point, since movement over cornes is not allowed
                    free_zones += 1;
                }
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
        self.items
            .iter()
            .enumerate()
            .map(|(i, t)| (MyMap1D::<T, X, Y, N>::index_to_coordinates(i), t))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (MapPoint<X, Y>, &mut T)> {
        self.items
            .iter_mut()
            .enumerate()
            .map(|(i, t)| (MyMap1D::<T, X, Y, N>::index_to_coordinates(i), t))
    }
    pub fn iter_row(&self, r: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        if r >= Y {
            panic!("line {}, row index is out of range", line!());
        }
        self.items
            .iter()
            .enumerate()
            .filter(move |(i, _)| i / X == r)
            .map(|(i, t)| (MyMap1D::<T, X, Y, N>::index_to_coordinates(i), t))
    }
    pub fn iter_column(&self, c: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        if c >= X {
            panic!("line {}, column index is out of range", line!());
        }
        self.items
            .iter()
            .enumerate()
            .filter(move |(i, _)| i % X == c)
            .map(|(i, t)| (MyMap1D::<T, X, Y, N>::index_to_coordinates(i), t))
    }
    pub fn iter_neighbors(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        center_point
            .iter_neighbors(Compass::N, true, false, false)
            .map(move |(p, _)| (p, self.get(p)))
    }
    pub fn iter_neighbors_mut(
        &mut self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &mut T)> {
        center_point
            .iter_neighbors(Compass::N, true, false, false)
            .map(move |(p, _)| unsafe { (p, &mut *(self.get_mut(p) as *mut _)) })
    }
    pub fn iter_neighbors_with_center(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        center_point
            .iter_neighbors(Compass::N, true, true, false)
            .map(move |(p, _)| (p, self.get(p)))
    }
    pub fn iter_neighbors_with_corners(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T, bool)> {
        center_point
            .iter_neighbors(Compass::N, true, false, true)
            .map(move |(p, o)| (p, self.get(p), o.is_ordinal()))
    }
    pub fn iter_neighbors_with_center_and_corners(
        &self,
        center_point: MapPoint<X, Y>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &T, bool)> {
        center_point
            .iter_neighbors(Compass::N, true, true, true)
            .map(move |(p, o)| (p, self.get(p), o.is_ordinal()))
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
    pub fn iter_diagonale_top_left(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(0, 0)
            .iter_orientation(Compass::SE)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_diagonale_top_right(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(X - 1, 0)
            .iter_orientation(Compass::SW)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_diagonale_bottom_left(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(0, Y - 1)
            .iter_orientation(Compass::NE)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_diagonale_bottom_right(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(X - 1, Y - 1)
            .iter_orientation(Compass::NW)
            .map(move |p| (p, self.get(p)))
    }
    pub fn iter_distance<'a>(
        &'a self,
        start_point: MapPoint<X, Y>,
        filter_fn: Box<dyn Fn(MapPoint<X, Y>, &T, usize) -> bool>,
    ) -> impl Iterator<Item = (MapPoint<X, Y>, &'a T, usize)> {
        // use filter_fn as follows (use "_" for unused variables):
        // let filter_fn = Box::new(|point_of_next_cell: MapPoint<X, Y>, value_of_next_cell: &T, current_distance: usize| current_point.use_it_somehow() || current_cell_value.use_it_somehow() || current_distance.use_it_somehow());
        DistanceIter::new(self, start_point, filter_fn)
    }
}

impl<T: Copy + Clone + Default, const X: usize, const Y: usize, const N: usize> Default
    for MyMap1D<T, X, Y, N>
{
    fn default() -> Self {
        Self::new()
    }
}

struct DistanceIter<'a, T, const X: usize, const Y: usize, const N: usize> {
    data_map: &'a MyMap1D<T, X, Y, N>,
    filter_fn: Box<dyn Fn(MapPoint<X, Y>, &T, usize) -> bool>, // input for filter_fn: next possible point, data from data_map of next possible point, distance of current point
    next_cells: MyArray<(MapPoint<X, Y>, usize), N>,
    index: usize,
}

impl<'a, T: Copy + Clone, const X: usize, const Y: usize, const N: usize>
    DistanceIter<'a, T, X, Y, N>
{
    fn new(
        data_map: &'a MyMap1D<T, X, Y, N>,
        start_point: MapPoint<X, Y>,
        filter_fn: Box<dyn Fn(MapPoint<X, Y>, &T, usize) -> bool>,
    ) -> Self {
        DistanceIter {
            data_map,
            filter_fn,
            next_cells: MyArray::init((start_point, 0), 1),
            index: 0,
        }
    }
}

impl<'a, T: Copy + Clone + Default, const X: usize, const Y: usize, const N: usize> Iterator
    for DistanceIter<'a, T, X, Y, N>
{
    type Item = (MapPoint<X, Y>, &'a T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.next_cells.len() {
            return None;
        }
        let (map_point, distance) = *self.next_cells.get(self.index).unwrap();
        let mut local_next_cells: MyArray<(MapPoint<X, Y>, usize), 4> = MyArray::new();
        for (next_cell, _) in self.data_map.iter_neighbors(map_point).filter(|(p, c)| {
            self.next_cells.iter().find(|(n, _)| n == p).is_none()
                && (self.filter_fn)(*p, *c, distance)
        }) {
            local_next_cells.push((next_cell, distance + 1));
        }
        self.next_cells.append_slice(local_next_cells.as_slice());
        self.index += 1;
        Some((map_point, self.data_map.get(map_point), distance))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn iter_map_test() {
        const X: usize = 3;
        const Y: usize = 3;
        const N: usize = X * Y;
        let zeros: MyMap1D<i32, X, Y, N> = MyMap1D::new();

        let mut zeros_iter = zeros.iter();
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(0, 0));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(1, 0));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(2, 0));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(0, 1));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(1, 1));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(2, 1));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(0, 2));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(1, 2));
        assert_eq!(zeros_iter.next().unwrap().0, MapPoint::<X, Y>::new(2, 2));
        assert_eq!(zeros_iter.next(), None);
    }

    #[test]
    fn iter_diagonale_test() {
        const X: usize = 4;
        const Y: usize = 7;
        const N: usize = X * Y;

        let mut zeros: MyMap1D<i32, X, Y, N> = MyMap1D::new();
        let mut ones: MyMap1D<i32, X, Y, N> = MyMap1D::init(1);

        for (point, value) in ones.iter_orientation(MapPoint::<X, Y>::new(2, 0), Compass::SW) {
            zeros.set(point, *value);
        }

        assert_eq!(zeros.iter().map(|(_, v)| *v).sum::<i32>(), 3);
        assert_eq!(*zeros.get(MapPoint::<X, Y>::new(2, 0)), 1);
        assert_eq!(*zeros.get(MapPoint::<X, Y>::new(1, 1)), 1);
        assert_eq!(*zeros.get(MapPoint::<X, Y>::new(0, 2)), 1);

        for (point, value) in zeros.iter_diagonale_top_left() {
            ones.set(point, *value + 1);
        }
        assert_eq!(*ones.get(MapPoint::<X, Y>::new(0, 0)), 1);
        assert_eq!(*ones.get(MapPoint::<X, Y>::new(1, 1)), 2);
        assert_eq!(*ones.get(MapPoint::<X, Y>::new(2, 2)), 1);
    }

    #[test]
    fn iter_diagonale_test_3x3() {
        const X: usize = 3;
        const Y: usize = X;
        const N: usize = X * Y;

        let zeros: MyMap1D<i32, X, Y, N> = MyMap1D::new();

        let mut pos_diag = zeros.iter_diagonale_top_right();
        assert_eq!(pos_diag.next().unwrap().0, MapPoint::<X, Y>::new(2, 0));
        assert_eq!(pos_diag.next().unwrap().0, MapPoint::<X, Y>::new(1, 1));
        assert_eq!(pos_diag.next().unwrap().0, MapPoint::<X, Y>::new(0, 2));
        assert_eq!(pos_diag.next(), None);

        let mut neg_diag = zeros.iter_diagonale_top_left();
        assert_eq!(neg_diag.next().unwrap().0, MapPoint::<X, Y>::new(0, 0));
        assert_eq!(neg_diag.next().unwrap().0, MapPoint::<X, Y>::new(1, 1));
        assert_eq!(neg_diag.next().unwrap().0, MapPoint::<X, Y>::new(2, 2));
        assert_eq!(neg_diag.next(), None);
    }
}
