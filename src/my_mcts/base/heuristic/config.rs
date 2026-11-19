// Base configuration for Heuristic and RecursiveHeuristic

use super::HeuristicConfig;

// Base configuration of heuristic, which supports HeuristicProgressiveWidening and HeuristicCutoff
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

// BaseRecursiveConfig provides parameters for RecursiveHeuristic and
// extends BaseHeuristicConfig for usage with RecursiveHeuristic
pub struct BaseRecursiveConfig {
    pub base_config: BaseHeuristicConfig,
    pub max_depth: usize,
    pub alpha: f32,
    pub alpha_reduction_factor: f32,
    pub target_alpha: f32,
    pub early_exit_threshold: f32,
    pub max_turn: Option<usize>,
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
            max_turn: None,
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
    fn max_turn(&self) -> Option<usize> {
        self.max_turn
    }
}
