// base implementation of MCTSConfig, which supports UTC, ProgressiveWidening and EarlyCutOff

use super::MCTSConfig;

// default progressive widening with C = 2, alpha = 1/2
// fast progressive widening with C = 4, alpha = 1/3
// slow progressive widening with C = 1, alpha = 2/3
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseConfig {
    pub exploration_constant: f32,
    pub progressive_widening_constant: f32,
    pub progressive_widening_exponent: f32,
    pub early_cut_off_depth: usize,
}

impl Default for BaseConfig {
    fn default() -> Self {
        BaseConfig {
            exploration_constant: 1.40,
            progressive_widening_constant: 2.0,
            progressive_widening_exponent: 0.5,
            early_cut_off_depth: 20,
        }
    }
}

impl MCTSConfig for BaseConfig {
    fn exploration_constant(&self) -> f32 {
        self.exploration_constant
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
