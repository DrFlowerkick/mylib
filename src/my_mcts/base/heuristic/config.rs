// Base configuration for Heuristic

use super::HeuristicConfig;

// Base configuration of heuristic, which supports HeuristicProgressiveWidening and HeuristicCutoff
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseHeuristicConfig {
    pub progressive_widening_initial_threshold: f32,
    pub progressive_widening_decay_rate: f32,
    pub early_cut_off_lower_bound: f32,
    pub early_cut_off_upper_bound: f32,
    pub max_turn: Option<usize>,
}

impl Default for BaseHeuristicConfig {
    fn default() -> Self {
        BaseHeuristicConfig {
            progressive_widening_initial_threshold: 0.8,
            progressive_widening_decay_rate: 0.95,
            early_cut_off_lower_bound: 0.05,
            early_cut_off_upper_bound: 0.95,
            max_turn: None,
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
    fn max_turn(&self) -> Option<usize> {
        self.max_turn
    }
}
