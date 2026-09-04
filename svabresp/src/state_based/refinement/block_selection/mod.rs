mod frontier_size;
pub use frontier_size::FrontierSizeSelectionHeuristics;

mod random;
pub use random::RandomBlockSelectionHeuristics;

mod winning_region_size;
pub use winning_region_size::{WinningRegionSizeCriterion, WinningRegionSizeSelectionHeuristics};

use super::{BlockSwitchingPair, PlayerPartition};
use crate::state_based::{StateBasedResponsibilityNonstochasticGame, grouping::StateGroups};
use crate::state_based::game::Objective;

pub trait BlockSelectionHeuristics {
    fn select_blocks<G: StateGroups, O: Objective>(
        &mut self,
        game: &StateBasedResponsibilityNonstochasticGame<G, O>,
        partition: &PlayerPartition,
        block_switching_pairs: Vec<BlockSwitchingPair>,
    ) -> Vec<BlockSwitchingPair>;
}
