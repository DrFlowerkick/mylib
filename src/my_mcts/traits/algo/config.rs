// Configuration trait for MCTS, which supports base implementation of UTC, expansion,
// and simulation policies.

pub trait MCTSConfig {
    fn exploration_constant(&self) -> f32 {
        1.4
    }
    fn non_perspective_player_exploration_boost(&self) -> f32 {
        1.0
    }
    fn progressive_widening_constant(&self) -> f32 {
        2.0
    }
    fn progressive_widening_exponent(&self) -> f32 {
        0.5
    }
    fn early_cut_off_depth(&self) -> usize {
        20
    }
}
