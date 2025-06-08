// all traits to define a specific heuristic for the game, which is defined by MCTSGame.
// This includes an optional trait for a recursive heuristic, which is calculated
// recursively over a configured depth.

mod caching;
mod config;
mod core;
mod recursive;

pub use self::core::*;
pub use caching::*;
pub use config::*;
pub use recursive::*;

use super::*;
