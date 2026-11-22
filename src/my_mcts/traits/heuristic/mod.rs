// all traits to define a specific heuristic for the game, which is defined by MCTSGame.

mod config;
mod core_heuristic;

pub use config::*;
pub use core_heuristic::*;

use super::{MCTSGame, SimulationScore};
