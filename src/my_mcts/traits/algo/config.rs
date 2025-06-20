// Configuration trait for MCTS, which supports base implementation of UTC, expansion,
// and simulation policies.

use super::GamePlayer;

pub trait MCTSConfig<Player: GamePlayer>: Clone + Sync + Send {
    fn exploration_constant(&self) -> f32 {
        1.4
    }
    fn exploration_boost(&self, _player: Player) -> f32 {
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
