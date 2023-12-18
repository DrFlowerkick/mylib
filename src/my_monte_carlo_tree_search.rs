pub mod game_engine;

use rand::prelude::*;
use rand::seq::IteratorRandom;
use std::any::Any;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use crate::my_tree::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MonteCarloPlayer {
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

// Trait for actions players can take to interact with game data.
pub trait MonteCarloPlayerAction: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(player_action: &impl MonteCarloPlayerAction) -> &Self;
    fn iter_actions(
        game_data: &impl MonteCarloGameData,
        player: MonteCarloPlayer,
        parent_game_turn: usize,
    ) -> Box<dyn Iterator<Item = Self> + '_>;
}

// Trait for updating game data after modifications through players. Normally there as some kind of random factor involved, e.g. drawing new ressources of several kind from a "bag".
pub trait MonteCarloGameDataUpdate: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(game_data_update: &impl MonteCarloGameDataUpdate) -> &Self;
    fn iter_game_data_updates(
        game_data: &impl MonteCarloGameData,
        force_update: bool,
    ) -> Box<dyn Iterator<Item = Self> + '_>;
}

// trait for game data, which works with Monte Carlo Tree Search
pub trait MonteCarloGameData: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(game_data: &impl MonteCarloGameData) -> &Self;
    fn apply_my_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool; // true if score event, which results in change of heuristic
    fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool; // true if score event, which results in change of heuristic
    fn simultaneous_player_actions_for_simultaneous_game_data_change(
        &mut self,
        my_action: &impl MonteCarloPlayerAction,
        opp_action: &impl MonteCarloPlayerAction,
    );
    fn is_game_data_update_required(&self, force_update: bool) -> bool;
    fn apply_game_data_update(
        &mut self,
        game_data_update: &impl MonteCarloGameDataUpdate,
        check_update_consistency: bool,
    ) -> bool; // true if consistent
    fn calc_heuristic(&self) -> f32;
    fn check_game_ending(&self, game_turn: usize) -> bool;
    fn game_winner(&self, game_turn: usize) -> Option<MonteCarloPlayer>; // None if tie
    fn check_consistency_of_game_data_during_init_root(
        &mut self,
        current_game_state: &Self,
        played_turns: usize,
    ) -> bool;
    fn check_consistency_of_game_data_update(
        &mut self,
        current_game_state: &Self,
        game_data_update: &impl MonteCarloGameDataUpdate,
        played_turns: usize,
    ) -> bool;
    fn check_consistency_of_action_result(
        &mut self,
        current_game_state: Self,
        my_action: &impl MonteCarloPlayerAction,
        opp_action: &impl MonteCarloPlayerAction,
        played_turns: usize,
        apply_player_actions_to_game_data: bool,
    ) -> bool;
}

// "G" is a trait object for a game data
#[derive(PartialEq, Clone, Copy)]
pub struct MonteCarloNode<
    G: MonteCarloGameData,
    A: MonteCarloPlayerAction,
    U: MonteCarloGameDataUpdate,
> {
    game_data: G,
    player_action: A,
    game_data_update: U,
    node_type: MonteCarloNodeType,
    next_node: MonteCarloNodeType,
    player: MonteCarloPlayer,
    game_turn: usize,
    heuristic: f32,
    alpha: f32,
    beta: f32,
    wins: f32,
    samples: f32,
    parent_samples: f32,
    exploitation_score: f32, // exploitation_score is needed to choose best action and to choose node to exploit
    exploration_score: f32,  // exploration_score is needed to identify nodes for exploration
    heuristic_score: f32,
    total_score: f32,
    pruned_node: bool,
    game_end_node: bool, // leave, at which the game ends
}

impl<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate>
    MonteCarloNode<G, A, U>
{
    fn new() -> Self {
        MonteCarloNode {
            game_data: G::default(),
            player_action: A::default(),
            game_data_update: U::default(),
            node_type: MonteCarloNodeType::ActionResult,
            next_node: MonteCarloNodeType::ActionResult,
            player: MonteCarloPlayer::Me,
            game_turn: 0,
            heuristic: 0.0,
            alpha: f32::INFINITY,
            beta: f32::NEG_INFINITY,
            wins: 0.0,
            samples: f32::NAN,
            parent_samples: 0.0,
            exploitation_score: 0.0,
            exploration_score: 0.0,
            heuristic_score: 0.0,
            total_score: 0.0,
            pruned_node: false,
            game_end_node: false,
        }
    }
    fn new_player_action_child(&self, player_action: A) -> Self {
        let mut new_child = Self::new();
        new_child.player_action = player_action;
        new_child.parent_samples = self.samples;
        new_child.game_turn = self.game_turn;
        new_child.player = self.player;
        new_child
    }
    fn new_game_data_update_child(&self, game_data_update: U) -> Self {
        let mut new_child = Self::new();
        new_child.game_data_update = game_data_update;
        new_child.parent_samples = self.samples;
        new_child.game_turn = self.game_turn;
        new_child.player = self.player;
        new_child.node_type = MonteCarloNodeType::GameDataUpdate;
        new_child
    }

    fn calc_heuristic(&mut self, use_heuristic_score: bool) {
        if use_heuristic_score {
            self.heuristic = self.game_data.calc_heuristic();
            match self.player {
                MonteCarloPlayer::Me => self.alpha = self.heuristic,
                MonteCarloPlayer::Opp => self.beta = self.heuristic,
            }
        }
    }
    fn calc_node_score(&mut self, parent_samples: f32, weighting_factor: f32) {
        if parent_samples != self.parent_samples {
            self.update_exploration_score(parent_samples, weighting_factor);
        }
        self.total_score = match self.player {
            MonteCarloPlayer::Me => {
                self.exploitation_score + self.exploration_score - self.heuristic_score
            }
            MonteCarloPlayer::Opp => {
                self.exploitation_score + self.exploration_score + self.heuristic_score
            }
        };
    }

    fn check_game_turn(&mut self, game_mode: MonteCarloGameMode) {
        match game_mode {
            MonteCarloGameMode::SameTurnParallel => {
                if self.player == MonteCarloPlayer::Opp {
                    self.game_turn += 1;
                }
            }
            MonteCarloGameMode::ByTurns => self.game_turn += 1,
        }
    }

    fn set_next_node(&mut self, force_update: bool) {
        if !self.game_end_node {
            self.next_node = if self.game_data.is_game_data_update_required(force_update) {
                MonteCarloNodeType::GameDataUpdate
            } else {
                MonteCarloNodeType::ActionResult
            };
        }
    }

    fn apply_action(
        &mut self,
        parent_game_data: &G,
        parent_action: &A,
        game_mode: MonteCarloGameMode,
        max_number_of_turns: usize,
        use_heuristic_score: bool,
    ) -> bool {
        // transfer game_data of parent
        self.game_data = *parent_game_data;
        self.samples = 0.0;
        // score_event depends on player action (e.g. scoring points) or end of game
        let mut score_event = self.apply_player_action();
        self.player = self.player.next_player();
        self.check_game_turn(game_mode);
        match game_mode {
            MonteCarloGameMode::SameTurnParallel => {
                if self.player == MonteCarloPlayer::Me {
                    // first check if game ends
                    if self.check_game_ending(max_number_of_turns) {
                        self.calc_heuristic(use_heuristic_score);
                        return true; // save time by skipping all next code, since this is a game_end_node
                    }
                    self.game_data
                        .simultaneous_player_actions_for_simultaneous_game_data_change(
                            parent_action,
                            &self.player_action,
                        );
                }
            }
            MonteCarloGameMode::ByTurns => {
                score_event = self.check_game_ending(max_number_of_turns) || score_event;
            }
        }
        if score_event {
            self.calc_heuristic(use_heuristic_score);
        }
        score_event && use_heuristic_score
    }

    fn apply_game_data_update(
        &mut self,
        parent_game_data: &G,
        check_update_consistency: bool,
    ) -> bool {
        // transfer game_data of parent
        self.game_data = *parent_game_data;
        self.samples = 0.0;
        // apply update
        self.game_data
            .apply_game_data_update(&self.game_data_update, check_update_consistency)
    }

    fn apply_player_action(&mut self) -> bool {
        match self.player {
            MonteCarloPlayer::Me => self.game_data.apply_my_action(&self.player_action),
            MonteCarloPlayer::Opp => self.game_data.apply_opp_action(&self.player_action),
        }
    }

    fn check_game_ending(&mut self, max_number_of_turns: usize) -> bool {
        self.game_end_node = self.game_turn == max_number_of_turns
            || self.game_data.check_game_ending(self.game_turn);
        self.game_end_node
    }

    fn calc_playout_score(&self) -> f32 {
        match self.game_data.game_winner(self.game_turn) {
            Some(player) => match player {
                MonteCarloPlayer::Me => 1.0,
                MonteCarloPlayer::Opp => 0.0,
            },
            None => 0.5,
        }
    }

    fn score_playout_result(
        &mut self,
        playout_score: f32,
        samples: f32,
        use_heuristic_score: bool,
    ) {
        self.wins += playout_score;
        self.samples += samples;
        self.exploitation_score = match self.player {
            MonteCarloPlayer::Me => 1.0 - self.wins / self.samples,
            MonteCarloPlayer::Opp => self.wins / self.samples,
        };
        if use_heuristic_score {
            self.heuristic_score = match self.player {
                MonteCarloPlayer::Me => {
                    if self.alpha.is_finite() {
                        self.alpha / self.samples
                    } else {
                        0.0
                    }
                }
                MonteCarloPlayer::Opp => {
                    if self.beta.is_finite() {
                        self.beta / self.samples
                    } else {
                        0.0
                    }
                }
            };
        }
    }

    fn update_exploration_score(&mut self, parent_samples: f32, weighting_factor: f32) {
        self.parent_samples = parent_samples;
        self.exploration_score =
            weighting_factor * (self.parent_samples.log10() / self.samples).sqrt();
    }

    fn update_consistent_node_during_init_phase(
        &mut self,
        current_game_state: &G,
        played_turns: usize,
        force_update: bool,
    ) -> bool {
        if !force_update {
            if !self
                .game_data
                .check_consistency_of_game_data_during_init_root(current_game_state, played_turns)
            {
                return false;
            }
        }
        self.game_data == *current_game_state
    }
}

pub struct MonteCarloTreeSearch<
    G: MonteCarloGameData,
    A: MonteCarloPlayerAction,
    U: MonteCarloGameDataUpdate,
> {
    tree_root: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
    keep_root: Option<Rc<TreeNode<MonteCarloNode<G, A, U>>>>,
    root_level: usize,
    game_mode: MonteCarloGameMode,
    starting_player: MonteCarloPlayer,
    played_turns: usize,
    max_number_of_turns: usize,
    force_update: bool,
    first_turn: bool,
    time_out_first_turn: Duration,
    time_out_successive_turns: Duration,
    weighting_factor: f32,
    use_heuristic_score: bool,
    debug: bool,
}

impl<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate>
    MonteCarloTreeSearch<G, A, U>
{
    pub fn new(
        game_mode: MonteCarloGameMode,
        max_number_of_turns: usize,
        force_update: bool,
        time_out_first_turn: Duration,
        time_out_successive_turns: Duration,
        weighting_factor: f32,
        use_heuristic_score: bool,
        debug: bool,
        keep_root: bool,
    ) -> Self {
        let mut result = MonteCarloTreeSearch {
            tree_root: TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0),
            keep_root: None,
            root_level: 0,
            game_mode,
            starting_player: MonteCarloPlayer::Me,
            played_turns: 0,
            max_number_of_turns,
            force_update,
            first_turn: true,
            time_out_first_turn,
            time_out_successive_turns,
            weighting_factor, // try starting with 1.0 and find a way to applicate a better value
            use_heuristic_score,
            debug,
        };
        if keep_root {
            result.keep_root = Some(result.tree_root.clone());
        }
        result
    }
    pub fn init_root(&mut self, game_data: &G, starting_player: MonteCarloPlayer) -> Instant {
        let start = Instant::now();
        if self.first_turn {
            self.starting_player = starting_player;
            // init root with initial game data
            self.tree_root.get_mut_value().game_data = *game_data;
            self.tree_root.get_mut_value().samples = 0.0;
            if self.game_mode == MonteCarloGameMode::ByTurns
                && self.starting_player == MonteCarloPlayer::Opp
            {
                // if opp is starting player, than with turn wise actions opp player already played a turn
                self.played_turns = 1;
                self.tree_root.get_mut_value().game_turn = 1;
                self.tree_root.get_mut_value().player = MonteCarloPlayer::Me;
            } else {
                // no action made yet: tree_root represents initial game data
                self.tree_root.get_mut_value().node_type = MonteCarloNodeType::GameDataUpdate;
                self.tree_root.get_mut_value().player = starting_player;
            }
        } else {
            // search new root node and move tree_root to it
            // root node is one node before next possible node with starting player as node owner
            let (search_turn, end_level) = match self.game_mode {
                MonteCarloGameMode::SameTurnParallel => (self.played_turns, Some(3)),
                MonteCarloGameMode::ByTurns => (self.played_turns + 1, Some(2)),
            };
            match self
                .tree_root
                .iter_level_order_traversal_with_bordes(1, end_level)
                .find(|(n, _)| {
                    let mut n_value = n.get_mut_value();
                    n_value.game_turn == search_turn
                        && n_value.next_node == MonteCarloNodeType::ActionResult
                        && n_value.player == MonteCarloPlayer::Me
                        && n_value.update_consistent_node_during_init_phase(
                            game_data,
                            self.played_turns,
                            self.force_update,
                        )
                }) {
                Some((new_root, _)) => {
                    self.tree_root = new_root;
                    self.root_level = self.tree_root.get_level();
                }
                None => {
                    // create new tree_root, since no node with game_data has been found
                    if self.debug {
                        eprintln!("Current game state not found in tree. Reinit tree after {} played turns", self.played_turns);
                    }
                    if self.keep_root.is_some() {
                        panic!("quit since root has been reset.");
                    }
                    self.tree_root = TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0);
                    self.root_level = 0;
                    self.tree_root.get_mut_value().game_data = *game_data;
                    self.tree_root.get_mut_value().samples = 0.0;
                    self.tree_root.get_mut_value().player = MonteCarloPlayer::Me;
                    self.tree_root.get_mut_value().game_turn = search_turn;
                }
            }
        }
        start
    }
    pub fn expand_tree(&mut self, start: Instant) {
        let time_out = if self.first_turn {
            self.first_turn = false;
            self.time_out_first_turn
        } else {
            self.time_out_successive_turns
        };
        // loop until time out or no more nodes to cycle
        let mut counter = 0;
        while start.elapsed() < time_out && !self.one_cycle(&start, time_out) {
            counter += 1;
        }
        if self.debug {
            eprintln!("number of expand cycles: {}", counter);
        }
    }
    pub fn choose_and_execute_actions(
        &mut self,
    ) -> (impl MonteCarloGameData, impl MonteCarloPlayerAction) {
        // my best action is at max exploitation_score
        let child = self
            .tree_root
            .iter_children()
            .max_by(|x, y| {
                x.get_value()
                    .exploitation_score
                    .partial_cmp(&y.get_value().exploitation_score)
                    .unwrap()
            })
            .unwrap();
        self.played_turns = child.get_value().game_turn;
        self.tree_root = child.clone();
        self.root_level = self.tree_root.get_level();
        // return game_data and my action
        let result = (child.get_value().game_data, child.get_value().player_action);
        result
    }

    fn one_cycle(&self, start: &Instant, time_out: Duration) -> bool {
        let selection_node = self.selection(start, time_out);
        match selection_node {
            Some(selection_node) => {
                let child_node = self.expansion(selection_node);
                match self.playout(child_node.clone(), start, time_out) {
                    Some((playout_score, backtrack_heuristic)) => {
                        self.propagation(child_node, playout_score, backtrack_heuristic)
                    }
                    None => (),
                }
            }
            None => return true, // no more nodes to simulate in tree or time over
        }
        false
    }

    fn selection(
        &self,
        start: &Instant,
        time_out: Duration,
    ) -> Option<Rc<TreeNode<MonteCarloNode<G, A, U>>>> {
        let mut rng = thread_rng();
        // search for node to select
        let mut selection_node = self.tree_root.clone();
        while !selection_node.is_leave() {
            if start.elapsed() >= time_out {
                // return None, if selection cannot finish in time
                return None;
            }
            // remove inconsistent children, if next_node is GameDataUpdate
            // if consistent child is detected it will be updated
            // if all children removed, return selection_node
            if self.remove_inconsistent_children(selection_node.clone()) {
                return Some(selection_node);
            }

            // search children without samples
            match selection_node
                .iter_children()
                .filter(|c| c.get_value().samples.is_nan())
                .choose(&mut rng)
            {
                Some(child_without_samples) => return Some(child_without_samples),
                None => (),
            }
            selection_node.iter_children().for_each(|c| {
                c.get_mut_value()
                    .calc_node_score(selection_node.get_value().samples, self.weighting_factor)
            });
            let selected_child = selection_node.iter_children().max_by(|a, b| {
                a.get_value()
                    .total_score
                    .partial_cmp(&b.get_value().total_score)
                    .unwrap()
            });
            selection_node = match selected_child {
                Some(child) => {
                    if self.force_update {
                        child.clone()
                    } else {
                        let node_type = child.get_value().node_type;
                        match node_type {
                            MonteCarloNodeType::ActionResult => {
                                // update child with parent game state (if no update is needed, nothing happens)
                                let child_action = child.get_value().player_action;
                                let apply_player_actions_to_game_data = match self.game_mode {
                                    MonteCarloGameMode::SameTurnParallel => {
                                        child.get_value().player == MonteCarloPlayer::Me
                                    }
                                    MonteCarloGameMode::ByTurns => true,
                                };
                                let child_game_data_changed = child
                                    .get_mut_value()
                                    .game_data
                                    .check_consistency_of_action_result(
                                        selection_node.get_value().game_data,
                                        &selection_node.get_value().player_action,
                                        &child_action,
                                        self.played_turns,
                                        apply_player_actions_to_game_data,
                                    );
                                if child_game_data_changed
                                    && child.get_value().next_node
                                        == MonteCarloNodeType::GameDataUpdate
                                    && child.is_leave()
                                {
                                    child.get_mut_value().set_next_node(self.force_update);
                                }
                                child.clone()
                            }
                            MonteCarloNodeType::GameDataUpdate => child.clone(),
                        }
                    }
                }
                None => panic!("selection should alway find a child!"),
            };
        }
        Some(selection_node)
    }

    fn expansion(
        &self,
        expansion_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
    ) -> Rc<TreeNode<MonteCarloNode<G, A, U>>> {
        if expansion_node.get_value().game_end_node
            || (expansion_node.get_level() > self.root_level
                && expansion_node.get_value().samples.is_nan())
        {
            return expansion_node;
        }

        let next_node = expansion_node.get_value().next_node;
        match next_node {
            MonteCarloNodeType::GameDataUpdate => {
                for game_data_update in U::iter_game_data_updates(
                    &expansion_node.get_value().game_data,
                    self.force_update,
                ) {
                    let new_game_data_update_node = expansion_node
                        .get_value()
                        .new_game_data_update_child(game_data_update);
                    expansion_node.add_child(new_game_data_update_node, 0);
                }
            }
            MonteCarloNodeType::ActionResult => {
                for player_action in A::iter_actions(
                    &expansion_node.get_value().game_data,
                    expansion_node.get_value().player,
                    expansion_node.get_value().game_turn,
                ) {
                    let new_player_action_node = expansion_node
                        .get_value()
                        .new_player_action_child(player_action);
                    expansion_node.add_child(new_player_action_node, 0);
                }
            }
        }
        expansion_node.get_child(0).unwrap()
    }

    fn playout(
        &self,
        playout_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
        start: &Instant,
        time_out: Duration,
    ) -> Option<(f32, bool)> {
        if playout_node.get_value().game_end_node {
            Some((playout_node.get_value().calc_playout_score(), false))
        } else {
            let node_type = playout_node.get_value().node_type;
            let parent = playout_node.get_parent().unwrap();
            let backtrack_heuristic = match node_type {
                MonteCarloNodeType::GameDataUpdate => {
                    if !playout_node
                        .get_mut_value()
                        .apply_game_data_update(&parent.get_value().game_data, !self.force_update)
                    {
                        // node is inconsistent -> delete this node from parent and search for new child
                        let index = parent
                            .iter_children()
                            .position(|c| *c.get_value() == *playout_node.get_value())
                            .unwrap();
                        parent.swap_remove_child(index);
                        return None;
                    }
                    playout_node
                        .get_mut_value()
                        .set_next_node(self.force_update);
                    false
                }
                MonteCarloNodeType::ActionResult => {
                    let parent_action = parent.get_value().player_action;
                    let backtrack_heuristic = playout_node.get_mut_value().apply_action(
                        &parent.get_value().game_data,
                        &parent_action,
                        self.game_mode,
                        self.max_number_of_turns,
                        self.use_heuristic_score,
                    );
                    playout_node
                        .get_mut_value()
                        .set_next_node(self.force_update);
                    backtrack_heuristic
                }
            };

            let mut rng = thread_rng();
            let mut playout = *playout_node.get_value();

            while !playout.game_end_node {
                if start.elapsed() >= time_out {
                    // return tie, if playout cannot finish in time
                    return None;
                }
                match playout.next_node {
                    MonteCarloNodeType::GameDataUpdate => {
                        // create new game game_data update
                        let parent_game_data = playout.game_data;
                        let game_data_update =
                            U::iter_game_data_updates(&playout.game_data, self.force_update)
                                .choose(&mut rng)
                                .unwrap();
                        playout = playout.new_game_data_update_child(game_data_update);
                        playout.apply_game_data_update(&parent_game_data, false);
                        playout.set_next_node(self.force_update);
                    }
                    MonteCarloNodeType::ActionResult => {
                        // set random next action
                        let parent_game_data = playout.game_data;
                        let parent_action = playout.player_action;
                        let player_action =
                            A::iter_actions(&playout.game_data, playout.player, playout.game_turn)
                                .choose(&mut rng)
                                .unwrap();
                        playout = playout.new_player_action_child(player_action);
                        playout.apply_action(
                            &parent_game_data,
                            &parent_action,
                            self.game_mode,
                            self.max_number_of_turns,
                            self.use_heuristic_score,
                        );
                        playout.set_next_node(self.force_update);
                    }
                }
            }
            Some((playout.calc_playout_score(), backtrack_heuristic))
        }
    }

    fn propagation(
        &self,
        start_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
        mut playout_score: f32,
        backtrack_heuristic: bool,
    ) {
        // score playout result and calc new exploitation score for start_node
        start_node.get_mut_value().score_playout_result(
            playout_score,
            1.0,
            self.use_heuristic_score,
        );
        // backtrack playout_score and heuristic if score event
        for node in start_node
            .iter_back_track()
            .skip(1)
            .filter(|n| n.get_level() >= self.root_level)
        {
            // first backtrack heuristic, since heuristic is used by score_playout_result()
            if backtrack_heuristic {
                // ToDo: how to do this with MonteCarloNodeType::GameDataUpdate
                let player = node.get_value().player;
                match player {
                    MonteCarloPlayer::Me => {
                        let max_beta = node
                            .iter_children()
                            .map(|c| c.get_value().beta)
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap();
                        node.get_mut_value().alpha = max_beta;
                    }
                    MonteCarloPlayer::Opp => {
                        let min_alpha = node
                            .iter_children()
                            .map(|c| c.get_value().alpha)
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap();
                        node.get_mut_value().beta = min_alpha;
                    }
                }
            }
            // do score_playout_result()
            if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate {
                let num_children = node.len_children() as f32;
                playout_score /= num_children;
            }
            // score playout result and calc new exploitation score
            node.get_mut_value()
                .score_playout_result(playout_score, 1.0, self.use_heuristic_score);
        }
    }

    fn reverse_propagation(
        &self,
        start_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
        mut wins: f32,
        mut samples: f32,
    ) {
        // remove samples and wins of inconsistent children and calc new exploitation score for start_node
        start_node
            .get_mut_value()
            .score_playout_result(wins, samples, self.use_heuristic_score);
        for node in start_node
            .iter_back_track()
            .skip(1)
            .filter(|n| n.get_level() >= self.root_level)
        {
            if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate {
                let num_children = node.len_children() as f32;
                wins /= num_children;
                samples /= num_children;
            }
            // remove samples and wins of inconsistent children and calc new exploitation score
            node.get_mut_value()
                .score_playout_result(wins, samples, self.use_heuristic_score);
        }
    }

    fn remove_inconsistent_children(
        &self,
        selection_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
    ) -> bool {
        if self.force_update
            || selection_node.get_value().next_node == MonteCarloNodeType::ActionResult
            || selection_node.len_children() == 1
        {
            return false;
        }

        let n_children = selection_node.len_children() as f32;
        let mut index = 0;
        let mut samples = 0.0;
        let mut wins = 0.0;
        let mut inconsistency_detected = false;
        while index < selection_node.len_children() {
            let child = selection_node.get_child(index).unwrap();
            // find child with samples
            if !child.get_value().samples.is_nan() {
                samples += child.get_value().samples;
                wins += child.get_value().wins;
                let child_game_data_update = child.get_value().game_data_update;
                if child
                    .get_mut_value()
                    .game_data
                    .check_consistency_of_game_data_update(
                        &selection_node.get_value().game_data,
                        &child_game_data_update,
                        self.played_turns,
                    )
                {
                    index += 1;
                } else {
                    selection_node.swap_remove_child(index);
                    inconsistency_detected = true;
                }
            } else {
                index += 1;
            }
        }

        if inconsistency_detected {
            // calc inconsistent playout results
            wins = -wins / n_children;
            samples = -samples / n_children;

            let consistent_child_index = selection_node
                .iter_children()
                .position(|c| !c.get_value().samples.is_nan());
            match consistent_child_index {
                Some(index) => {
                    // If inconsistent children were removed and a child with samples remains, only
                    // this child can be consistent, while all other children are inconsistent.
                    // It's wins and samples are valid and thus not removed by reverse_propagation.
                    wins += selection_node.get_child(index).unwrap().get_value().wins;
                    samples += selection_node.get_child(index).unwrap().get_value().samples;
                    self.reverse_propagation(selection_node.clone(), wins, samples);
                    // remove all other children, since they are inconsistent
                    selection_node.split_off_children(index, true);
                    selection_node.split_off_children(1, false);
                }
                None => {
                    // no consistent child with samples left -> remove all children and reset next node
                    self.reverse_propagation(selection_node.clone(), wins, samples);
                    selection_node.clear_children(0);
                    selection_node
                        .get_mut_value()
                        .set_next_node(self.force_update);
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::my_tic_tac_toe::mcts_tic_tac_toe::*;
    use crate::my_tic_tac_toe::*;

    use std::time::Duration;
    const MAX_NUMBER_OF_TURNS: usize = 9;
    const FORCE_UPDATE: bool = true;
    const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(1000);
    const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(100);
    const WEIGHTING_FACTOR: f32 = 50.0;
    const DEBUG: bool = true;
    const KEEP_ROOT: bool = true;

    #[test]
    fn test_tree_width_and_depth_opp_first() {
        let use_heuristic_score = false;
        let mut last_winner: Option<MonteCarloPlayer> = None;
        let mut wins = 0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut ttt_match = TicTacToeGameData::new();
            // let opp do 1. action by choosing a random action
            match ttt_match.choose_random_next_action() {
                Some(next_action) => {
                    ttt_match.apply_opp_action(&next_action);
                }
                None => (),
            }
            let mut mcts_player: MonteCarloTreeSearch<
                TicTacToeGameData,
                TicTacToePlayerAction,
                TicTacToeGameDataUpdate,
            > = MonteCarloTreeSearch::new(
                MonteCarloGameMode::ByTurns,
                MAX_NUMBER_OF_TURNS,
                FORCE_UPDATE,
                TIME_OUT_FIRST_TURN,
                TIME_OUT_SUCCESSIVE_TURNS,
                WEIGHTING_FACTOR,
                use_heuristic_score,
                DEBUG,
                KEEP_ROOT,
            );
            while !ttt_match.check_game_ending(0) {
                let start = mcts_player.init_root(&ttt_match, MonteCarloPlayer::Opp);
                mcts_player.expand_tree(start);
                eprint!("me  ");
                let (current_game_data, _) = mcts_player.choose_and_execute_actions();
                let parent = mcts_player.tree_root.get_parent().unwrap();
                for child in parent.iter_children() {
                    let child_node = child.get_value();
                    let child_action =
                        TicTacToePlayerAction::downcast_self(&child_node.player_action);
                    eprintln!("({}, {}): w: {:.1}, s: {:.0}, ets: {:.2}, ers: {:.2}, hs: {:.2}, beta: {:.0}", child_action.cell.x(), child_action.cell.y(), child_node.wins, child_node.samples, child_node.exploitation_score, child_node.exploration_score, child_node.heuristic_score, child_node.beta);
                }
                eprintln!("opp options:");
                for child in mcts_player.tree_root.iter_children() {
                    let child_node = child.get_value();
                    let child_action =
                        TicTacToePlayerAction::downcast_self(&child_node.player_action);
                    eprintln!("({}, {}): w: {:.1}, s: {:.0}, ets: {:.2}, ers: {:.2}, hs: {:.2}, alpha: {:.0}", child_action.cell.x(), child_action.cell.y(), child_node.wins, child_node.samples, child_node.exploitation_score, child_node.exploration_score, child_node.heuristic_score, child_node.alpha);
                }
                ttt_match = *TicTacToeGameData::downcast_self(&current_game_data);
                if !ttt_match.check_game_ending(0) {
                    // let opp act by choosing a random actions
                    match ttt_match.choose_random_next_action() {
                        Some(next_action) => {
                            ttt_match.apply_opp_action(&next_action);
                        }
                        None => (),
                    }
                }
            }
            last_winner = ttt_match.game_winner(0);
            match last_winner {
                Some(player) => match player {
                    MonteCarloPlayer::Me => {
                        wins += 1;
                        eprintln!("me winner ({})", wins);
                    }
                    MonteCarloPlayer::Opp => {
                        eprintln!("opp winner");
                        break;
                    }
                },
                None => eprintln!("tie"),
            }
        }
        assert_ne!(last_winner, Some(MonteCarloPlayer::Opp));
        assert!(wins > 40)
    }

    #[test]
    fn test_tree_width_and_depth_me_first() {
        let use_heuristic_score = false;
        let mut last_winner: Option<MonteCarloPlayer> = None;
        let mut wins = 0;
        for i in 0..50 {
            eprintln!("________match {}________", i + 1);
            let mut ttt_match = TicTacToeGameData::new();
            let mut mcts_player: MonteCarloTreeSearch<
                TicTacToeGameData,
                TicTacToePlayerAction,
                TicTacToeGameDataUpdate,
            > = MonteCarloTreeSearch::new(
                MonteCarloGameMode::ByTurns,
                MAX_NUMBER_OF_TURNS,
                FORCE_UPDATE,
                TIME_OUT_FIRST_TURN,
                TIME_OUT_SUCCESSIVE_TURNS,
                WEIGHTING_FACTOR,
                use_heuristic_score,
                DEBUG,
                KEEP_ROOT,
            );
            while !ttt_match.check_game_ending(0) {
                let start = mcts_player.init_root(&ttt_match, MonteCarloPlayer::Me);
                mcts_player.expand_tree(start);
                eprint!("me  ");
                let (current_game_data, _) = mcts_player.choose_and_execute_actions();
                let parent = mcts_player.tree_root.get_parent().unwrap();
                for child in parent.iter_children() {
                    let child_node = child.get_value();
                    let child_action =
                        TicTacToePlayerAction::downcast_self(&child_node.player_action);
                    eprintln!("({}, {}): w: {:.1}, s: {:.0}, ets: {:.2}, ers: {:.2}, hs: {:.2}, beta: {:.0}", child_action.cell.x(), child_action.cell.y(), child_node.wins, child_node.samples, child_node.exploitation_score, child_node.exploration_score, child_node.heuristic_score, child_node.beta);
                }
                ttt_match = *TicTacToeGameData::downcast_self(&current_game_data);
                if !ttt_match.check_game_ending(0) {
                    // let opp act by choosing a random actions
                    match ttt_match.choose_random_next_action() {
                        Some(next_action) => {
                            ttt_match.apply_opp_action(&next_action);
                        }
                        None => (),
                    }
                }
            }
            last_winner = ttt_match.game_winner(0);
            match last_winner {
                Some(player) => match player {
                    MonteCarloPlayer::Me => {
                        wins += 1;
                        eprintln!("me winner ({})", wins);
                    }
                    MonteCarloPlayer::Opp => {
                        eprintln!("opp winner");
                        break;
                    }
                },
                None => eprintln!("tie"),
            }
        }
        assert_ne!(last_winner, Some(MonteCarloPlayer::Opp));
        assert!(wins > 40)
    }
}
