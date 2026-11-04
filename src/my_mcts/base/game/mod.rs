// base implementation of game traits, which at this point is ony GamePlayer

mod outcome;
mod player;

pub use outcome::*;
pub use player::*;

use super::{GameOutcome, GamePlayer};
