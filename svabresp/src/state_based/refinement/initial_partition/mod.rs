mod random;
pub use random::RandomInitialPartition;

mod singleton;
pub use singleton::SingletonInitialPartition;

use super::PlayerPartition;
use crate::state_based::{StateBasedResponsibilityNonstochasticGame, grouping::StateGroups};
use crate::state_based::game::Objective;

pub trait InitialPartitionProvider {
    fn get_initial_coalition<G: StateGroups, O: Objective>(
        self,
        game: &StateBasedResponsibilityNonstochasticGame<G, O>,
    ) -> PlayerPartition;
}
