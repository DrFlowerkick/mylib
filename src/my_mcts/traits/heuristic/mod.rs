// all traits to define a specific heuristic for the game, which is defined by MCTSGame.
// This includes an optional trait for a recursive heuristic, which is calculated
// recursively over a configured depth.

mod config;
mod core_heuristic;

pub use config::*;
pub use core_heuristic::*;

use super::MCTSGame;
