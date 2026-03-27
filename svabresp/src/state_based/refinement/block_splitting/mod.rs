mod frontier;
pub use frontier::{FrontierSplittingHeuristics, FrontierSplittingVariant};

mod random;
pub use random::RandomSplittingHeuristics;

use super::PlayerPartition;
use crate::state_based::{StateBasedResponsibilityNonstochasticGame, grouping::StateGroups};
use probabilistic_model_algorithms::deterministic_games::SolvableNonstochasticGame;

pub trait BlockSplittingHeuristics {
    fn split_block<G: StateGroups, A: SolvableNonstochasticGame>(
        &mut self,
        game: &StateBasedResponsibilityNonstochasticGame<G, A>,
        partition: &mut PlayerPartition,
        bsp: super::BlockSwitchingPair<A::WinningRegionType>,
    );
}
