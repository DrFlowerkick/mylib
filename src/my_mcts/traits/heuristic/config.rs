// configuration traits for heuristic.

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
    // heuristic will be used while current_turn < max_turn
    fn max_turn(&self) -> Option<usize> {
        None
    }
}
