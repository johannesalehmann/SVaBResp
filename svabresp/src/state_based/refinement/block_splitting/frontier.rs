use super::{BlockSplittingHeuristics, PlayerPartition};
use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::regions::StateRegion;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::{ActionCollection, Distribution, Valuation};
use rand::Rng;

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

struct OverlapData {
    states_to_winning: usize,
    states_to_losing: usize,
    random_value: usize,
}

impl OverlapData {
    fn new(random_value: usize) -> Self {
        Self {
            states_to_winning: 0,
            states_to_losing: 0,
            random_value,
        }
    }
    fn total_overlap(&self) -> usize {
        self.states_to_losing + self.states_to_winning
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
            let mut overlap_value = OverlapData::new(rand::rng().random_range(0..1_000_000));

            for state in game.get_grouping().get_states(player) {
                if !bsp.winning_region_without.contains(state)
                    && bsp.winning_region_with.contains(state)
                {
                    println!(
                        "    {} is in the winning region delta",
                        game.get_solvable().get_game().states[state]
                            .valuation
                            .displayable(&game.get_solvable().get_game().valuation_context)
                    );
                    let game = game.get_solvable().get_game();
                    for action in game.states[state].actions.iter() {
                        for destination in action.successors.iter() {
                            if bsp.winning_region_without.contains(destination.index) {
                                overlap_value.states_to_winning += 1;
                            }
                            if !bsp.winning_region_with.contains(destination.index) {
                                overlap_value.states_to_losing += 1;
                            }
                        }
                    }
                }
            }

            overlap_sizes.push(overlap_value);
        }

        let zipped = players.iter().zip(overlap_sizes);
        let split_player = zipped
            .max_by(|(_, o1), (_, o2)| {
                match self.variant {
                    FrontierSplittingVariant::AnyState => {
                        o1.total_overlap().cmp(&o2.total_overlap())
                    }
                    FrontierSplittingVariant::PreferStatesReachingLosing => o1
                        .states_to_losing
                        .cmp(&o2.states_to_losing)
                        .then(o1.states_to_winning.cmp(&o2.states_to_winning)),
                    FrontierSplittingVariant::PreferStatesReachingWinning => o1
                        .states_to_winning
                        .cmp(&o2.states_to_winning)
                        .then(o1.states_to_losing.cmp(&o2.states_to_losing)),
                }
                .then(o1.random_value.cmp(&o2.random_value))
            })
            .map(|(p, _)| *p)
            .expect("Could not refine any players");

        partition.split_entry(bsp.block_index, |p| if p == split_player { 1 } else { 0 });
    }
}
