use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{
    BlockSelectionHeuristics, BlockSwitchingPair, PlayerPartition,
};
use probabilistic_model_algorithms::regions::StateRegion;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::{ActionCollection, Distribution};

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
    fn select_blocks<G: StateGroups, A: SolvableGame>(
        &mut self,
        game: &StateBasedResponsibilityGame<G, A>,
        partition: &PlayerPartition,
        mut refinement_candidates: Vec<BlockSwitchingPair<A::WinningRegionType>>,
    ) -> Vec<BlockSwitchingPair<A::WinningRegionType>> {
        let _ = (game, partition);

        let mut res = Vec::new();

        let game = game.get_solvable().get_game();
        for refinement_candidate in refinement_candidates {
            let mut frontier_size = 0;
            for state in 0..game.states.len() {
                if refinement_candidate.winning_region_with.contains(state)
                    && !refinement_candidate.winning_region_without.contains(state)
                {
                    for action in game.states[state].actions.iter() {
                        for transition in action.successors.iter() {
                            if refinement_candidate
                                .winning_region_with
                                .contains(transition.index)
                                || !refinement_candidate
                                    .winning_region_without
                                    .contains(transition.index)
                            {
                                frontier_size += 1;
                            }
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
