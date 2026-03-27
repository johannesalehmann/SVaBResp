mod frontier_size;
pub use frontier_size::FrontierSizeSelectionHeuristics;

mod random;
pub use random::RandomBlockSelectionHeuristics;

mod winning_region_size;
pub use winning_region_size::{WinningRegionSizeCriterion, WinningRegionSizeSelectionHeuristics};

use super::{BlockSwitchingPair, PlayerPartition};
use crate::state_based::{StateBasedResponsibilityNonstochasticGame, grouping::StateGroups};
use probabilistic_model_algorithms::deterministic_games::SolvableNonstochasticGame;

pub trait BlockSelectionHeuristics {
    fn select_blocks<G: StateGroups, A: SolvableNonstochasticGame>(
        &mut self,
        game: &StateBasedResponsibilityNonstochasticGame<G, A>,
        partition: &PlayerPartition,
        block_switching_pairs: Vec<BlockSwitchingPair<A::WinningRegionType>>,
    ) -> Vec<BlockSwitchingPair<A::WinningRegionType>>;
}
