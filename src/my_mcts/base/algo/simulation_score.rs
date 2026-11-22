// implementing SimulationScore for TwoPlayer and TwoPlayerOutcome

use super::{MCTSGame, SimulationScore, TwoPlayer, TwoPlayerOutcome};

#[derive(Debug, Clone, Copy)]
pub struct TwoPlayerScore {
    pub first: f32,
    pub second: f32,
}

impl Default for TwoPlayerScore {
    fn default() -> Self {
        TwoPlayerScore {
            first: 0.5,
            second: 0.5,
        }
    }
}

impl From<TwoPlayerOutcome> for TwoPlayerScore {
    fn from(value: TwoPlayerOutcome) -> Self {
        match value {
            TwoPlayerOutcome::Draw => TwoPlayerScore::default(),
            TwoPlayerOutcome::FirstIsWinner => TwoPlayerScore {
                first: 1.0,
                second: 0.0,
            },
            TwoPlayerOutcome::SecondIsWinner => TwoPlayerScore {
                first: 0.0,
                second: 1.0,
            },
        }
    }
}

impl<G> SimulationScore<G> for TwoPlayerScore
where
    G: MCTSGame<Player = TwoPlayer, Outcome = TwoPlayerOutcome>,
{
    fn get_player_score(&self, player: G::Player) -> f32 {
        match player {
            TwoPlayer::First => self.first,
            TwoPlayer::Second => self.second,
        }
    }
}
