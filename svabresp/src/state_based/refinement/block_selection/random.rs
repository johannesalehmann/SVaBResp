use crate::state_based::StateBasedResponsibilityNonstochasticGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{
    BlockSelectionHeuristics, BlockSwitchingPair, PlayerPartition,
};
use rand::RngExt;
use crate::state_based::game::Objective;

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
    fn select_blocks<G: StateGroups, O: Objective>(
        &mut self,
        game: &StateBasedResponsibilityNonstochasticGame<G, O>,
        partition: &PlayerPartition,
        mut refinement_candidates: Vec<BlockSwitchingPair>,
    ) -> Vec<BlockSwitchingPair> {
        let _ = (game, partition);

        let mut res = Vec::new();

        while refinement_candidates.len() > 0 && res.len() < self.blocks_per_iteration {
            let sample = rand::rng().random_range(0..refinement_candidates.len());
            res.push(refinement_candidates.swap_remove(sample));
        }

        res
    }
}
