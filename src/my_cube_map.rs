// CubeMap is a six sided cube with 2-Dim Maps on each surface.
// As input to create a CubeMap a flat_map_str is used.
// A flat_map_str is constructed by unfolding the cube at it's edges.
// There are a lot of different cube unfolding patterns, but all patterns fit into a 3x4 or 4x3 map,
// where 6 tiles of this map represent each a cube surface and the other 6 map tiles are empty.
// Therefore each char of the flat_map_str represent either an element of a 2-Dim Map or
// empty elements of the empty map tiles of the unfolding pattern.

use crate::my_compass::Compass;
use crate::my_geometry::my_point::{Point, Point3D};
use crate::my_map_point::MapPoint;
use crate::my_map_two_dim::MyMap2D;
use std::collections::HashMap;
use std::fmt::Debug;

pub type CubeMapPoint<const N: usize> = (usize, MapPoint<N, N>);

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CubeMap<T: Copy + Clone + Default + From<char> + Debug, const N: usize> {
    surfaces: [MyMap2D<T, N, N>; 6],
    flat_map_surfaces: HashMap<Point, usize>,
    edges: HashMap<(usize, Compass), (usize, Compass)>,
}

impl<T: Copy + Clone + Default + From<char> + Debug, const N: usize> From<&str> for CubeMap<T, N> {
    fn from(value: &str) -> Self {
        Self::new_from_flat_map_str(value)
    }
}

impl<T: Copy + Clone + Default + From<char> + Debug, const N: usize> CubeMap<T, N> {
    pub fn new_from_flat_map_str(flat_map: &str) -> Self {
        let mut cj = CubeMap::<T, N>::default();
        // read in surfaces
        let n_chars_in_line = flat_map.lines().map(|l| l.chars().count()).max().unwrap();
        let n_lines = flat_map.lines().count();
        assert!(n_chars_in_line % N == 0);
        assert!(n_lines % N == 0);
        let x_blocks = n_chars_in_line / N;
        let y_blocks = n_lines / N;
        assert_eq!(x_blocks.min(y_blocks), 3);
        assert_eq!(x_blocks.max(y_blocks), 4);
        let mut index_surface = 0;
        for iy in 0..y_blocks {
            for ix in 0..x_blocks {
                let offset_x = ix * N;
                let offset_y = iy * N;
                if let Some(tile) = flat_map
                    .lines()
                    .nth(offset_y)
                    .unwrap()
                    .chars()
                    .nth(offset_x)
                {
                    if tile.is_whitespace() {
                        continue;
                    }
                    let mut surface: MyMap2D<T, N, N> = MyMap2D::default();
                    for (y, line) in flat_map.lines().skip(offset_y).take(N).enumerate() {
                        for (x, tile) in line.chars().skip(offset_x).take(N).enumerate() {
                            assert!(['.', '#'].contains(&tile));
                            *surface.get_mut((x, y).into()) = T::from(tile);
                        }
                    }
                    cj.surfaces[index_surface] = surface;
                    cj.flat_map_surfaces
                        .insert(Point::new(ix as i64, iy as i64), index_surface);
                    index_surface += 1;
                }
            }
        }
        assert_eq!(index_surface, 6);
        // set edge relations from surface block positions
        for (block, si) in cj.flat_map_surfaces.iter() {
            for dir in [Compass::N, Compass::E, Compass::S, Compass::W] {
                if let Some(neighbor_si) = cj.flat_map_surfaces.get(&block.add(dir.into())) {
                    cj.edges.insert((*si, dir), (*neighbor_si, dir.flip()));
                }
            }
        }
        // set cube corner coordinates for each surface
        let mut corners: HashMap<(usize, Compass), Point3D> = HashMap::new();
        corners.insert((0, Compass::NW), (0, 0, 0).into());
        corners.insert((0, Compass::NE), (1, 0, 0).into());
        corners.insert((0, Compass::SW), (0, 1, 0).into());
        corners.insert((0, Compass::SE), (1, 1, 0).into());
        let mut surfaces: Vec<usize> = vec![0];
        let mut index = 0;
        while index < surfaces.len() {
            let current_surface = surfaces[index];
            for ((_, cdir), (ni, ndir)) in cj
                .edges
                .iter()
                .filter(|((si, _), _)| *si == current_surface)
            {
                if corners.keys().any(|(i, _)| i == ni) {
                    // surface corners are already in corners
                    continue;
                }
                assert_eq!(
                    corners
                        .keys()
                        .filter(|(i, _)| *i == current_surface)
                        .count(),
                    4
                );
                surfaces.push(*ni);
                // add connected corners
                let cdir_corners = cdir.get_ordinals().unwrap();
                let ndir_corners = ndir.get_ordinals().unwrap();
                for (cdir_corner, ndir_corner) in cdir_corners.iter().zip(ndir_corners.iter()) {
                    let corner = *corners.get(&(current_surface, *cdir_corner)).unwrap();
                    corners.insert((*ni, *ndir_corner), corner);
                    // a_vector: get corner 2x compass clockwise and substrat corner
                    let a_vector = corners
                        .get(&(current_surface, cdir_corner.clockwise().clockwise()))
                        .unwrap()
                        .substract(&corner);
                    // b_vector: get corner 2x compass counterclockwise and substrat corner
                    let b_vector = corners
                        .get(&(
                            current_surface,
                            cdir_corner.counterclockwise().counterclockwise(),
                        ))
                        .unwrap()
                        .substract(&corner);
                    // x-product: a x b + corner is second corner to add
                    let far_corner = a_vector.cross_product(&b_vector).add(&corner);
                    corners.insert((*ni, *cdir_corner), far_corner);
                }
            }
            index += 1;
        }
        // each surface has 4 cube corners
        for s in 0..6 {
            assert_eq!(corners.iter().filter(|((i, _), _)| *i == s).count(), 4);
        }

        // add remaining edge relations from cube corner coordinates
        for i in 0..6 {
            let missing_edges: Vec<Compass> = [Compass::N, Compass::E, Compass::S, Compass::W]
                .into_iter()
                .filter(|c| !cj.edges.contains_key(&(i, *c)))
                .collect();
            for missing_edge in missing_edges {
                let key = (i, missing_edge);
                let imec = missing_edge.get_ordinals().unwrap();
                let cube_corners: Vec<Point3D> = imec
                    .iter()
                    .map(|c| *corners.get(&(i, *c)).unwrap())
                    .collect();
                let edge_value = corners
                    .iter()
                    .filter(|((s, _), c)| *s != i && **c == cube_corners[0])
                    .filter_map(|((o1, o1c), _)| {
                        corners
                            .iter()
                            .find(|((o2, _), c)| o1 == o2 && **c == cube_corners[1])
                            .map(|((o, o2c), _)| (*o, o1c.get_cardinal(o2c).unwrap()))
                    })
                    .next()
                    .unwrap();
                cj.edges.insert(key, edge_value);
                // edge works in both directions
                assert!(!cj.edges.contains_key(&edge_value));
                cj.edges.insert(edge_value, key);
            }
        }
        assert_eq!(cj.edges.len(), 24);
        for edge in cj.edges.values() {
            // every key is also an edge_value and vice versa
            assert!(cj.edges.contains_key(edge));
        }
        // return cj
        cj
    }
    pub fn get_surface(&self, index: usize) -> &MyMap2D<T, N, N> {
        assert!(index < 6);
        &self.surfaces[index]
    }
    pub fn get_cube_map_point_value(&self, cube_map_point: &CubeMapPoint<N>) -> Option<&T> {
        if cube_map_point.0 < 6 {
            return None;
        }
        Some(self.surfaces[cube_map_point.0].get(cube_map_point.1))
    }
    pub fn cube_map_point_to_flat_map_coordinates(
        &self,
        cube_map_point: &CubeMapPoint<N>,
    ) -> Option<(usize, usize)> {
        if cube_map_point.0 >= 6 {
            return None;
        }
        let (flat_map_index, _) = self
            .flat_map_surfaces
            .iter()
            .find(|(_, s)| **s == cube_map_point.0)
            .unwrap();
        Some((
            cube_map_point.1.x() + N * flat_map_index.x as usize,
            cube_map_point.1.y() + N * flat_map_index.y as usize,
        ))
    }
    pub fn flat_map_coordinates_to_cube_map_point(
        &self,
        flat_map_coordinates: (usize, usize),
    ) -> Option<CubeMapPoint<N>> {
        let flat_map_index_x = (flat_map_coordinates.0 / N) as i64;
        let flat_map_index_y = (flat_map_coordinates.1 / N) as i64;
        if let Some(surface_index) = self
            .flat_map_surfaces
            .get(&(flat_map_index_x, flat_map_index_y).into())
        {
            let surface_x = flat_map_coordinates.0 % N;
            let surface_y = flat_map_coordinates.1 % N;
            return Some((*surface_index, (surface_x, surface_y).into()));
        }
        None
    }
    pub fn wrap_around_edge(
        &self,
        cube_map_point: &CubeMapPoint<N>,
        orientation: Compass,
    ) -> Option<(CubeMapPoint<N>, Compass)> {
        if !orientation.is_cardinal() {
            return None;
        }
        match (orientation, cube_map_point.1.map_position()) {
            (Compass::N, Compass::NE)
            | (Compass::N, Compass::N)
            | (Compass::N, Compass::NW)
            | (Compass::E, Compass::NE)
            | (Compass::E, Compass::E)
            | (Compass::E, Compass::SE)
            | (Compass::S, Compass::SE)
            | (Compass::S, Compass::S)
            | (Compass::S, Compass::SW)
            | (Compass::W, Compass::NW)
            | (Compass::W, Compass::W)
            | (Compass::W, Compass::SW) => (),
            _ => return None,
        }
        let (new_surface, new_surface_side) =
            self.edges.get(&(cube_map_point.0, orientation)).unwrap();
        let wrapped_map_point: MapPoint<N, N> = match (orientation, *new_surface_side) {
            (Compass::N, Compass::S)
            | (Compass::E, Compass::E)
            | (Compass::S, Compass::N)
            | (Compass::W, Compass::W) => {
                (cube_map_point.1.x(), N - 1 - cube_map_point.1.y()).into()
            }
            (Compass::N, Compass::N)
            | (Compass::E, Compass::W)
            | (Compass::S, Compass::S)
            | (Compass::W, Compass::E) => {
                (N - 1 - cube_map_point.1.x(), cube_map_point.1.y()).into()
            }
            (Compass::N, Compass::W)
            | (Compass::E, Compass::S)
            | (Compass::S, Compass::E)
            | (Compass::W, Compass::N) => (cube_map_point.1.y(), cube_map_point.1.x()).into(),
            (Compass::N, Compass::E)
            | (Compass::E, Compass::N)
            | (Compass::S, Compass::W)
            | (Compass::W, Compass::S) => {
                (N - 1 - cube_map_point.1.y(), N - 1 - cube_map_point.1.x()).into()
            }
            _ => panic!(
                "line {}, bad edge combination {:?} - {:?}",
                line!(),
                orientation,
                new_surface_side
            ),
        };
        Some(((*new_surface, wrapped_map_point), new_surface_side.flip()))
    }
    pub fn iter_orientation(
        &self,
        start_point: CubeMapPoint<N>,
        orientation: Compass,
    ) -> impl Iterator<Item = (CubeMapPoint<N>, Compass, &T)> {
        IterOrientation::new(self, start_point, orientation)
    }
}

struct IterOrientation<'a, T: Copy + Clone + Default + From<char> + Debug, const N: usize> {
    cube_map: &'a CubeMap<T, N>,
    current_point: CubeMapPoint<N>,
    orientation: Compass,
    finished: bool,
}

impl<'a, T: Copy + Clone + Default + From<char> + Debug, const N: usize> IterOrientation<'a, T, N> {
    fn new(
        cube_map: &'a CubeMap<T, N>,
        start_point: CubeMapPoint<N>,
        orientation: Compass,
    ) -> Self {
        Self {
            cube_map,
            current_point: start_point,
            orientation,
            finished: !orientation.is_cardinal(),
        }
    }
}

impl<'a, T: Copy + Clone + Default + From<char> + Debug, const N: usize> Iterator
    for IterOrientation<'a, T, N>
{
    type Item = (CubeMapPoint<N>, Compass, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result = (
            self.current_point,
            self.orientation,
            self.cube_map
                .get_surface(self.current_point.0)
                .get(self.current_point.1),
        );
        match self.current_point.1.neighbor(self.orientation) {
            Some(next_point) => self.current_point.1 = next_point,
            None => {
                (self.current_point, self.orientation) = self
                    .cube_map
                    .wrap_around_edge(&self.current_point, self.orientation)
                    .unwrap()
            }
        }

        Some(result)
    }
}
