// miscellaneous mcts type definitions

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum MonteCarloPlayer {
    #[default]
    Me,
    Opp,
}

impl MonteCarloPlayer {
    pub fn next_player(&self) -> Self {
        match self {
            MonteCarloPlayer::Me => MonteCarloPlayer::Opp,
            MonteCarloPlayer::Opp => MonteCarloPlayer::Me,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MonteCarloNodeType {
    GameDataUpdate,
    ActionResult,
}

#[derive(Copy, Clone, PartialEq)]
// each game mode describes a different handling of player actions, see below
// normally each player has one action
// if multiple actions per player are possible, than starting_player does his actions, afterward the other player. this is true for every mode
pub enum MonteCarloGameMode {
    SameTurnParallel, // both players act parallel on same turn. Actions change game data at the same time
    ByTurns,          // each turn only one player acts, players switch at turn end
}

#[derive(Copy, Clone, PartialEq)]
pub enum MonteCarloNodeConsistency {
    Inconsistent,
    Consistent,
    ConsistentNeedsUpdate,
    PossibleFutureGameState,
}
