// trait to define player abilities

pub trait GamePlayer: PartialEq {
    fn next(&self) -> Self;
}
