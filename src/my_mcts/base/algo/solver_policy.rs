// implementation of trait SolverPolicy

use super::{MCTSGame, SolverPolicy, WinDrawLoss};

#[derive(Debug, Clone, Copy, Default)]
pub struct NoSolver {}

impl<G> SolverPolicy<G> for NoSolver
where
    G: MCTSGame,
{
    fn set_solved(&mut self, _outcome: G::Outcome) {}
}

#[derive(Debug, Clone, Copy)]
pub struct SolverBounds {
    pessimistic: WinDrawLoss,
    optimistic: WinDrawLoss,
}

impl Default for SolverBounds {
    fn default() -> Self {
        Self {
            pessimistic: WinDrawLoss::Loss,
            optimistic: WinDrawLoss::Win,
        }
    }
}

impl<G> SolverPolicy<G> for SolverBounds
where
    G: MCTSGame<Outcome = WinDrawLoss>,
{
    fn set_solved(&mut self, outcome: G::Outcome) {
        self.optimistic = outcome;
        self.pessimistic = outcome;
    }
    fn get_solved(&self) -> Option<G::Outcome> {
        if self.optimistic == self.pessimistic {
            Some(self.optimistic)
        } else {
            None
        }
    }
}
