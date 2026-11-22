// base implementation of GameOutcome

#[derive(Debug, Clone, Copy)]
pub enum TwoPlayerOutcome {
    FirstIsWinner,
    SecondIsWinner,
    Draw,
}
