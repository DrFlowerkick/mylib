// base implementation of GameOutcome

use std::ops::Neg;

use super::GameOutcome;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WinDrawLoss {
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

impl GameOutcome for WinDrawLoss {
    fn to_score(&self) -> f32 {
        match self {
            WinDrawLoss::Loss => 0.0,
            WinDrawLoss::Draw => 0.5,
            WinDrawLoss::Win => 1.0,
        }
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
