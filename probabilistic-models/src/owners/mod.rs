pub trait Owners {
    fn default_owner() -> Self;
    fn max_player_count() -> usize;
}
pub type SinglePlayer = ();
impl Owners for SinglePlayer {
    fn default_owner() -> Self {
        ()
    }
    fn max_player_count() -> usize {
        1
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TwoPlayer {
    PlayerOne,
    PlayerTwo,
}

impl Owners for TwoPlayer {
    fn default_owner() -> Self {
        TwoPlayer::PlayerOne
    }
    fn max_player_count() -> usize {
        2
    }
}
