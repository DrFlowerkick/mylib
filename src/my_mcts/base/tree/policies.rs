// implementation of tree policy traits

use super::{
    ExploitationAndExploration, ExploitationAndExplorationWithScoreCache, ExploitationOnly,
    ExploitationOnlyWithScoreCache, ExplorationOnly, ExplorationOnlyWithScoreCache, Heuristic,
    MCTSAlgo, MCTSGame, NoUCTCache, UCTStatsPolicy,
};

#[derive(Clone)]
pub struct NodeOnly;

impl<G, H, A> UCTStatsPolicy<G, H, A> for NodeOnly
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node = ExploitationAndExploration;
    type Edge = NoUCTCache;
}

#[derive(Clone)]
pub struct EdgeOnly;

impl<G, H, A> UCTStatsPolicy<G, H, A> for EdgeOnly
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node = NoUCTCache;
    type Edge = ExploitationAndExploration;
}

#[derive(Clone)]
pub struct Hybrid;

impl<G, H, A> UCTStatsPolicy<G, H, A> for Hybrid
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node = ExploitationOnly;
    type Edge = ExplorationOnly;
}

#[derive(Clone)]
pub struct NodeOnlyWithScoreCache;

impl<G, H, A> UCTStatsPolicy<G, H, A> for NodeOnlyWithScoreCache
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node = ExploitationAndExplorationWithScoreCache;
    type Edge = NoUCTCache;
}

#[derive(Clone)]
pub struct EdgeOnlyWithScoreCache;

impl<G, H, A> UCTStatsPolicy<G, H, A> for EdgeOnlyWithScoreCache
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node = NoUCTCache;
    type Edge = ExploitationAndExplorationWithScoreCache;
}

#[derive(Clone)]
pub struct HybridWithScoreCache;

impl<G, H, A> UCTStatsPolicy<G, H, A> for HybridWithScoreCache
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node = ExploitationOnlyWithScoreCache;
    type Edge = ExplorationOnlyWithScoreCache;
}
