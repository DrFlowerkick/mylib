// The surfaces of this box are always aligned with the coordinate axis. Therefore it can be described
// by two 3d points: left_front_bottom (min corner) and right_back_top (max corner)
// Box corners may be on same axis, resulting either in
// - a surface (one identical axis)
// - a line (two identical axis)
// - a point (three identical axis -> identical corners)

use crate::my_geometry::my_point::Point3D;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default, Hash)]
pub struct Box3D {
    pub left_front_bottom: Point3D,
    pub right_back_top: Point3D,
}

impl Box3D {
    pub fn new(a: Point3D, b: Point3D) -> Self {
        let box3d = Box3D {
            left_front_bottom: a,
            right_back_top: b,
        };

        box3d.normalized()
    }

    // we consume self by purpose to make sure, that only normalized boxes are used
    pub fn normalized(mut self) -> Self {
        if self.left_front_bottom.x > self.right_back_top.x {
            std::mem::swap(&mut self.left_front_bottom.x, &mut self.right_back_top.x)
        };
        if self.left_front_bottom.y > self.right_back_top.y {
            std::mem::swap(&mut self.left_front_bottom.y, &mut self.right_back_top.y)
        };
        if self.left_front_bottom.z > self.right_back_top.z {
            std::mem::swap(&mut self.left_front_bottom.z, &mut self.right_back_top.z)
        };
        self
    }

    pub fn is_valid(&self) -> bool {
        self.left_front_bottom.x <= self.right_back_top.x
            && self.left_front_bottom.y <= self.right_back_top.y
            && self.left_front_bottom.z <= self.right_back_top.z
    }

    pub fn intersect(&self, other: Box3D) -> Option<Box3D> {
        let left_front_bottom = Point3D {
            x: self.left_front_bottom.x.max(other.left_front_bottom.x),
            y: self.left_front_bottom.y.max(other.left_front_bottom.y),
            z: self.left_front_bottom.z.max(other.left_front_bottom.z),
        };
        let right_back_top = Point3D {
            x: self.right_back_top.x.min(other.right_back_top.x),
            y: self.right_back_top.y.min(other.right_back_top.y),
            z: self.right_back_top.z.min(other.right_back_top.z),
        };
        let i = Box3D {
            left_front_bottom,
            right_back_top,
        };
        i.is_valid().then_some(i)
    }

    pub fn subtract(&self, intersect: Box3D) -> Vec<Box3D> {
        let mut remaining_boxes = Vec::with_capacity(6);

        // 1) X-Boxes (left and right of intersect)
        let left = Box3D {
            left_front_bottom: self.left_front_bottom,
            right_back_top: Point3D {
                x: intersect.left_front_bottom.x - 1,
                y: self.right_back_top.y,
                z: self.right_back_top.z,
            },
        };
        if left.is_valid() {
            remaining_boxes.push(left)
        };
        let right = Box3D {
            left_front_bottom: Point3D {
                x: intersect.right_back_top.x + 1,
                y: self.left_front_bottom.y,
                z: self.left_front_bottom.z,
            },
            right_back_top: self.right_back_top,
        };
        if right.is_valid() {
            remaining_boxes.push(right)
        };

        // 2) Y-Boxes inside of X intersect
        let front = Box3D {
            left_front_bottom: Point3D {
                x: intersect.left_front_bottom.x,
                y: self.left_front_bottom.y,
                z: self.left_front_bottom.z,
            },
            right_back_top: Point3D {
                x: intersect.right_back_top.x,
                y: intersect.left_front_bottom.y - 1,
                z: self.right_back_top.z,
            },
        };
        if front.is_valid() {
            remaining_boxes.push(front)
        };
        let back = Box3D {
            left_front_bottom: Point3D {
                x: intersect.left_front_bottom.x,
                y: intersect.right_back_top.y + 1,
                z: self.left_front_bottom.z,
            },
            right_back_top: Point3D {
                x: intersect.right_back_top.x,
                y: self.right_back_top.y,
                z: self.right_back_top.z,
            },
        };
        if back.is_valid() {
            remaining_boxes.push(back)
        };

        // 3) Z-Boxes inside of X and Y intersect
        let bottom = Box3D {
            left_front_bottom: Point3D {
                x: intersect.left_front_bottom.x,
                y: intersect.left_front_bottom.y,
                z: self.left_front_bottom.z,
            },
            right_back_top: Point3D {
                x: intersect.right_back_top.x,
                y: intersect.right_back_top.y,
                z: intersect.left_front_bottom.z - 1,
            },
        };
        if bottom.is_valid() {
            remaining_boxes.push(bottom)
        };
        let top = Box3D {
            left_front_bottom: Point3D {
                x: intersect.left_front_bottom.x,
                y: intersect.left_front_bottom.y,
                z: intersect.right_back_top.z + 1,
            },
            right_back_top: Point3D {
                x: intersect.right_back_top.x,
                y: intersect.right_back_top.y,
                z: self.right_back_top.z,
            },
        };
        if top.is_valid() {
            remaining_boxes.push(top)
        };

        remaining_boxes
    }

    pub fn split_intersecting(&self, other: Box3D) -> Option<(Box3D, Vec<Box3D>, Vec<Box3D>)> {
        self.is_valid().then_some(())?;
        other.is_valid().then_some(())?;
        let i = self.intersect(other)?;
        Some((i, self.subtract(i), other.subtract(i)))
    }

    pub fn size(&self) -> Option<i64> {
        self.is_valid().then_some(
            (self.right_back_top.x - self.left_front_bottom.x + 1)
                * (self.right_back_top.y - self.left_front_bottom.y + 1)
                * (self.right_back_top.z - self.left_front_bottom.z + 1),
        )
    }

    pub fn delta_to_point(&self, point: Point3D) -> i64 {
        let dx = if point.x < self.left_front_bottom.x {
            self.left_front_bottom.x - point.x
        } else if point.x > self.right_back_top.x {
            point.x - self.right_back_top.x
        } else {
            0
        };
        let dy = if point.y < self.left_front_bottom.y {
            self.left_front_bottom.y - point.y
        } else if point.y > self.right_back_top.y {
            point.y - self.right_back_top.y
        } else {
            0
        };
        let dz = if point.z < self.left_front_bottom.z {
            self.left_front_bottom.z - point.z
        } else if point.z > self.right_back_top.z {
            point.z - self.right_back_top.z
        } else {
            0
        };
        dx + dy + dz
    }

    pub fn split_box(&self) -> Vec<Box3D> {
        if let Some(size) = self.size() {
            if size <= 1 {
                return vec![*self];
            }
        } else {
            return vec![];
        }
        let mut boxes = Vec::with_capacity(8);
        let mid_x = (self.left_front_bottom.x + self.right_back_top.x) / 2;
        let mid_y = (self.left_front_bottom.y + self.right_back_top.y) / 2;
        let mid_z = (self.left_front_bottom.z + self.right_back_top.z) / 2;

        let coords_x = [
            (self.left_front_bottom.x, mid_x),
            (mid_x + 1, self.right_back_top.x),
        ];
        let coords_y = [
            (self.left_front_bottom.y, mid_y),
            (mid_y + 1, self.right_back_top.y),
        ];
        let coords_z = [
            (self.left_front_bottom.z, mid_z),
            (mid_z + 1, self.right_back_top.z),
        ];

        for &(x1, x2) in &coords_x {
            for &(y1, y2) in &coords_y {
                for &(z1, z2) in &coords_z {
                    let b = Box3D::new(Point3D::new(x1, y1, z1), Point3D::new(x2, y2, z2));
                    boxes.push(b);
                }
            }
        }

        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        let box_a_corner_a = Point3D::new(10, 10, 10);
        let box_a_corner_b = Point3D::new(12, 12, 12);
        let box_a = Box3D::new(box_a_corner_a, box_a_corner_b);
        let size_a = box_a.size().unwrap();

        let box_b_corner_a = Point3D::new(11, 11, 11);
        let box_b_corner_b = Point3D::new(13, 13, 13);
        let box_b = Box3D::new(box_b_corner_a, box_b_corner_b);
        let size_b = box_b.size().unwrap();

        let (intersection, remaining_a, remaining_b) = box_a.split_intersecting(box_b).unwrap();
        let size_intersection = intersection.size().unwrap();

        // check intersection
        assert_eq!(intersection.left_front_bottom, Point3D::new(11, 11, 11));
        assert_eq!(intersection.right_back_top, Point3D::new(12, 12, 12));

        // check remaining a
        assert_eq!(remaining_a.len(), 3);
        let remaining_a_box_left = remaining_a[0];
        assert_eq!(
            remaining_a_box_left.left_front_bottom,
            Point3D::new(10, 10, 10)
        );
        assert_eq!(
            remaining_a_box_left.right_back_top,
            Point3D::new(10, 12, 12)
        );
        let size_remaining_a_box_left = remaining_a_box_left.size().unwrap();
        let remaining_a_box_front = remaining_a[1];
        assert_eq!(
            remaining_a_box_front.left_front_bottom,
            Point3D::new(11, 10, 10)
        );
        assert_eq!(
            remaining_a_box_front.right_back_top,
            Point3D::new(12, 10, 12)
        );
        let size_remaining_a_box_front = remaining_a_box_front.size().unwrap();
        let remaining_a_box_bottom = remaining_a[2];
        assert_eq!(
            remaining_a_box_bottom.left_front_bottom,
            Point3D::new(11, 11, 10)
        );
        assert_eq!(
            remaining_a_box_bottom.right_back_top,
            Point3D::new(12, 12, 10)
        );
        let size_remaining_a_box_bottom = remaining_a_box_bottom.size().unwrap();
        assert_eq!(
            size_a,
            size_intersection
                + size_remaining_a_box_left
                + size_remaining_a_box_front
                + size_remaining_a_box_bottom
        );

        // check remaining b
        assert_eq!(remaining_b.len(), 3);
        let remaining_b_box_right = remaining_b[0];
        assert_eq!(
            remaining_b_box_right.left_front_bottom,
            Point3D::new(13, 11, 11)
        );
        assert_eq!(
            remaining_b_box_right.right_back_top,
            Point3D::new(13, 13, 13)
        );
        let size_remaining_b_box_right = remaining_b_box_right.size().unwrap();
        let remaining_b_box_back = remaining_b[1];
        assert_eq!(
            remaining_b_box_back.left_front_bottom,
            Point3D::new(11, 13, 11)
        );
        assert_eq!(
            remaining_b_box_back.right_back_top,
            Point3D::new(12, 13, 13)
        );
        let size_remaining_b_box_back = remaining_b_box_back.size().unwrap();
        let remaining_b_box_top = remaining_b[2];
        assert_eq!(
            remaining_b_box_top.left_front_bottom,
            Point3D::new(11, 11, 13)
        );
        assert_eq!(remaining_b_box_top.right_back_top, Point3D::new(12, 12, 13));
        let size_remaining_b_box_top = remaining_b_box_top.size().unwrap();
        assert_eq!(
            size_b,
            size_intersection
                + size_remaining_b_box_right
                + size_remaining_b_box_back
                + size_remaining_b_box_top
        );

        // check again sum sizes
        let size_remaining_a: i64 = remaining_a.iter().filter_map(|b| b.size()).sum();
        let size_remaining_b: i64 = remaining_b.iter().filter_map(|b| b.size()).sum();
        // intersection is part of a and b, therefore factor 2
        assert_eq!(
            size_a + size_b,
            2 * size_intersection + size_remaining_a + size_remaining_b
        );
    }

    #[test]
    fn test_intersection_box_inside_box() {
        let box_a_corner_a = Point3D::new(0, 0, 0);
        let box_a_corner_b = Point3D::new(3, 3, 3);
        let box_a = Box3D::new(box_a_corner_a, box_a_corner_b);
        let size_a = box_a.size().unwrap();
        assert_eq!(size_a, 64);

        let box_b_corner_a = Point3D::new(1, 1, 1);
        let box_b_corner_b = Point3D::new(2, 2, 2);
        let box_b = Box3D::new(box_b_corner_a, box_b_corner_b);
        let size_b = box_b.size().unwrap();
        assert_eq!(size_b, 8);

        let (intersection, remaining_a, remaining_b) = box_a.split_intersecting(box_b).unwrap();
        let size_intersection = intersection.size().unwrap();
        assert_eq!(size_intersection, 8);
        assert_eq!(
            remaining_a
                .into_iter()
                .filter_map(|b| b.size())
                .collect::<Vec<i64>>(),
            [16, 16, 8, 8, 4, 4]
        );
        assert_eq!(remaining_b.len(), 0);
    }
}
