mod algorithm_collections;
pub use algorithm_collections::*;

mod attractor;

mod solvable_game;
pub use solvable_game::*;

pub trait ChangeableOwners {
    fn set_owner(&mut self, state: usize, owner: probabilistic_models::TwoPlayer);
}

pub struct ReachabilityAlgorithmCollectionWithCachedTarget {
    target_states: Vec<usize>,
    buffer: attractor::AttractorBuffer,
}
