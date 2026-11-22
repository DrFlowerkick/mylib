// implementations of ExpansionPolicy

use super::{ExpansionPolicy, Heuristic, HeuristicConfig, MCTSConfig, MCTSGame, SimulationScore};
use rand::{prelude::SliceRandom, rng};

#[derive(Clone)]
pub struct ExpandAll<G>
where
    G: MCTSGame,
{
    unexpanded_moves: Option<Vec<G::Move>>,
}

impl<G> Default for ExpandAll<G>
where
    G: MCTSGame,
{
    fn default() -> Self {
        Self {
            unexpanded_moves: None,
        }
    }
}

impl<G, S, H, Config> ExpansionPolicy<G, S, H, Config> for ExpandAll<G>
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
    fn should_expand(
        &self,
        _visits: usize,
        _num_children: usize,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        if let Some(m) = &self.unexpanded_moves {
            !m.is_empty()
        } else {
            false
        }
    }
    fn pop_expandable_move(
        &mut self,
        _current_turn: usize,
        state: &G::State,
        _visits: usize,
        _num_children: usize,
        _force: bool,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Option<G::Move> {
        if let Some(moves) = &mut self.unexpanded_moves {
            moves.pop()
        } else {
            let mut moves: Vec<_> = G::available_moves(state).collect();
            moves.shuffle(&mut rng());
            let mv = moves.pop();
            self.unexpanded_moves = Some(moves);
            mv
        }
    }
    fn is_fully_expanded(&self) -> bool {
        if let Some(m) = &self.unexpanded_moves {
            m.is_empty()
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct ProgressiveWidening<G, Config>
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
    pub unexpanded_moves: Option<Vec<G::Move>>,
    phantom: std::marker::PhantomData<Config>,
}

impl<G, Config> Default for ProgressiveWidening<G, Config>
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
    fn default() -> Self {
        Self {
            unexpanded_moves: None,
            phantom: std::marker::PhantomData,
        }
    }
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

impl<G, S, H, Config> ExpansionPolicy<G, S, H, Config> for ProgressiveWidening<G, Config>
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
    fn should_expand(
        &self,
        visits: usize,
        num_children: usize,
        mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        if let Some(moves) = &self.unexpanded_moves
            && num_children < Self::allowed_children(visits, mcts_config)
            && !moves.is_empty()
        {
            true
        } else {
            false
        }
    }
    fn pop_expandable_move(
        &mut self,
        _current_turn: usize,
        state: &G::State,
        visits: usize,
        num_children: usize,
        force: bool,
        mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Option<G::Move> {
        if let Some(moves) = &mut self.unexpanded_moves {
            if force || num_children < Self::allowed_children(visits, mcts_config) {
                moves.pop()
            } else {
                None
            }
        } else {
            let mut moves = G::available_moves(state).collect::<Vec<_>>();
            moves.shuffle(&mut rng());
            let mv = moves.pop();
            self.unexpanded_moves = Some(moves);
            mv
        }
    }
    fn is_fully_expanded(&self) -> bool {
        if let Some(moves) = &self.unexpanded_moves {
            moves.is_empty()
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct HeuristicProgressiveWidening<G, S, H, Config>
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
    pub unexpanded_moves: Option<Vec<(f32, G::Move)>>,
    phantom: std::marker::PhantomData<(S, H, Config)>,
}

impl<G, S, H, Config> Default for HeuristicProgressiveWidening<G, S, H, Config>
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
    fn default() -> Self {
        Self {
            unexpanded_moves: None,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, S, H, Config> HeuristicProgressiveWidening<G, S, H, Config>
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
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

impl<G, S, H, Config> ExpansionPolicy<G, S, H, Config>
    for HeuristicProgressiveWidening<G, S, H, Config>
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
    fn should_expand(
        &self,
        visits: usize,
        num_children: usize,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> bool {
        let threshold = Self::threshold(visits, heuristic_config);
        if let Some(moves) = &self.unexpanded_moves
            && num_children < Self::allowed_children(visits, mcts_config)
            && moves.iter().any(|(score, _)| *score >= threshold)
        {
            true
        } else {
            false
        }
    }
    fn pop_expandable_move(
        &mut self,
        current_turn: usize,
        state: &G::State,
        visits: usize,
        num_children: usize,
        force: bool,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move> {
        if let Some(moves) = &mut self.unexpanded_moves {
            if let Some(max_turn) = heuristic_config.max_turn()
                && max_turn > current_turn
            {
                let threshold = Self::threshold(visits, heuristic_config);
                if force
                    || (num_children < Self::allowed_children(visits, mcts_config)
                        && moves.iter().any(|(score, _)| *score >= threshold))
                {
                    moves.pop().map(|(_, m)| m)
                } else {
                    None
                }
            } else {
                // fall back to progressive widening, if max turn for heuristic is reached
                if force || num_children < Self::allowed_children(visits, mcts_config) {
                    moves.pop().map(|(_, m)| m)
                } else {
                    None
                }
            }
        } else if let Some(max_turn) = heuristic_config.max_turn()
            && max_turn > current_turn
        {
            let mut heuristic_moves = G::available_moves(state)
                .map(|mv| {
                    (
                        // current player, because we apply mv of current player to state and want to get
                        // resulting heuristic score of current player
                        H::evaluate_move(state, &mv, heuristic_config)
                            .get_player_score(G::current_player(state)),
                        mv,
                    )
                })
                .collect::<Vec<_>>();
            heuristic_moves.shuffle(&mut rng());
            heuristic_moves.sort_unstable_by(|a, b| {
                a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
            });
            let mv = heuristic_moves.pop().map(|(_, m)| m);
            self.unexpanded_moves = Some(heuristic_moves);
            mv
        } else {
            let mut moves = G::available_moves(state)
                .map(|mv| (0.0, mv))
                .collect::<Vec<_>>();
            moves.shuffle(&mut rng());
            let mv = moves.pop().map(|(_, mv)| mv);
            self.unexpanded_moves = Some(moves);
            mv
        }
    }
    fn is_fully_expanded(&self) -> bool {
        if let Some(moves) = &self.unexpanded_moves {
            moves.is_empty()
        } else {
            false
        }
    }
}
