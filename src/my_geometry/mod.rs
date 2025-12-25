// library for some basic geometric functions
// at the moment I use integer, if possible, since a lot of challenges
// use integers or pixels

pub mod my_box;
pub mod my_circle;
pub mod my_diamond;
pub mod my_line;
pub mod my_point;
pub mod my_rectangle;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FormOrdering {
    Identical,
    Inside,
    InsideTouching,
    Overlapping,
    Touching,
    NonOverlapping,
}
