// base implementation of MCTSConfig, which supports UTC, ProgressiveWidening and EarlyCutOff

use std::collections::HashMap;

use super::{MCTSConfig, GamePlayer};

// default progressive widening with C = 2, alpha = 1/2
// fast progressive widening with C = 4, alpha = 1/3
// slow progressive widening with C = 1, alpha = 2/3
#[derive(Debug, Clone, PartialEq)]
pub struct BaseConfig<Player: GamePlayer> {
    pub exploration_constant: f32,
    pub exploration_boost: HashMap<Player, f32>,
    pub progressive_widening_constant: f32,
    pub progressive_widening_exponent: f32,
    pub early_cut_off_depth: usize,
}

impl<Player: GamePlayer> Default for BaseConfig<Player> {
    fn default() -> Self {
        BaseConfig {
            exploration_constant: 1.40,
            exploration_boost: HashMap::new(),
            progressive_widening_constant: 2.0,
            progressive_widening_exponent: 0.5,
            early_cut_off_depth: 20,
        }
    }
}

impl<Player: GamePlayer> MCTSConfig<Player> for BaseConfig<Player> {
    fn exploration_constant(&self) -> f32 {
        self.exploration_constant
    }
    fn exploration_boost(&self, player: Player) -> f32 {
        self.exploration_boost.get(&player).cloned().unwrap_or(1.0)  
    }
    fn progressive_widening_constant(&self) -> f32 {
        self.progressive_widening_constant
    }
    fn progressive_widening_exponent(&self) -> f32 {
        self.progressive_widening_exponent
    }
    fn early_cut_off_depth(&self) -> usize {
        self.early_cut_off_depth
    }
}
