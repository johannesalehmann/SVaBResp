use crate::state_based::StateBasedResponsibilityNonstochasticGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{
    BlockSelectionHeuristics, BlockSwitchingPair, PlayerPartition,
};
use probabilistic_models::traits::ReadStateSpace;
use crate::state_based::game::Objective;

pub struct FrontierSizeSelectionHeuristics {
    blocks_per_iteration: usize,
}

impl FrontierSizeSelectionHeuristics {
    pub fn new(blocks_per_iteration: usize) -> Self {
        Self {
            blocks_per_iteration,
        }
    }
}

impl BlockSelectionHeuristics for FrontierSizeSelectionHeuristics {
    fn select_blocks<G: StateGroups, O: Objective>(
        &mut self,
        game: &StateBasedResponsibilityNonstochasticGame<G, O>,
        partition: &PlayerPartition,
        refinement_candidates: Vec<BlockSwitchingPair>,
    ) -> Vec<BlockSwitchingPair> {
        let _ = (game, partition);

        let mut res = Vec::new();

        let game = game.get_game();
        for refinement_candidate in refinement_candidates {
            let mut frontier_size = 0;
            for state in game.states() {
                if refinement_candidate.winning_region_with.contains(state)
                    && !refinement_candidate.winning_region_without.contains(state)
                {
                    for destination in game.successors_of_state(state) {
                        if refinement_candidate.winning_region_with.contains(destination)
                            || !refinement_candidate
                                .winning_region_without
                                .contains(destination)
                        {
                            frontier_size += 1;
                        }
                    }
                }
            }
            res.push((refinement_candidate, frontier_size));
        }

        res.sort_by(|(_, frontier_size_1), (_, frontier_size_2)| {
            frontier_size_1.cmp(frontier_size_2)
        });

        while res.len() > self.blocks_per_iteration {
            res.remove(res.len() - 1);
        }

        res.into_iter().map(|(r, _)| r).collect()
    }
}
