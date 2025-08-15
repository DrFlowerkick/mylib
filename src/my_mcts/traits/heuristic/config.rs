// configuration traits for heuristic and recursive heuristic.

pub trait HeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32 {
        0.8
    }
    fn progressive_widening_decay_rate(&self) -> f32 {
        0.95
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        0.05
    }
    fn early_cut_off_lower_bound(&self) -> f32 {
        0.95
    }
}

pub trait RecursiveHeuristicConfig: HeuristicConfig {
    fn max_depth(&self) -> usize {
        0
    }
    fn alpha(&self) -> f32 {
        0.0
    }
    fn alpha_reduction_factor(&self) -> f32 {
        0.0
    }
    fn target_alpha(&self) -> f32 {
        0.0
    }
    fn early_exit_threshold(&self) -> f32 {
        0.0
    }
}
