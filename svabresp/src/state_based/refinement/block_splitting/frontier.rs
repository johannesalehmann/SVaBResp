use super::{BlockSplittingHeuristics, PlayerPartition};
use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::regions::StateRegion;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::{ActionCollection, Distribution};

pub enum FrontierSplittingVariant {
    AnyState,
    PreferStatesReachingLosing,
    PreferStatesReachingWinning,
}

pub struct FrontierSplittingHeuristics {
    variant: FrontierSplittingVariant,
}

impl FrontierSplittingHeuristics {
    pub fn any_state() -> Self {
        Self {
            variant: FrontierSplittingVariant::AnyState,
        }
    }

    pub fn prefer_states_reaching_losing() -> Self {
        Self {
            variant: FrontierSplittingVariant::PreferStatesReachingLosing,
        }
    }

    pub fn prefer_states_reaching_winning() -> Self {
        Self {
            variant: FrontierSplittingVariant::PreferStatesReachingWinning,
        }
    }
}

impl BlockSplittingHeuristics for FrontierSplittingHeuristics {
    fn split_block<G: StateGroups, A: SolvableGame>(
        &mut self,
        game: &StateBasedResponsibilityGame<G, A>,
        partition: &mut PlayerPartition,
        bsp: super::super::BlockSwitchingPair<A::WinningRegionType>,
    ) {
        let players = &partition.entries[bsp.block_index].players;

        let mut overlap_sizes = Vec::new();

        for &player in players {
            let mut states_to_losing = 0;
            let mut states_to_winning = 0;

            for state in game.get_grouping().get_states(player) {
                if !bsp.winning_region_without.contains(state)
                    && bsp.winning_region_with.contains(state)
                {
                    let game = game.get_solvable().get_game();
                    for action in game.states[state].actions.iter() {
                        for destination in action.successors.iter() {
                            if bsp.winning_region_without.contains(destination.index) {
                                states_to_winning += 1;
                            }
                            if !bsp.winning_region_without.contains(destination.index) {
                                states_to_losing += 1;
                            }
                        }
                    }
                }
            }

            overlap_sizes.push((states_to_winning, states_to_losing));
        }

        let zipped = players.iter().zip(overlap_sizes);
        let split_player = match self.variant {
            FrontierSplittingVariant::AnyState => zipped
                .max_by(|(_, (o1_w, o1_l)), (_, (o2_w, o2_l))| (o1_w + o1_l).cmp(&(o2_w + o2_l)))
                .map(|(p, _)| *p)
                .expect("Could not refine any players"),
            FrontierSplittingVariant::PreferStatesReachingLosing => zipped
                .max_by(|(_, (o1_w, o1_l)), (_, (o2_w, o2_l))| {
                    (o1_w + o1_l * 10000).cmp(&(o2_w + o2_l * 10000))
                })
                .map(|(p, _)| *p)
                .expect("Could not refine any players"),
            FrontierSplittingVariant::PreferStatesReachingWinning => zipped
                .max_by(|(_, (o1_w, o1_l)), (_, (o2_w, o2_l))| {
                    (o1_w * 10000 + o1_l).cmp(&(o2_w * 100000 + o2_l))
                })
                .map(|(p, _)| *p)
                .expect("Could not refine any players"),
        };

        partition.split_entry(bsp.block_index, |p| if p == split_player { 1 } else { 0 });
    }
}
