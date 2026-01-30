mod random;
pub use random::RandomInitialPartition;

mod singleton;
pub use singleton::SingletonInitialPartition;

use super::PlayerPartition;
use crate::state_based::{StateBasedResponsibilityGame, grouping::StateGroups};
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;

pub trait InitialPartitionProvider {
    fn get_initial_coalition<G: StateGroups, A: SolvableGame>(
        self,
        game: &StateBasedResponsibilityGame<G, A>,
    ) -> PlayerPartition;
}
