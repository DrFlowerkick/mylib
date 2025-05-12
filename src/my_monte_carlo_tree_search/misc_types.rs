// miscellaneous mcts type definitions

use super::{
    ExpansionPolicy, GameCache, Heuristic, HeuristicCache, MCTSGame, MCTSPlayer, SimulationPolicy,
    UCTPolicy, UTCCache, MCTSConfig, HeuristicConfig,
};
use rand::prelude::SliceRandom;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum TwoPlayer {
    #[default]
    Me,
    Opp,
}

impl TwoPlayer {
    pub fn next_player(&self) -> Self {
        match self {
            TwoPlayer::Me => TwoPlayer::Opp,
            TwoPlayer::Opp => TwoPlayer::Me,
        }
    }
}

impl MCTSPlayer for TwoPlayer {
    fn next(&self) -> Self {
        match self {
            TwoPlayer::Me => TwoPlayer::Opp,
            TwoPlayer::Opp => TwoPlayer::Me,
        }
    }
}


// default progressive widening with C = 2, alpha = 1/2
// fast progressive widening with C = 4, alpha = 1/3
// slow progressive widening with C = 1, alpha = 2/3
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

pub struct NoGameCache<State, Move> {
    phantom: std::marker::PhantomData<(State, Move)>,
}

impl<State, Move> GameCache<State, Move> for NoGameCache<State, Move> {
    fn new() -> Self {
        NoGameCache {
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct StaticC {}

impl<G: MCTSGame> UCTPolicy<G> for StaticC {}

pub struct DynamicC {}

impl<G: MCTSGame> UCTPolicy<G> for DynamicC {
    fn exploration_score(visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32 {
        let dynamic_c = mcts_config.exploration_constant() / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

pub struct NoUTCCache;

impl<G: MCTSGame, UP: UCTPolicy<G>> UTCCache<G, UP> for NoUTCCache {
    fn new() -> Self {
        NoUTCCache
    }

    fn update_exploitation(&mut self, _v: usize, _a: f32, _l: G::Player, _p: G::Player) {}
    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        UP::exploitation_score(acc_value, visits, last_player, perspective_player)
    }

    fn update_exploration(&mut self, _v: usize, _p: usize, _mc: &G::Config) {}
    fn get_exploration(&self, visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32 {
        UP::exploration_score(visits, parent_visits, mcts_config)
    }
}

pub struct CachedUTC {
    exploitation: f32,
    exploration: f32,
    last_parent_visits: usize,
}

impl<G: MCTSGame, UP: UCTPolicy<G>> UTCCache<G, UP> for CachedUTC {
    fn new() -> Self {
        CachedUTC {
            exploitation: 0.0,
            exploration: 0.0,
            last_parent_visits: 0,
        }
    }

    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            UP::exploitation_score(acc_value, visits, last_player, perspective_player);
    }

    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }

    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &G::Config) {
        if self.last_parent_visits != parent_visits {
            self.exploration = UP::exploration_score(visits, parent_visits, mcts_config);
            self.last_parent_visits = parent_visits;
        }
    }

    fn get_exploration(&self, _v: usize, _p: usize, _mc: &G::Config) -> f32 {
        self.exploration
    }
}

pub struct ExpandAll<G: MCTSGame> {
    phantom: std::marker::PhantomData<G>,
}

impl<G: MCTSGame, H: Heuristic<G>> ExpansionPolicy<G, H> for ExpandAll<G> {
    fn new(
        _state: &<G as MCTSGame>::State,
        _game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut <H as Heuristic<G>>::Cache,
        _heuristic_config: &<H as Heuristic<G>>::Config,
    ) -> Self {
        ExpandAll {
            phantom: std::marker::PhantomData,
        }
    }
}

struct BaseProgressiveWidening<G: MCTSGame> {
    pub unexpanded_moves: Vec<G::Move>,
}

impl<G: MCTSGame> BaseProgressiveWidening<G>
{
    fn allowed_children(visits: usize, mcts_config: &G::Config) -> usize {
        if visits == 0 {
            1
        } else {
            (mcts_config.progressive_widening_constant() * (visits as f32).powf(mcts_config.progressive_widening_exponent())).floor() as usize
        }
    }

    fn should_expand(&self, visits: usize, num_parent_children: usize, mcts_config: &G::Config) -> bool {
        num_parent_children < Self::allowed_children(visits, mcts_config) && !self.unexpanded_moves.is_empty()
    }

    fn expandable_moves(
        &mut self,
        visits: usize,
        num_parent_children: usize,
        _state: &<G as MCTSGame>::State,
        mcts_config: &G::Config,
    ) -> Box<dyn Iterator<Item = <G as MCTSGame>::Move> + '_> {
        let allowed_children = Self::allowed_children(visits, mcts_config);
        if allowed_children > num_parent_children && !self.unexpanded_moves.is_empty() {
            let num_expandable_moves = self
                .unexpanded_moves
                .len()
                .min(allowed_children - num_parent_children);
            Box::new(self.unexpanded_moves.drain(..num_expandable_moves))
        } else {
            Box::new(std::iter::empty())
        }
    }
}

pub struct ProgressiveWidening<G: MCTSGame>(BaseProgressiveWidening<G>);

impl<G: MCTSGame, H: Heuristic<G>> ExpansionPolicy<G, H> for ProgressiveWidening<G>
{
    fn new(
        state: &<G as MCTSGame>::State,
        game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut <H as Heuristic<G>>::Cache,
        _heuristic_config: &<H as Heuristic<G>>::Config,
    ) -> Self {
        let is_terminal = match game_cache.get_terminal_value(state) {
            Some(status) => status.is_some(),
            None => G::evaluate(state, game_cache).is_some(),
        };
        if is_terminal {
            return ProgressiveWidening(BaseProgressiveWidening {
                unexpanded_moves: vec![],
            });
        }
        let mut unexpanded_moves = G::available_moves(state).collect::<Vec<_>>();
        unexpanded_moves.shuffle(&mut rand::thread_rng());
        ProgressiveWidening(BaseProgressiveWidening { unexpanded_moves })
    }
    fn should_expand(&self, visits: usize, num_parent_children: usize, mcts_config: &G::Config) -> bool {
        self.0.should_expand(visits, num_parent_children, mcts_config)
    }
    fn expandable_moves(
        &mut self,
        visits: usize,
        num_parent_children: usize,
        state: &<G as MCTSGame>::State,
        mcts_config: &G::Config,
    ) -> Box<dyn Iterator<Item = <G as MCTSGame>::Move> + '_> {
        self.0.expandable_moves(visits, num_parent_children, state, mcts_config)
    }
}

pub struct HeuristicProgressiveWidening<G: MCTSGame>(BaseProgressiveWidening<G>);

impl<G: MCTSGame, H: Heuristic<G>> ExpansionPolicy<G, H> for HeuristicProgressiveWidening<G> {
    fn new(
        state: &<G as MCTSGame>::State,
        game_cache: &mut <G as MCTSGame>::Cache,
        heuristic_cache: &mut <H as Heuristic<G>>::Cache,
        heuristic_config: &<H as Heuristic<G>>::Config,
    ) -> Self {
        let is_terminal = match game_cache.get_terminal_value(state) {
            Some(status) => status.is_some(),
            None => G::evaluate(state, game_cache).is_some(),
        };
        if is_terminal {
            return HeuristicProgressiveWidening(BaseProgressiveWidening {
                unexpanded_moves: vec![],
            });
        }
        let unexpanded_moves = G::available_moves(state).collect::<Vec<_>>();
        let unexpanded_moves = H::sort_moves(
            state,
            unexpanded_moves,
            game_cache,
            heuristic_cache,
            heuristic_config,
        );
        HeuristicProgressiveWidening(BaseProgressiveWidening { unexpanded_moves })
    }
    fn should_expand(&self, visits: usize, num_parent_children: usize, mcts_config: &G::Config) -> bool {
        self.0.should_expand(visits, num_parent_children, mcts_config)
    }
    fn expandable_moves(
        &mut self,
        visits: usize,
        num_parent_children: usize,
        state: &<G as MCTSGame>::State,
        mcts_config: &G::Config,
    ) -> Box<dyn Iterator<Item = <G as MCTSGame>::Move> + '_> {
        self.0.expandable_moves(visits, num_parent_children, state, mcts_config)
    }
}

pub struct DefaultSimulationPolicy {}

impl<G: MCTSGame, H: Heuristic<G>> SimulationPolicy<G, H> for DefaultSimulationPolicy {}

pub struct HeuristicCutoff {}

impl<G: MCTSGame, H: Heuristic<G>> SimulationPolicy<G, H> for HeuristicCutoff {
    fn should_cutoff(
        state: &G::State,
        depth: usize,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        perspective_player: Option<G::Player>,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Option<f32> {
        let heuristic = H::evaluate_state(state, game_cache, heuristic_cache, perspective_player, heuristic_config);
        if depth >= mcts_config.early_cut_off_depth() || heuristic <= heuristic_config.early_cut_off_lower_bound() || heuristic >= heuristic_config.early_cut_off_upper_bound() {
            Some(heuristic)
        } else {
            None
        }
    }
}

pub struct NoHeuristicCache<State, Move> {
    phantom: std::marker::PhantomData<(State, Move)>,
}

impl<State, Move> HeuristicCache<State, Move> for NoHeuristicCache<State, Move> {
    fn new() -> Self {
        NoHeuristicCache {
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct BaseHeuristicConfig {
    pub early_cut_off_lower_bound: f32,
    pub early_cut_off_upper_bound: f32,
    pub evaluate_state_recursive_depth: usize,
    pub evaluate_state_recursive_alpha: f32,
}

impl Default for BaseHeuristicConfig {
    fn default() -> Self {
        BaseHeuristicConfig {
            early_cut_off_lower_bound: 0.05,
            early_cut_off_upper_bound: 0.95,
            evaluate_state_recursive_depth: 0,
            evaluate_state_recursive_alpha: 0.7,
        }
    }
}

impl HeuristicConfig for BaseHeuristicConfig {
    fn early_cut_off_lower_bound(&self) -> f32 {
        self.early_cut_off_lower_bound
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        self.early_cut_off_upper_bound
    }
    fn evaluate_state_recursive_depth(&self) -> usize {
        self.evaluate_state_recursive_depth
    }
    fn evaluate_state_recursive_alpha(&self) -> f32 {
        self.evaluate_state_recursive_alpha
    }
}

pub struct NoHeuristic {}

impl<G: MCTSGame> Heuristic<G> for NoHeuristic {
    type Cache = NoHeuristicCache<G::State, G::Move>;
    type Config = BaseHeuristicConfig;

    fn evaluate_state(
        state: &<G as MCTSGame>::State,
        game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
        _perspective_player: Option<G::Player>,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        G::evaluate(state, game_cache).unwrap_or(0.5)
    }
    fn evaluate_move(
        _state: &<G as MCTSGame>::State,
        _mv: &<G as MCTSGame>::Move,
        _game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        0.0
    }
}
