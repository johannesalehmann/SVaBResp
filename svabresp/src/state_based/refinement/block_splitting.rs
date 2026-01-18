use super::PlayerPartition;
use crate::state_based::{StateBasedResponsibilityGame, grouping::StateGroups};
use probabilistic_model_algorithms::regions::StateRegion;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::{ActionCollection, Distribution};

pub trait BlockSplittingHeuristics {
    fn split_block<G: StateGroups, A: SolvableGame>(
        &mut self,
        game: &StateBasedResponsibilityGame<G, A>,
        partition: &mut PlayerPartition,
        bsp: super::BlockSwitchingPair<A::WinningRegionType>,
    );
}

pub struct FrontierSplittingHeuristics {}

impl FrontierSplittingHeuristics {
    pub fn new() -> Self {
        Self {}
    }
}

impl BlockSplittingHeuristics for FrontierSplittingHeuristics {
    fn split_block<G: StateGroups, A: SolvableGame>(
        &mut self,
        game: &StateBasedResponsibilityGame<G, A>,
        partition: &mut PlayerPartition,
        bsp: super::BlockSwitchingPair<A::WinningRegionType>,
    ) {
        let players = &partition.entries[bsp.block_index].players;

        let mut overlap_sizes = Vec::new();

        for &player in players {
            let mut overlap_size = 0;

            for state in game.get_grouping().get_states(player) {
                if !bsp.winning_region_without.contains(state)
                    && bsp.winning_region_with.contains(state)
                {
                    let game = game.get_solvable().get_game();
                    for action in game.states[state].actions.iter() {
                        for destination in action.successors.iter() {
                            if bsp.winning_region_without.contains(destination.index) {
                                overlap_size += 1;
                            }
                        }
                    }
                }
            }

            overlap_sizes.push(overlap_size);
        }

        let split_player = players
            .iter()
            .zip(overlap_sizes)
            .max_by(|(_, o1), (_, o2)| o1.cmp(o2))
            .map(|(p, _)| *p)
            .expect("Could not refine any players");

        partition.split_entry(bsp.block_index, |p| if p == split_player { 1 } else { 0 });
    }
}
