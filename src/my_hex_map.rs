// The hex map is derived from codingame hex maps, where each hex is identified by a "map index".
// The orientation of hex is pointy-top. Map index of neighbors is given for each hex starting on the right side
// with neighbor index 0 counting counterclockwise around the hex up to 5. If a neighbor on one side does not exist,
// codingame returns -1. Since we index with usize in rust, we save indices of neighbors inside Option<usize>, with
// None indicating a not existing neighbor.

use crate::my_array::*;

pub type HexNeigh = [Option<usize>; 6];

#[derive(Copy, Clone, PartialEq)]
pub struct MyHexMap<T, const N: usize> {
    items: MyArray<T, N>,
    map: MyArray<HexNeigh, N>,
}

impl<T: Copy + Clone + Default, const N: usize> MyHexMap<T, N> {
    pub fn new() -> Self {
        Self {
            items: MyArray::default(),
            map: MyArray::default(),
        }
    }
    pub fn push(&mut self, item: T, neighbors: HexNeigh) {
        self.items.push(item);
        self.map.push(neighbors);
    }
    pub fn get_item(&mut self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
    pub fn get_mut_item(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)
    }
    pub fn iter_cells(&self) -> impl Iterator<Item = (usize, &T)> {
        self.items.iter().enumerate()
    }
    pub fn iter_neighbors(&self, index: usize) -> impl Iterator<Item = (usize, &T)> {
        self.map[index]
            .iter()
            .filter_map(|m| *m)
            .map(move |i| (i, self.items.get(i).unwrap()))
    }
    pub fn iter_distance<'a>(
        &'a self,
        start_hexes: MyArray<usize, N>,
        filter_fn: Box<dyn Fn(usize, &T, usize) -> bool>,
    ) -> impl Iterator<Item = (usize, &'a T, usize)> {
        // use filter_fn as follows (use "_" for unused variables):
        // let filter_fn = Box::new(|index_of_next_hex: usize, value_of_next_cell: &T, current_distance: usize| index_of_next_hex.use_it_somehow() || value_of_next_cell.use_it_somehow() || current_distance.use_it_somehow());
        HexDistanceIter::new(self, start_hexes, filter_fn)
    }
}

struct HexDistanceIter<'a, T, const N: usize> {
    data_hex_map: &'a MyHexMap<T, N>,
    filter_fn: Box<dyn Fn(usize, &T, usize) -> bool>, // input for filter_fn: index_of_next_hex, value_of_next_cell, distance of current hex
    next_hexes: MyArray<(usize, usize), N>,           // contains index and distance
    index: usize,
}

impl<'a, T: Copy + Clone, const N: usize> HexDistanceIter<'a, T, N> {
    fn new(
        data_hex_map: &'a MyHexMap<T, N>,
        start_hexes: MyArray<usize, N>,
        filter_fn: Box<dyn Fn(usize, &T, usize) -> bool>,
    ) -> Self {
        let mut next_hexes: MyArray<(usize, usize), N> = MyArray::default();
        for hex in start_hexes.iter() {
            next_hexes.push((*hex, 0));
        }
        HexDistanceIter {
            data_hex_map,
            filter_fn,
            next_hexes,
            index: 0,
        }
    }
}

impl<'a, T: Copy + Clone + Default, const N: usize> Iterator for HexDistanceIter<'a, T, N> {
    type Item = (usize, &'a T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.next_hexes.len() {
            return None;
        }
        let (map_index, distance) = self.next_hexes[self.index];
        let mut local_next_cells: MyArray<(usize, usize), 6> = MyArray::new();
        for (next_hex_index, _) in self
            .data_hex_map
            .iter_neighbors(map_index)
            .filter(|(p, c)| {
                self.next_hexes.iter().find(|(n, _)| n == p).is_none()
                    && (self.filter_fn)(*p, *c, distance)
            })
        {
            local_next_cells.push((next_hex_index, distance + 1));
        }
        self.next_hexes.append_slice(local_next_cells.as_slice());
        self.index += 1;
        Some((map_index, &self.data_hex_map.items[map_index], distance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::my_map_point::*;
    use crate::my_map_two_dim::*;
    // see C:\Users\Marc\Documents\Repos\basic_rust\projects\my_lib\example_hex_map.png
    const N: usize = 27;
    const M: usize = N * N;
    // neigh 0 to neigh 5
    const HEXMAP: &str = "1 3 -1 2 4 -1\n\
                          5 -1 3 0 -1 -1\n\
                          0 -1 -1 6 -1 4\n\
                          -1 -1 7 -1 0 1\n\
                          -1 0 2 -1 -1 8\n\
                          11 -1 -1 1 -1 18\n\
                          2 -1 17 12 -1 -1\n\
                          -1 -1 -1 9 -1 3\n\
                          10 -1 4 -1 -1 -1\n\
                          7 -1 -1 15 -1 -1\n\
                          16 -1 -1 8 -1 -1\n\
                          19 21 -1 5 18 26\n\
                          6 17 25 20 22 -1\n\
                          23 -1 -1 -1 -1 -1\n\
                          -1 -1 -1 24 -1 -1\n\
                          9 -1 -1 -1 17 -1\n\
                          -1 18 -1 10 -1 -1\n\
                          -1 15 -1 25 12 6\n\
                          26 11 5 -1 16 -1\n\
                          -1 -1 21 11 26 -1\n\
                          12 25 -1 -1 -1 22\n\
                          -1 -1 23 -1 11 19\n\
                          -1 12 20 -1 -1 24\n\
                          -1 -1 -1 13 -1 21\n\
                          14 -1 22 -1 -1 -1\n\
                          17 -1 -1 -1 20 12\n\
                          -1 19 11 18 -1 -1";
    fn read_hexmap() -> MyArray<HexNeigh, N> {
        let init_item: HexNeigh = [None; 6];
        let mut neighbors: MyArray<HexNeigh, N> = MyArray::init(init_item, N);
        let lines = HEXMAP.lines().map(|l| l.trim());
        for (index, line) in lines.enumerate() {
            for (i, neighbor) in line
                .split_whitespace()
                .map(|n| n.parse::<i32>().unwrap())
                .enumerate()
            {
                neighbors.get_mut(index).unwrap()[i] = if neighbor < 0 {
                    None
                } else {
                    Some(neighbor as usize)
                };
            }
        }
        neighbors
    }

    #[test]
    fn test_hex_distance() {
        let mut distance_map: MyMap2D<usize, N, N, M> = MyMap2D::default();
        let mut hex_map: MyHexMap<usize, N> = MyHexMap::new();
        let neighbors = read_hexmap();
        for (item, neighbor) in neighbors.iter().enumerate() {
            hex_map.push(item, *neighbor);
        }
        for start_hex in hex_map.items.iter() {
            let filter_fn = Box::new(|_: usize, _: &usize, _: usize| true);
            let start_hexes: MyArray<usize, N> = MyArray::init(*start_hex, 1);
            for (target_hex, _, distance) in hex_map.iter_distance(start_hexes, filter_fn) {
                distance_map.set(MapPoint::<N, N>::new(*start_hex, target_hex), distance);
            }
        }

        assert_eq!(*distance_map.get(MapPoint::<N, N>::new(9, 10)), 6);
        assert_eq!(*distance_map.get(MapPoint::<N, N>::new(19, 20)), 8);
        for x in 0..N {
            for y in 0..N {
                assert_eq!(
                    *distance_map.get(MapPoint::<N, N>::new(x, y)),
                    *distance_map.get(MapPoint::<N, N>::new(y, x))
                );
            }
        }
    }
}
