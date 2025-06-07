// configuration traits

pub trait MCTSConfig {
    fn exploration_constant(&self) -> f32;
    fn progressive_widening_constant(&self) -> f32;
    fn progressive_widening_exponent(&self) -> f32;
    fn early_cut_off_depth(&self) -> usize;
}

pub trait HeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32;
    fn progressive_widening_decay_rate(&self) -> f32;
    fn early_cut_off_upper_bound(&self) -> f32;
    fn early_cut_off_lower_bound(&self) -> f32;
}

pub trait RecursiveHeuristicConfig: HeuristicConfig {
    fn max_depth(&self) -> usize;
    fn alpha(&self) -> f32;
    fn alpha_reduction_factor(&self) -> f32;
    fn target_alpha(&self) -> f32;
    fn early_exit_threshold(&self) -> f32;
}
