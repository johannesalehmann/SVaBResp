mod block_selection;
pub use block_selection::*;

mod block_splitting;
pub use block_splitting::*;

mod initial_partition;
pub use initial_partition::*;

mod grouped_game;
mod partition;

use super::StateBasedResponsibilityGame;
use crate::shapley::{BruteForceAlgorithm, MinimalCoalitionCache, OnePairPerStateCollector};
use crate::state_based::grouping::StateGroups;
use log::trace;
pub use partition::{PlayerPartition, PlayerPartitionEntry};
use probabilistic_model_algorithms::regions::StateRegion;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;

pub trait GroupBlockingProvider {
    fn compute_blocks<G: StateGroups, A: SolvableGame>(
        self,
        game: &mut StateBasedResponsibilityGame<G, A>,
    ) -> PlayerPartition;
}

pub struct IdentityGroupBlockingProvider {}

impl IdentityGroupBlockingProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl GroupBlockingProvider for IdentityGroupBlockingProvider {
    fn compute_blocks<G: StateGroups, A: SolvableGame>(
        self,
        game: &mut StateBasedResponsibilityGame<G, A>,
    ) -> PlayerPartition {
        let mut partition = PlayerPartition::new();
        for player in 0..game.get_grouping().get_count() {
            partition.add_entry(PlayerPartitionEntry::with_players(vec![player]));
        }
        partition
    }
}

pub struct RefinementGroupBlockingProvider<
    InitialPartition: InitialPartitionProvider,
    SelectionHeuristics: BlockSelectionHeuristics,
    SplittingHeuristics: BlockSplittingHeuristics,
> {
    initial_partition: InitialPartition,
    selection_heuristics: SelectionHeuristics,
    splitting_heuristics: SplittingHeuristics,
}

impl<
    InitialPartition: InitialPartitionProvider,
    SelectionHeuristics: BlockSelectionHeuristics,
    SplittingHeuristics: BlockSplittingHeuristics,
> RefinementGroupBlockingProvider<InitialPartition, SelectionHeuristics, SplittingHeuristics>
{
    pub fn new(
        initial_partition: InitialPartition,
        selection_heuristics: SelectionHeuristics,
        splitting_heuristics: SplittingHeuristics,
    ) -> Self {
        Self {
            initial_partition,
            selection_heuristics,
            splitting_heuristics,
        }
    }
}

impl<
    InitialPartition: InitialPartitionProvider,
    SelectionHeuristics: BlockSelectionHeuristics,
    SplittingHeuristics: BlockSplittingHeuristics,
> GroupBlockingProvider
    for RefinementGroupBlockingProvider<InitialPartition, SelectionHeuristics, SplittingHeuristics>
{
    fn compute_blocks<G: StateGroups, A: SolvableGame>(
        self,
        game: &mut StateBasedResponsibilityGame<G, A>,
    ) -> PlayerPartition {
        let mut algorithm = RefinementAlgorithm::new(
            game,
            self.initial_partition,
            self.selection_heuristics,
            self.splitting_heuristics,
        );
        algorithm.run();
        algorithm.current_partition
    }
}

pub struct BlockSwitchingPair<R: StateRegion> {
    block_index: usize,
    #[allow(unused)]
    // It might be useful to have the coalition available in the future, even if it currently is not used
    coalition_bitmap: u64,
    winning_region_without: R,
    winning_region_with: R,
}

impl<R: StateRegion> BlockSwitchingPair<R> {
    fn winning_region_size_delta(&self) -> usize {
        self.winning_region_with.size() - self.winning_region_without.size()
    }
}

pub struct RefinementAlgorithm<
    'a,
    G: StateGroups,
    A: SolvableGame,
    SelectionHeuristics: BlockSelectionHeuristics,
    SplittingHeuristics: BlockSplittingHeuristics,
> {
    game: &'a mut StateBasedResponsibilityGame<G, A>,
    current_partition: PlayerPartition,
    selection_heuristics: SelectionHeuristics,
    splitting_heuristics: SplittingHeuristics,
}

impl<
    'a,
    G: StateGroups,
    A: SolvableGame,
    SelectionHeuristics: BlockSelectionHeuristics,
    SplittingHeuristics: BlockSplittingHeuristics,
> RefinementAlgorithm<'a, G, A, SelectionHeuristics, SplittingHeuristics>
{
    pub fn new<I: InitialPartitionProvider>(
        game: &'a mut StateBasedResponsibilityGame<G, A>,
        initial_coalition_provider: I,
        selection_heuristics: SelectionHeuristics,
        splitting_heuristics: SplittingHeuristics,
    ) -> Self {
        let initial_partition = initial_coalition_provider.get_initial_coalition(&game);

        Self {
            game,
            current_partition: initial_partition,
            selection_heuristics,
            splitting_heuristics,
        }
    }

    pub fn run(&mut self) {
        trace!("Running refinement algorithm");
        while let Some(bsps) = self.compute_refinement_candidates() {
            self.iteration(bsps);
        }
        trace!("Finished refinement");
    }

    pub fn iteration(&mut self, bsps: Vec<BlockSwitchingPair<A::WinningRegionType>>) {
        trace!("Performing refinement iteration");
        trace!("Selecting refinement targets");
        let to_refine =
            self.selection_heuristics
                .select_blocks(&self.game, &self.current_partition, bsps);

        let n = to_refine.len();
        trace!(
            "Splitting {} {}",
            n,
            if n == 1 { "block" } else { "blocks" }
        );
        for (index, block) in to_refine.into_iter().enumerate() {
            trace!("Splitting block {}/{}", index, n);
            self.splitting_heuristics
                .split_block(&self.game, &mut self.current_partition, block);
        }
    }

    pub fn compute_refinement_candidates(
        &mut self,
    ) -> Option<Vec<BlockSwitchingPair<A::WinningRegionType>>> {
        trace!("Computing refinement candidates");
        let mut bsps = Vec::with_capacity(self.current_partition.entries.len());
        for _ in 0..self.current_partition.entries.len() {
            bsps.push(None);
        }

        let game = grouped_game::GroupedGame::new(&mut self.game, &self.current_partition);
        let cached_group_game = MinimalCoalitionCache::create(game);
        // Create another copy of the game so we can compute winning regions for the blocks with BSPs
        let mut game = grouped_game::GroupedGame::new(&mut self.game, &self.current_partition);

        for coalition in cached_group_game.minimal_coalitions {
            for block in 0..self.current_partition.entries.len() {
                if self.current_partition.entries[block].players.len() > 1 {
                    if coalition & 1 << block != 0 {
                        if bsps[block].is_none() {
                            let coalition_without = coalition & !(1 << block);
                            let winning_region_without = game.get_winning_region(coalition_without);
                            let winning_region_with = game.get_winning_region(coalition);

                            bsps[block] = Some(BlockSwitchingPair {
                                block_index: block,
                                coalition_bitmap: coalition_without,
                                winning_region_without,
                                winning_region_with,
                            })
                        }
                    }
                }
            }
        }

        trace!("Found {} refinement candidates", bsps.len());

        let mut res = Vec::new();
        for bsp in bsps {
            if let Some(bsp) = bsp {
                res.push(bsp);
            }
        }

        if res.len() == 0 { None } else { Some(res) }
    }
}
