// The surfaces of this box are always aligned with the coordinate axis. Therefore it can be described
// by two 3d points: left_front_bottom (min corner) and right_back_top (max corner)

use super::my_point::Point3D;

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
            std::mem::swap(&mut self.left_front_bottom.y, &mut self.right_back_top.z)
        };
        if self.left_front_bottom.y > self.right_back_top.y {
            std::mem::swap(&mut self.left_front_bottom.y, &mut self.right_back_top.z)
        };
        self
    }

    pub fn is_valid(&self) -> bool {
        self.left_front_bottom.x < self.right_back_top.x
            && self.left_front_bottom.y < self.right_back_top.y
            && self.left_front_bottom.z < self.right_back_top.z
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
                x: intersect.left_front_bottom.x,
                y: self.right_back_top.y,
                z: self.right_back_top.z,
            },
        };
        if left.is_valid() {
            remaining_boxes.push(left)
        };
        let right = Box3D {
            left_front_bottom: Point3D {
                x: intersect.right_back_top.x,
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
                y: intersect.left_front_bottom.y,
                z: self.right_back_top.z,
            },
        };
        if front.is_valid() {
            remaining_boxes.push(front)
        };
        let back = Box3D {
            left_front_bottom: Point3D {
                x: intersect.left_front_bottom.x,
                y: intersect.right_back_top.y,
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
                z: intersect.left_front_bottom.z,
            },
        };
        if bottom.is_valid() {
            remaining_boxes.push(bottom)
        };
        let top = Box3D {
            left_front_bottom: Point3D {
                x: intersect.left_front_bottom.x,
                y: intersect.left_front_bottom.y,
                z: intersect.right_back_top.z,
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
}
