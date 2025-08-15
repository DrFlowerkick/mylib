// library for some basic geometric functions
// at the moment I use integer, if possible, since a lot of challenges
// use integers or pixels

pub mod my_circle;
pub mod my_diamond;
pub mod my_line;
pub mod my_point;
pub mod my_rectangle;

pub enum FormOrdering {
    Identical,
    Inside,
    InsideTouching,
    Overlapping,
    Touching,
    NonOverlapping,
}

// mathematic helper functions

// greatest common divider
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.abs()
}
