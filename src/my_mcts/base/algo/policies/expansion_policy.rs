// implementations of ExpansionPolicy

use super::{ExpansionPolicy, GameCache, Heuristic, HeuristicConfig, MCTSConfig, MCTSGame};
use rand::{prelude::SliceRandom, rng};

#[derive(Clone)]
pub struct ExpandAll {}

impl<G, H, Config> ExpansionPolicy<G, H, Config> for ExpandAll
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
    fn new(
        _state: &G::State,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
        _heuristic_config: &H::Config,
    ) -> Self {
        ExpandAll {}
    }
    fn expandable_moves(
        &mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &G::State,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        let mut moves: Vec<_> = G::available_moves(state).collect();
        moves.shuffle(&mut rng());
        moves
    }
}

#[derive(Clone)]
pub struct ProgressiveWidening<G, Config>
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
    pub unexpanded_moves: Vec<G::Move>,
    phantom: std::marker::PhantomData<Config>,
}

impl<G, Config> ProgressiveWidening<G, Config>
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
    fn allowed_children(visits: usize, mcts_config: &Config) -> usize {
        if visits == 0 {
            1
        } else {
            (mcts_config.progressive_widening_constant()
                * (visits as f32).powf(mcts_config.progressive_widening_exponent()))
            .floor() as usize
        }
    }
}

impl<G, H, Config> ExpansionPolicy<G, H, Config> for ProgressiveWidening<G, Config>
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
    fn new(
        state: &G::State,
        game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
        _heuristic_config: &H::Config,
    ) -> Self {
        let is_terminal = match game_cache.get_terminal_value(state) {
            Some(status) => status.is_some(),
            None => G::evaluate(state, game_cache).is_some(),
        };
        if is_terminal {
            return ProgressiveWidening {
                unexpanded_moves: vec![],
                phantom: std::marker::PhantomData,
            };
        }
        let mut unexpanded_moves = G::available_moves(state).collect::<Vec<_>>();
        unexpanded_moves.shuffle(&mut rng());
        ProgressiveWidening {
            unexpanded_moves,
            phantom: std::marker::PhantomData,
        }
    }
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        num_parent_children < Self::allowed_children(visits, mcts_config)
            && !self.unexpanded_moves.is_empty()
    }
    fn expandable_moves(
        &mut self,
        visits: usize,
        num_parent_children: usize,
        _state: &G::State,
        mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        let allowed_children = Self::allowed_children(visits, mcts_config);
        if allowed_children > num_parent_children && !self.unexpanded_moves.is_empty() {
            let num_expandable_moves = self
                .unexpanded_moves
                .len()
                .min(allowed_children - num_parent_children);
            self.unexpanded_moves
                .drain(..num_expandable_moves)
                .collect()
        } else {
            vec![]
        }
    }
}

#[derive(Clone)]
pub struct HeuristicProgressiveWidening<G, H, Config>
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
    pub unexpanded_moves: Vec<(f32, G::Move)>,
    phantom: std::marker::PhantomData<(H, Config)>,
}

impl<G, H, Config> HeuristicProgressiveWidening<G, H, Config>
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
    fn allowed_children(visits: usize, mcts_config: &Config) -> usize {
        if visits == 0 {
            1
        } else {
            (mcts_config.progressive_widening_constant()
                * (visits as f32).powf(mcts_config.progressive_widening_exponent()))
            .floor() as usize
        }
    }
    fn threshold(visits: usize, heuristic_config: &H::Config) -> f32 {
        heuristic_config.progressive_widening_initial_threshold()
            * heuristic_config
                .progressive_widening_decay_rate()
                .powi(visits as i32)
    }
}

impl<G, H, Config> ExpansionPolicy<G, H, Config> for HeuristicProgressiveWidening<G, H, Config>
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
    fn new(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        heuristic_config: &H::Config,
    ) -> Self {
        let is_terminal = match game_cache.get_terminal_value(state) {
            Some(status) => status.is_some(),
            None => G::evaluate(state, game_cache).is_some(),
        };
        if is_terminal {
            return HeuristicProgressiveWidening {
                unexpanded_moves: vec![],
                phantom: std::marker::PhantomData,
            };
        }
        let unexpanded_moves = G::available_moves(state).collect::<Vec<_>>();
        let unexpanded_moves = H::sort_moves(
            state,
            unexpanded_moves,
            game_cache,
            heuristic_cache,
            heuristic_config,
        );
        HeuristicProgressiveWidening {
            unexpanded_moves,
            phantom: std::marker::PhantomData,
        }
    }
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> bool {
        let threshold = Self::threshold(visits, heuristic_config);
        num_parent_children < Self::allowed_children(visits, mcts_config)
            && self
                .unexpanded_moves
                .iter()
                .any(|(score, _)| *score >= threshold)
    }
    fn expandable_moves(
        &mut self,
        visits: usize,
        num_parent_children: usize,
        _state: &G::State,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        let allowed_children = Self::allowed_children(visits, mcts_config);
        if num_parent_children < allowed_children && !self.unexpanded_moves.is_empty() {
            let num_expandable_moves = self
                .unexpanded_moves
                .len()
                .min(allowed_children - num_parent_children);

            let threshold = Self::threshold(visits, heuristic_config);
            let cutoff_index = self
                .unexpanded_moves
                .iter()
                .position(|(score, _)| *score < threshold)
                .unwrap_or(self.unexpanded_moves.len());
            // max(1) is required, if leaf node with no children is selected and all moves have
            // a heuristic score below threshold
            let selected_count = cutoff_index.min(num_expandable_moves).max(1);
            self.unexpanded_moves
                .drain(..selected_count)
                .map(|(_, mv)| mv)
                .collect()
        } else {
            vec![]
        }
    }
}
