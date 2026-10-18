// caching traits for mcts including TranspositionTable

// ToDo: some refactoring. We will add an Edge trait and struct and UTCCache
// can be used by both, Node and Edge. We will implement at least 4 times:
// NoUTCCache
// ExploitationCache
// ExplorationCache
// ExploitationAndExplorationCache
// We will implement at tree level functions to return exploitation and exploration
// score (including updating stats during back propagation). These functions will be
// implemented depending on Type of UTCCache used by Node and Edge. We will allow
// only these combinations:
// Node: ExploitationAndExplorationCache and Edge: NoUTCCache
// Node: NoUTCCache and Edge: ExploitationAndExplorationCache
// Node: ExploitationCache and Edge: ExplorationCache (this combination is best suited
// for transposition table).
// All other combinations will not be able to compile.
// At the moment I plan to save score values together with visits and accumulated_value
// respectively last_parent_visits.
// It is maybe required to create some marker trait like UTCEdgeOnly, UTCNodeOnly,
// UTCHybrid for these three use cases. This would allow reuse of code, if you want
// to implement UTCCache with only the minimal required stats (visits and accumulated_value).

use super::{MCTSConfig, MCTSGame, UCTPolicy};

pub trait UTCCache<G, UTC, Config>: Clone + Sync + Send + Default
where
    G: MCTSGame,
    UTC: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn update_exploitation(
        &mut self,
        visits: usize,
        accumulated_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    );

    fn get_exploitation(
        &self,
        visits: usize,
        accumulated_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32;

    fn update_exploration(
        &mut self,
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    );

    fn get_exploration(
        &self,
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32;
}
