use crate::TwoPlayer::PlayerTwo;

pub trait Owners {
    fn default_owner() -> Self;
}
pub type SinglePlayer = ();
impl Owners for SinglePlayer {
    fn default_owner() -> Self {
        ()
    }
}

#[derive(PartialEq)]
pub enum TwoPlayer {
    PlayerOne,
    PlayerTwo,
}

impl Owners for TwoPlayer {
    fn default_owner() -> Self {
        TwoPlayer::PlayerOne
    }
}
