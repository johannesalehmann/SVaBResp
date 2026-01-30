use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{
    BlockSelectionHeuristics, BlockSwitchingPair, PlayerPartition,
};
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use rand::Rng;

pub struct RandomBlockSelectionHeuristics {
    blocks_per_iteration: usize,
}

impl RandomBlockSelectionHeuristics {
    pub fn new(blocks_per_iteration: usize) -> Self {
        Self {
            blocks_per_iteration,
        }
    }
}

impl BlockSelectionHeuristics for RandomBlockSelectionHeuristics {
    fn select_blocks<G: StateGroups, A: SolvableGame>(
        &mut self,
        game: &StateBasedResponsibilityGame<G, A>,
        partition: &PlayerPartition,
        mut refinement_candidates: Vec<BlockSwitchingPair<A::WinningRegionType>>,
    ) -> Vec<BlockSwitchingPair<A::WinningRegionType>> {
        let _ = (game, partition);

        let mut res = Vec::new();

        while refinement_candidates.len() > 0 && res.len() < self.blocks_per_iteration {
            let sample = rand::rng().random_range(0..refinement_candidates.len());
            res.push(refinement_candidates.swap_remove(sample));
        }

        res
    }
}
