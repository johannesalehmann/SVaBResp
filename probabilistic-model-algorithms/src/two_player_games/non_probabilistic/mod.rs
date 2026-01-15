mod algorithm_collections;
pub use algorithm_collections::*;

pub mod attractor;

mod solvable_game;
pub use solvable_game::*;

pub trait ChangeableOwners {
    fn set_owner(&mut self, state: usize, owner: probabilistic_models::TwoPlayer);
}

pub struct ReachabilityAlgorithmCollectionWithCachedTarget {
    pub target_states: Vec<usize>,
    pub buffer: attractor::AttractorBuffer,
}
