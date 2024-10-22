// main functionality of mcts

use rand::prelude::*;
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::Duration;
use std::time::Instant;

use super::{
    MonteCarloGameData, MonteCarloGameDataUpdate, MonteCarloGameMode, MonteCarloNode,
    MonteCarloNodeType, MonteCarloPlayer, MonteCarloPlayerAction,
};
use crate::my_tree::*;

pub struct MonteCarloTreeSearch<
    G: MonteCarloGameData,
    A: MonteCarloPlayerAction,
    U: MonteCarloGameDataUpdate,
> {
    tree_root: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
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
    use_caching: bool,
    #[allow(clippy::type_complexity)]
    cache: HashMap<(G, MonteCarloPlayer, usize), Weak<TreeNode<MonteCarloNode<G, A, U>>>>,
    debug: bool,
}

impl<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate>
    MonteCarloTreeSearch<G, A, U>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        game_mode: MonteCarloGameMode,
        max_number_of_turns: usize,
        force_update: bool,
        time_out_first_turn: Duration,
        time_out_successive_turns: Duration,
        weighting_factor: f32,
        use_heuristic_score: bool,
        use_caching: bool,
        debug: bool,
    ) -> Self {
        MonteCarloTreeSearch {
            tree_root: TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0),
            game_mode,
            starting_player: MonteCarloPlayer::Me,
            played_turns: 0,
            max_number_of_turns,
            force_update,
            first_turn: true,
            time_out_first_turn,
            time_out_successive_turns,
            weighting_factor, // try starting with 1.0 and find a way to tune to a better value
            use_heuristic_score,
            use_caching,
            cache: HashMap::new(),
            debug,
        }
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
                .iter_level_order_traversal_with_borders(1, end_level)
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
                }
                None => {
                    // create new tree_root, since no node with game_data has been found
                    if self.debug {
                        eprintln!("Current game state not found in tree. Reinitialize tree after {} played turns", self.played_turns);
                    }
                    self.tree_root = TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0);
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
        // clear cache from old nodes
        if self.use_caching {
            self.cache.retain(|_, v| v.weak_count() > 0);
        }
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
        // return game_data and my action
        let result = (child.get_value().game_data, child.get_value().player_action);
        result
    }

    fn one_cycle(&mut self, start: &Instant, time_out: Duration) -> bool {
        let mut start_node = self.tree_root.clone();
        loop {
            match self.selection(start, time_out, start_node) {
                Some(selection_node) => {
                    // if expansion only creates links to cached tree nodes, it returns None
                    match self.expansion(selection_node) {
                        Ok(child_node) => {
                            // if time out, simulation returns None
                            if let Some(simulation_score) =
                                self.simulation(child_node.clone(), start, time_out)
                            {
                                self.propagation(child_node, simulation_score)
                            }
                        }
                        Err(parent_with_cached_child) => {
                            // Err contains a parent node, which was newly linked to at least one cached node
                            // restart selection from this node with newly linked cached node(s)
                            start_node = parent_with_cached_child;
                            continue;
                        }
                    }
                }
                None => return true, // no more nodes to simulate in tree or time over
            }
            break;
        }
        false
    }

    fn selection(
        &self,
        start: &Instant,
        time_out: Duration,
        mut selection_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
    ) -> Option<Rc<TreeNode<MonteCarloNode<G, A, U>>>> {
        let mut rng = thread_rng();
        // search for node to select
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
            if let Some(child_without_samples) = selection_node
                .iter_children()
                .filter(|c| c.get_value().samples.is_nan())
                .choose(&mut rng)
            {
                return Some(child_without_samples);
            }

            // calc exploitation score depending on parent
            selection_node.iter_children().for_each(|c| {
                c.get_mut_value()
                    .calc_node_score(selection_node.get_value().samples, self.weighting_factor)
            });
            // select child with maximum total score
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
                None => panic!("selection should always find a child!"),
            };
        }
        Some(selection_node)
    }

    #[allow(clippy::type_complexity)]
    fn expansion(
        &mut self,
        expansion_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
    ) -> Result<Rc<TreeNode<MonteCarloNode<G, A, U>>>, Rc<TreeNode<MonteCarloNode<G, A, U>>>> {
        if expansion_node.get_value().game_end_node || expansion_node.get_value().samples.is_nan()
        {
            return Ok(expansion_node);
        }
        let mut found_cached_game_state = false;
        let next_node = expansion_node.get_value().next_node;
        match next_node {
            MonteCarloNodeType::GameDataUpdate => {
                for game_data_update in U::iter_game_data_updates(
                    &expansion_node.get_value().game_data,
                    self.force_update,
                ) {
                    let mut new_game_data_update_node = expansion_node
                        .get_value()
                        .new_game_data_update_child(game_data_update);
                    if new_game_data_update_node.apply_game_data_update(
                        &expansion_node.get_value().game_data,
                        !self.force_update,
                    ) {
                        // node is consistent
                        new_game_data_update_node.set_next_node(self.force_update);
                        expansion_node.add_child(new_game_data_update_node, 0);
                    }
                }
            }
            MonteCarloNodeType::ActionResult => {
                for player_action in A::iter_actions(
                    &expansion_node.get_value().game_data,
                    expansion_node.get_value().player,
                    expansion_node.get_value().game_turn,
                ) {
                    let mut new_player_action_node = expansion_node
                        .get_value()
                        .new_player_action_child(player_action);
                    new_player_action_node.apply_action(
                        &expansion_node.get_value().game_data,
                        &expansion_node.get_value().player_action,
                        self.game_mode,
                        self.max_number_of_turns,
                        self.use_heuristic_score,
                    );
                    new_player_action_node.set_next_node(self.force_update);
                    // try cache new child
                    if self.use_caching {
                        let cache_key = (
                            new_player_action_node.game_data,
                            new_player_action_node.player,
                            new_player_action_node.game_turn,
                        );
                        if let Some(cached_child) = self.cache.get(&cache_key) {
                            // game state is already in cache -> link node (it should still exist)
                            if let Some(child) = cached_child.upgrade() {
                                expansion_node.link_child_to_parent(child);
                                found_cached_game_state = true;
                                continue;
                            }
                        }
                        let child = expansion_node.add_child(new_player_action_node, 0);
                        // cache only certain nodes
                        if self.game_mode == MonteCarloGameMode::ByTurns
                            || (self.game_mode == MonteCarloGameMode::SameTurnParallel
                                && new_player_action_node.player == MonteCarloPlayer::Me)
                        {
                            self.cache.insert(cache_key, Rc::downgrade(&child));
                        }
                    } else {
                        expansion_node.add_child(new_player_action_node, 0);
                    }
                }
            }
        }
        if found_cached_game_state {
            return Err(expansion_node);
        }
        Ok(expansion_node.get_child(0).unwrap())
    }

    fn simulation(
        &self,
        simulation_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
        start: &Instant,
        time_out: Duration,
    ) -> Option<f32> {
        if simulation_node.get_value().game_end_node {
            Some(simulation_node.get_value().calc_simulation_score())
        } else {
            let mut rng = thread_rng();
            let mut simulation = *simulation_node.get_value();
            while !simulation.game_end_node {
                if start.elapsed() >= time_out {
                    // return tie, if simulation cannot finish in time
                    return None;
                }
                match simulation.next_node {
                    MonteCarloNodeType::GameDataUpdate => {
                        // create new game game_data update
                        let parent_game_data = simulation.game_data;
                        let game_data_update =
                            U::iter_game_data_updates(&simulation.game_data, self.force_update)
                                .choose(&mut rng)
                                .unwrap();
                        simulation = simulation.new_game_data_update_child(game_data_update);
                        simulation.apply_game_data_update(&parent_game_data, false);
                        simulation.set_next_node(self.force_update);
                    }
                    MonteCarloNodeType::ActionResult => {
                        // set random next action
                        let parent_game_data = simulation.game_data;
                        let parent_action = simulation.player_action;
                        let player_action = A::iter_actions(
                            &simulation.game_data,
                            simulation.player,
                            simulation.game_turn,
                        )
                        .choose(&mut rng)
                        .unwrap();
                        simulation = simulation.new_player_action_child(player_action);
                        simulation.apply_action(
                            &parent_game_data,
                            &parent_action,
                            self.game_mode,
                            self.max_number_of_turns,
                            self.use_heuristic_score,
                        );
                        simulation.set_next_node(self.force_update);
                    }
                }
            }
            Some(simulation.calc_simulation_score())
        }
    }

    fn propagation(
        &self,
        start_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
        mut simulation_score: f32,
    ) {
        // first set number of samples of start_node from NaN to 0.0, if required
        if start_node.get_value().samples.is_nan() {
            start_node.get_mut_value().samples = 0.0;
        }
        // backtrack simulation_score and heuristic if score event
        for nodes in start_node.iter_back_track() {
            for node in nodes.iter() {
                // ToDo: how to weight GameDataUpdate Nodes properly?
                // ToDo: ChatGPT suggest to give a penalty on exploration score, not the exploitation score
                if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate
                    && node.len_children() > 0
                {
                    let num_children = node.len_children() as f32;
                    simulation_score /= num_children;
                }
                // score simulation result and calc new exploitation score
                node.get_mut_value().score_simulation_result(
                    simulation_score,
                    1.0,
                    self.use_heuristic_score,
                );
            }
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
            .score_simulation_result(wins, samples, self.use_heuristic_score);
        for nodes in start_node.iter_back_track().skip(1) {
            for node in nodes.iter() {
                if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate {
                    let num_children = node.len_children() as f32;
                    wins /= num_children;
                    samples /= num_children;
                }
                // remove samples and wins of inconsistent children and calc new exploitation score
                node.get_mut_value().score_simulation_result(
                    wins,
                    samples,
                    self.use_heuristic_score,
                );
            }
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
            // calc inconsistent simulation results
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
    const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(200);
    const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(50);
    const WEIGHTING_FACTOR: f32 = 1.4;
    const USE_CACHING: bool = true;
    const DEBUG: bool = true;

    #[test]
    fn test_tree_width_and_depth_opp_first() {
        let use_heuristic_score = false;
        let mut wins = 0;
        let mut losses = 0;
        eprintln!("player symbols:");
        eprintln!("me: {}", TicTacToeStatus::Player(MonteCarloPlayer::Me));
        eprintln!("opp: {}", TicTacToeStatus::Player(MonteCarloPlayer::Opp));
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
                USE_CACHING,
                DEBUG,
            );
            while !ttt_match.check_game_ending(0) {
                let start = mcts_player.init_root(&ttt_match, MonteCarloPlayer::Opp);
                mcts_player.expand_tree(start);
                let (current_game_data, _) = mcts_player.choose_and_execute_actions();
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
            eprintln!("{}", ttt_match);
            match ttt_match.game_winner(0) {
                Some(player) => match player {
                    MonteCarloPlayer::Me => {
                        wins += 1;
                        eprintln!("me winner ({})", wins);
                    }
                    MonteCarloPlayer::Opp => {
                        losses += 1;
                        eprintln!("opp winner");
                        break;
                    }
                },
                None => eprintln!("tie"),
            }
        }
        assert_eq!(losses, 0);
        assert!(wins > 40)
    }

    #[test]
    fn test_tree_width_and_depth_me_first() {
        let use_heuristic_score = false;
        let mut wins = 0;
        let mut losses = 0;
        eprintln!("player symbols:");
        eprintln!("me: {}", TicTacToeStatus::Player(MonteCarloPlayer::Me));
        eprintln!("opp: {}", TicTacToeStatus::Player(MonteCarloPlayer::Opp));
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
                USE_CACHING,
                DEBUG,
            );
            while !ttt_match.check_game_ending(0) {
                let start = mcts_player.init_root(&ttt_match, MonteCarloPlayer::Me);
                mcts_player.expand_tree(start);
                let (current_game_data, _) = mcts_player.choose_and_execute_actions();
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
            eprintln!("{}", ttt_match);
            match ttt_match.game_winner(0) {
                Some(player) => match player {
                    MonteCarloPlayer::Me => {
                        wins += 1;
                        eprintln!("me winner ({})", wins);
                    }
                    MonteCarloPlayer::Opp => {
                        losses += 1;
                        eprintln!("opp winner");
                        break;
                    }
                },
                None => eprintln!("tie"),
            }
        }
        assert_eq!(losses, 0);
        assert!(wins > 40)
    }
}
