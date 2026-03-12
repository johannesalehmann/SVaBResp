mod frontier;
pub use frontier::{FrontierSplittingHeuristics, FrontierSplittingVariant};

mod random;
pub use random::RandomSplittingHeuristics;

use super::PlayerPartition;
use crate::state_based::{StateBasedResponsibilityGame, grouping::StateGroups};
use probabilistic_model_algorithms::deterministic_games::SolvableGame;

pub trait BlockSplittingHeuristics {
    fn split_block<G: StateGroups, A: SolvableGame>(
        &mut self,
        game: &StateBasedResponsibilityGame<G, A>,
        partition: &mut PlayerPartition,
        bsp: super::BlockSwitchingPair<A::WinningRegionType>,
    );
}
