pub trait Owners {
    fn default_owner() -> Self;
}
pub type SinglePlayer = ();
impl Owners for SinglePlayer {
    fn default_owner() -> Self {
        ()
    }
}
