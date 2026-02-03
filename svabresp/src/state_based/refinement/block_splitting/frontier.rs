use super::{BlockSplittingHeuristics, PlayerPartition};
use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::regions::StateRegion;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::{ActionCollection, Distribution};
use rand::Rng;

pub enum FrontierSplittingVariant {
    RandomState,
    MostEdgesToWinningAndLosing,
    MostEdgesToLosing,
    MostEdgesToWinning,
}

pub struct FrontierSplittingHeuristics {
    variant: FrontierSplittingVariant,
}

impl FrontierSplittingHeuristics {
    pub fn random_state() -> Self {
        Self {
            variant: FrontierSplittingVariant::RandomState,
        }
    }

    pub fn most_edges_to_winning_and_losing() -> Self {
        Self {
            variant: FrontierSplittingVariant::MostEdgesToWinningAndLosing,
        }
    }

    pub fn most_edges_to_losing() -> Self {
        Self {
            variant: FrontierSplittingVariant::MostEdgesToLosing,
        }
    }

    pub fn most_edges_to_winning() -> Self {
        Self {
            variant: FrontierSplittingVariant::MostEdgesToWinning,
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

        println!(
            "Splitting block {} with coalition {:b}",
            bsp.block_index, bsp.coalition_bitmap
        );

        let mut overlap_sizes = Vec::new();

        for &player in players {
            let mut overlap_value = OverlapData::new(rand::rng().random_range(0..1_000_000));

            for state in game.get_grouping().get_states(player) {
                if !bsp.winning_region_without.contains(state)
                    && bsp.winning_region_with.contains(state)
                {
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

        for (i, overlap) in overlap_sizes.iter().enumerate() {
            println!(
                "  {}: {} to winning, {} to losing, random {}",
                i, overlap.states_to_winning, overlap.states_to_losing, overlap.random_value
            );
        }

        let zipped = players.iter().zip(overlap_sizes);
        let split_player = zipped
            .max_by(|(_, o1), (_, o2)| {
                match self.variant {
                    FrontierSplittingVariant::RandomState => 1.cmp(&1),
                    FrontierSplittingVariant::MostEdgesToWinningAndLosing => {
                        o1.total_overlap().cmp(&o2.total_overlap())
                    }
                    FrontierSplittingVariant::MostEdgesToLosing => o1
                        .states_to_losing
                        .cmp(&o2.states_to_losing)
                        .then(o1.states_to_winning.cmp(&o2.states_to_winning)),
                    FrontierSplittingVariant::MostEdgesToWinning => o1
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
