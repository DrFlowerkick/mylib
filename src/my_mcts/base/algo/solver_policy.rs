// implementation of trait SolverPolicy

use super::{GameOutcome, MCTSGame, SolverPolicy};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoSolver {}

impl<G> SolverPolicy<G> for NoSolver
where
    G: MCTSGame,
{
    fn set_outcome(&mut self, _outcome: G::Outcome, _current_player: G::Player) {}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SolverBounds<G: MCTSGame> {
    pessimistic: G::Outcome,
    optimistic: G::Outcome,
}

impl<G: MCTSGame> Default for SolverBounds<G> {
    fn default() -> Self {
        Self {
            pessimistic: G::Outcome::WORST_OUTCOME,
            optimistic: G::Outcome::BEST_OUTCOME,
        }
    }
}

impl<G: MCTSGame> SolverBounds<G> {
    fn best_for_parent(&self) -> G::Outcome {
        -self.pessimistic
    }
    fn worst_for_parent(&self) -> G::Outcome {
        -self.optimistic
    }
}

impl<G: MCTSGame> SolverPolicy<G> for SolverBounds<G> {
    fn set_outcome(&mut self, outcome: G::Outcome, current_player: G::Player) {
        if current_player == G::perspective_player() {
            self.optimistic = outcome;
            self.pessimistic = outcome;
        } else {
            self.optimistic = -outcome;
            self.pessimistic = -outcome;
        }
    }
    fn get_outcome(&self, current_player: G::Player) -> Option<G::Outcome> {
        if self.optimistic == self.pessimistic {
            if current_player == G::perspective_player() {
                Some(self.optimistic)
            } else {
                Some(-self.optimistic)
            }
        } else {
            None
        }
    }
    fn filter_selection(&self, child: &Self) -> bool {
        child.best_for_parent() > self.pessimistic
    }
    fn update<'a>(
        &'a mut self,
        children: impl Iterator<Item = &'a Self>,
        is_fully_expanded: bool,
    ) -> bool {
        let (optimistic, pessimistic) = children.fold(
            (
                if is_fully_expanded {
                    G::Outcome::WORST_OUTCOME
                } else {
                    // unexpanded children give always an opportunity to Win!
                    G::Outcome::BEST_OUTCOME
                },
                G::Outcome::WORST_OUTCOME,
            ),
            |(o, p), c| (o.max(c.best_for_parent()), p.max(c.worst_for_parent())),
        );
        let out = self.optimistic != optimistic || self.pessimistic != pessimistic;
        self.optimistic = optimistic;
        self.pessimistic = pessimistic;
        out
    }
    fn get_guaranteed_outcome(&self) -> Option<G::Outcome> {
        Some(self.worst_for_parent())
    }
    fn filter_select_move(&self, child: &Self) -> bool {
        // if certain loss, do not filter. Otherwise
        // ignore moves resulting in certain loss and
        // keep all moves, which yield at least best guaranteed outcome (pessimistic)
        self.optimistic == G::Outcome::WORST_OUTCOME
            || (child.best_for_parent() > G::Outcome::WORST_OUTCOME
                && child.worst_for_parent() >= self.pessimistic)
    }
}
