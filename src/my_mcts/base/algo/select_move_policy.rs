// implementations of SelectMovePolicy

use super::{MCTSGame, MoveStats, SelectMovePolicy};

#[derive(Clone)]
pub struct MostVisited;

impl<G> SelectMovePolicy<G> for MostVisited
where
    G: MCTSGame,
{
    fn select<'a, I>(c: I) -> Option<&'a G::Move>
    where
        I: Iterator<Item = (&'a G::Move, MoveStats)>,
    {
        c.max_by(|(_, a), (_, b)| a.visits_from_parent.cmp(&b.visits_from_parent))
            .map(|(m, _)| m)
    }
}

#[derive(Clone)]
pub struct MostVisitedAndMaxMean;

impl<G> SelectMovePolicy<G> for MostVisitedAndMaxMean
where
    G: MCTSGame,
{
    fn select<'a, I>(c: I) -> Option<&'a G::Move>
    where
        I: Iterator<Item = (&'a G::Move, MoveStats)>,
    {
        c.max_by(|(_, a), (_, b)| {
            a.visits_from_parent
                .cmp(&b.visits_from_parent)
                .then_with(|| {
                    (a.accumulated_value / a.acc_visits as f32)
                        .total_cmp(&(b.accumulated_value / b.acc_visits as f32))
                })
        })
        .map(|(m, _)| m)
    }
}
