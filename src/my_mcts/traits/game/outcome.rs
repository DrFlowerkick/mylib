// trait for outcomes of games. Some games have only win or loss, some have although
// draw or other outcomes.

use std::{cmp::Ord, ops::Neg};

pub trait GameOutcome: PartialEq + Ord + Clone + Copy + Sync + Send + Neg<Output = Self> {
    const BEST_OUTCOME: Self;
    const WORST_OUTCOME: Self;

    fn to_score(&self) -> f32;
}
