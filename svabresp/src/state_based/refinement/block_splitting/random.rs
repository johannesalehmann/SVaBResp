use super::BlockSplittingHeuristics;
use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{BlockSwitchingPair, PlayerPartition};
use probabilistic_model_algorithms::regions::StateRegion;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use rand::Rng;

pub struct RandomSplittingHeuristics {}

impl RandomSplittingHeuristics {
    pub fn new() -> Self {
        Self {}
    }
}

impl BlockSplittingHeuristics for RandomSplittingHeuristics {
    fn split_block<G: StateGroups, A: SolvableGame>(
        &mut self,
        game: &StateBasedResponsibilityGame<G, A>,
        partition: &mut PlayerPartition,
        bsp: BlockSwitchingPair<A::WinningRegionType>,
    ) {
        let mut overlapping_players = Vec::new();
        let players = &partition.entries[bsp.block_index].players;
        for &player in players {
            for state in game.get_grouping().get_states(player) {
                if !bsp.winning_region_without.contains(state)
                    && bsp.winning_region_with.contains(state)
                {
                    overlapping_players.push(player);
                }
            }
        }

        if overlapping_players.len() == 0 {
            panic!("None of the players overlap with the winning region.")
        }

        let split_player = rand::rng().random_range(0..overlapping_players.len());
        partition.split_entry(bsp.block_index, |p| if p == split_player { 1 } else { 0 });
    }
}
