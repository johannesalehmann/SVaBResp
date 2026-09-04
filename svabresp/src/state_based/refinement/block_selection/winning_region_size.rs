use crate::state_based::StateBasedResponsibilityNonstochasticGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::game::Objective;
use crate::state_based::refinement::{
    BlockSelectionHeuristics, BlockSwitchingPair, PlayerPartition,
};

pub struct WinningRegionSizeSelectionHeuristics {
    blocks_per_iteration: usize,
    criterion: WinningRegionSizeCriterion,
}

pub enum WinningRegionSizeCriterion {
    MaximiseDelta,
    MinimiseDelta,
}

impl WinningRegionSizeSelectionHeuristics {
    pub fn maximise_delta(blocks_per_iteration: usize) -> Self {
        Self {
            blocks_per_iteration,
            criterion: WinningRegionSizeCriterion::MaximiseDelta,
        }
    }
    pub fn minimise_delta(blocks_per_iteration: usize) -> Self {
        Self {
            blocks_per_iteration,
            criterion: WinningRegionSizeCriterion::MinimiseDelta,
        }
    }
}

impl BlockSelectionHeuristics for WinningRegionSizeSelectionHeuristics {
    fn select_blocks<G: StateGroups, O: Objective>(
        &mut self,
        game: &StateBasedResponsibilityNonstochasticGame<G, O>,
        partition: &PlayerPartition,
        mut refinement_candidates: Vec<BlockSwitchingPair>,
    ) -> Vec<BlockSwitchingPair> {
        let _ = (game, partition);

        refinement_candidates.sort_by(|r1, r2| {
            r1.winning_region_size_delta()
                .cmp(&r2.winning_region_size_delta())
        });

        let count = self.blocks_per_iteration.min(refinement_candidates.len());
        match self.criterion {
            WinningRegionSizeCriterion::MaximiseDelta => {
                let mut res = Vec::with_capacity(count);
                for i in (refinement_candidates.len() - count..refinement_candidates.len()).rev() {
                    res.push(refinement_candidates.remove(i));
                }
                res
            }
            WinningRegionSizeCriterion::MinimiseDelta => {
                for i in (count..refinement_candidates.len()).rev() {
                    refinement_candidates.remove(i);
                }
                refinement_candidates
            }
        }
    }
}
