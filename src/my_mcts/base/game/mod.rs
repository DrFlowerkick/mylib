// base implementation of game traits, which at this point is ony GamePlayer

mod caching;
mod player;

pub use caching::*;
pub use player::*;

use super::{GameCache, GamePlayer};
