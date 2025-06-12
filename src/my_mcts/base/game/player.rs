// TwoPlayer can be used for all two player games. Who would have thought...

use super::GamePlayer;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum TwoPlayer {
    #[default]
    First,
    Second,
}

impl TwoPlayer {
    pub fn next_player(&self) -> Self {
        match self {
            TwoPlayer::First => TwoPlayer::Second,
            TwoPlayer::Second => TwoPlayer::First,
        }
    }
}

impl GamePlayer for TwoPlayer {
    fn next(&self) -> Self {
        match self {
            TwoPlayer::First => TwoPlayer::Second,
            TwoPlayer::Second => TwoPlayer::First,
        }
    }
}
