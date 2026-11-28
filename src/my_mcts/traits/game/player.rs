// trait to define player abilities

use std::hash::Hash;

pub trait GamePlayer: PartialEq + Eq + Hash + Clone + Copy + Sync + Send {}
