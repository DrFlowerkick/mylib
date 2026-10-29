// trait for outcomes of games. Some games have only win or loss, some have although
// draw or other outcomes.

use std::cmp::Ord;

pub trait GameOutcome: PartialEq + Ord + Clone + Copy + Sync + Send {
    fn to_score(&self) -> f32;
}
