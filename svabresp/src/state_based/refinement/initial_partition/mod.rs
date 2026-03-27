mod random;
pub use random::RandomInitialPartition;

mod singleton;
pub use singleton::SingletonInitialPartition;

use super::PlayerPartition;
use crate::state_based::{StateBasedResponsibilityNonstochasticGame, grouping::StateGroups};
use probabilistic_model_algorithms::deterministic_games::SolvableNonstochasticGame;

pub trait InitialPartitionProvider {
    fn get_initial_coalition<G: StateGroups, A: SolvableNonstochasticGame>(
        self,
        game: &StateBasedResponsibilityNonstochasticGame<G, A>,
    ) -> PlayerPartition;
}
