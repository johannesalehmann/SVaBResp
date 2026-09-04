mod frontier;
pub use frontier::{FrontierSplittingHeuristics, FrontierSplittingVariant};

mod random;
pub use random::RandomSplittingHeuristics;

use super::PlayerPartition;
use crate::state_based::{StateBasedResponsibilityNonstochasticGame, grouping::StateGroups};
use crate::state_based::game::Objective;

pub trait BlockSplittingHeuristics {
    fn split_block<G: StateGroups, O: Objective>(
        &mut self,
        game: &StateBasedResponsibilityNonstochasticGame<G, O>,
        partition: &mut PlayerPartition,
        bsp: super::BlockSwitchingPair,
    );
}
