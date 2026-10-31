// implementation of trait SolverPolicy

use super::{MCTSGame, SolverPolicy, WinDrawLoss};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoSolver {}

impl<G> SolverPolicy<G> for NoSolver
where
    G: MCTSGame,
{
    fn set_solved(&mut self, _outcome: G::Outcome, _previous_player: G::Player) {}
    fn new_from_children<'a>(_children: impl IntoIterator<Item = &'a Self>) -> Self
    where
        Self: 'a,
    {
        NoSolver {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl SolverBounds {
    fn best_for_parent(&self) -> WinDrawLoss {
        -self.pessimistic
    }
    fn worst_for_parent(&self) -> WinDrawLoss {
        -self.optimistic
    }
}

impl<G> SolverPolicy<G> for SolverBounds
where
    G: MCTSGame<Outcome = WinDrawLoss>,
{
    fn set_solved(&mut self, outcome: G::Outcome, previous_player: G::Player) {
        if previous_player == G::perspective_player() {
            self.optimistic = outcome;
            self.pessimistic = outcome;
        } else {
            self.optimistic = -outcome;
            self.pessimistic = -outcome;
        }
    }
    fn get_solved(&self, previous_player: G::Player) -> Option<G::Outcome> {
        if self.optimistic == self.pessimistic {
            if previous_player == G::perspective_player() {
                Some(self.optimistic)
            } else {
                Some(-self.optimistic)
            }
        } else {
            None
        }
    }
    fn new_from_children<'a>(children: impl IntoIterator<Item = &'a Self>) -> Self
    where
        Self: 'a,
    {
        let (optimistic, pessimistic) = children
            .into_iter()
            .fold((WinDrawLoss::Loss, WinDrawLoss::Loss), |(bo, wo), c| {
                (bo.max(c.best_for_parent()), wo.max(c.worst_for_parent()))
            });
        Self {
            pessimistic,
            optimistic,
        }
    }
    fn update(&mut self, update: Self) -> bool {
        let out = self.optimistic != update.optimistic || self.pessimistic != update.pessimistic;
        out
    }
    fn filter_selection(&self, child: &Self) -> bool {
        child.best_for_parent() > self.pessimistic
    }
}
