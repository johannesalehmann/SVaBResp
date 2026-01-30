use crate::shapley::SimpleCooperativeGame;
use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{
    InitialPartitionProvider, PlayerPartition, PlayerPartitionEntry,
};
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use rand::Rng;

pub struct RandomInitialPartition {
    block_count: usize,
}

impl RandomInitialPartition {
    pub fn new(block_count: usize) -> Self {
        Self { block_count }
    }
}

impl InitialPartitionProvider for RandomInitialPartition {
    fn get_initial_coalition<G: StateGroups, A: SolvableGame>(
        self,
        game: &StateBasedResponsibilityGame<G, A>,
    ) -> PlayerPartition {
        let mut blocks = PlayerPartition::new();
        for _ in 0..self.block_count {
            blocks.add_entry(PlayerPartitionEntry::new());
        }
        for player in 0..game.get_player_count() {
            let block = rand::rng().random_range(0..self.block_count);
            blocks.entries[block].players.push(player);
        }

        blocks
    }
}
