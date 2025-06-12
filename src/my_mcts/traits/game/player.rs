// trait to define player abilities

use std::hash::Hash;

pub trait GamePlayer: PartialEq + Eq + Hash {
    fn next(&self) -> Self;
}
