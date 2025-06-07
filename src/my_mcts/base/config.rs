// base implementation of MCTSConfig

use super::super::{HeuristicConfig, MCTSConfig, RecursiveHeuristicConfig};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseHeuristicConfig {
    pub progressive_widening_initial_threshold: f32,
    pub progressive_widening_decay_rate: f32,
    pub early_cut_off_lower_bound: f32,
    pub early_cut_off_upper_bound: f32,
}

impl Default for BaseHeuristicConfig {
    fn default() -> Self {
        BaseHeuristicConfig {
            progressive_widening_initial_threshold: 0.8,
            progressive_widening_decay_rate: 0.95,
            early_cut_off_lower_bound: 0.05,
            early_cut_off_upper_bound: 0.95,
        }
    }
}

impl HeuristicConfig for BaseHeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32 {
        self.progressive_widening_initial_threshold
    }
    fn progressive_widening_decay_rate(&self) -> f32 {
        self.progressive_widening_decay_rate
    }
    fn early_cut_off_lower_bound(&self) -> f32 {
        self.early_cut_off_lower_bound
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        self.early_cut_off_upper_bound
    }
}

pub struct BaseRecursiveConfig {
    pub base_config: BaseHeuristicConfig,
    pub max_depth: usize,
    pub alpha: f32,
    pub alpha_reduction_factor: f32,
    pub target_alpha: f32,
    pub early_exit_threshold: f32,
}

impl Default for BaseRecursiveConfig {
    fn default() -> Self {
        BaseRecursiveConfig {
            base_config: BaseHeuristicConfig::default(),
            max_depth: 0,
            alpha: 0.7,
            alpha_reduction_factor: 0.9,
            target_alpha: 0.5,
            early_exit_threshold: 0.95,
        }
    }
}

impl HeuristicConfig for BaseRecursiveConfig {
    fn progressive_widening_initial_threshold(&self) -> f32 {
        self.base_config.progressive_widening_initial_threshold
    }
    fn progressive_widening_decay_rate(&self) -> f32 {
        self.base_config.progressive_widening_decay_rate
    }
    fn early_cut_off_lower_bound(&self) -> f32 {
        self.base_config.early_cut_off_lower_bound
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        self.base_config.early_cut_off_upper_bound
    }
}

impl RecursiveHeuristicConfig for BaseRecursiveConfig {
    fn max_depth(&self) -> usize {
        self.max_depth
    }
    fn alpha(&self) -> f32 {
        self.alpha
    }
    fn alpha_reduction_factor(&self) -> f32 {
        self.alpha_reduction_factor
    }
    fn target_alpha(&self) -> f32 {
        self.target_alpha
    }
    fn early_exit_threshold(&self) -> f32 {
        self.early_exit_threshold
    }
}
