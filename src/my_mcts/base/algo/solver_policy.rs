// implementation of trait SolverPolicy

use super::{MCTSGame, SolverPolicy, TwoPlayer, TwoPlayerOutcome};
use std::ops::Neg;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoSolver {}

impl<G> SolverPolicy<G> for NoSolver
where
    G: MCTSGame,
{
    fn set_outcome(&mut self, _outcome: G::Outcome, _current_player: G::Player) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WinDrawLoss {
    Loss,
    Draw,
    Win,
}

impl Neg for WinDrawLoss {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            WinDrawLoss::Draw => WinDrawLoss::Draw,
            WinDrawLoss::Loss => WinDrawLoss::Win,
            WinDrawLoss::Win => WinDrawLoss::Loss,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SolverBounds<G: MCTSGame<Player = TwoPlayer, Outcome = TwoPlayerOutcome>> {
    pessimistic: WinDrawLoss,
    optimistic: WinDrawLoss,
    phantom: std::marker::PhantomData<G>,
}

impl<G: MCTSGame<Player = TwoPlayer, Outcome = TwoPlayerOutcome>> Default for SolverBounds<G> {
    fn default() -> Self {
        Self {
            pessimistic: WinDrawLoss::Loss,
            optimistic: WinDrawLoss::Win,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G: MCTSGame<Player = TwoPlayer, Outcome = TwoPlayerOutcome>> SolverBounds<G> {
    fn best_for_parent(&self) -> WinDrawLoss {
        -self.pessimistic
    }
    fn worst_for_parent(&self) -> WinDrawLoss {
        -self.optimistic
    }
}

impl<G: MCTSGame<Player = TwoPlayer, Outcome = TwoPlayerOutcome>> SolverPolicy<G>
    for SolverBounds<G>
{
    fn set_outcome(&mut self, outcome: G::Outcome, current_player: G::Player) {
        match (outcome, current_player) {
            (TwoPlayerOutcome::Draw, _) => {
                self.pessimistic = WinDrawLoss::Draw;
                self.optimistic = WinDrawLoss::Draw;
            }
            (TwoPlayerOutcome::FirstIsWinner, TwoPlayer::First)
            | (TwoPlayerOutcome::SecondIsWinner, TwoPlayer::Second) => {
                self.pessimistic = WinDrawLoss::Win;
                self.optimistic = WinDrawLoss::Win;
            }
            (TwoPlayerOutcome::SecondIsWinner, TwoPlayer::First)
            | (TwoPlayerOutcome::FirstIsWinner, TwoPlayer::Second) => {
                self.pessimistic = WinDrawLoss::Loss;
                self.optimistic = WinDrawLoss::Loss;
            }
        }
    }
    fn get_outcome(&self, current_player: G::Player) -> Option<G::Outcome> {
        if self.optimistic == self.pessimistic {
            self.get_guaranteed_outcome(current_player)
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
                    WinDrawLoss::Loss
                } else {
                    // unexpanded children give always an opportunity to Win!
                    WinDrawLoss::Win
                },
                WinDrawLoss::Loss,
            ),
            |(o, p), c| (o.max(c.best_for_parent()), p.max(c.worst_for_parent())),
        );
        let out = self.optimistic != optimistic || self.pessimistic != pessimistic;
        self.optimistic = optimistic;
        self.pessimistic = pessimistic;
        out
    }
    fn get_guaranteed_outcome(&self, current_player: G::Player) -> Option<G::Outcome> {
        Some(match (self.pessimistic, current_player) {
            (WinDrawLoss::Draw, _) => TwoPlayerOutcome::Draw,
            (WinDrawLoss::Win, TwoPlayer::First) | (WinDrawLoss::Loss, TwoPlayer::Second) => {
                TwoPlayerOutcome::FirstIsWinner
            }
            (WinDrawLoss::Win, TwoPlayer::Second) | (WinDrawLoss::Loss, TwoPlayer::First) => {
                TwoPlayerOutcome::SecondIsWinner
            }
        })
    }
    fn filter_select_move(&self, child: &Self) -> bool {
        // if certain loss, do not filter. Otherwise
        // ignore moves resulting in certain loss and
        // keep all moves, which yield at least best guaranteed outcome (pessimistic)
        self.optimistic == WinDrawLoss::Loss
            || (child.best_for_parent() > WinDrawLoss::Loss
                && child.worst_for_parent() >= self.pessimistic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ord() {
        assert!(WinDrawLoss::Win > WinDrawLoss::Draw);
        assert!(WinDrawLoss::Draw > WinDrawLoss::Loss);
        assert!(WinDrawLoss::Win == WinDrawLoss::Win);
    }
}
