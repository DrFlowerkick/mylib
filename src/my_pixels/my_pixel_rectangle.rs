//! rectangles made of discrete pixels
//! E.g. a chess board, made of 8x8 pixels

use crate::my_geometry::{my_line::LineSegment, my_point::Point, my_rectangle::Rectangle};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PixelRectangle {
    Rectangle(Rectangle),
    Segment(LineSegment),
    Pixel(Point),
}
