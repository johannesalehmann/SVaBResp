use crate::shapley::SimpleCooperativeGame;
use crate::state_based::StateBasedResponsibilityNonstochasticGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{
    InitialPartitionProvider, PlayerPartition, PlayerPartitionEntry,
};
use rand::RngExt;
use crate::state_based::game::Objective;

pub struct RandomInitialPartition {
    block_count: usize,
}

impl RandomInitialPartition {
    pub fn new(block_count: usize) -> Self {
        Self { block_count }
    }
}

impl InitialPartitionProvider for RandomInitialPartition {
    fn get_initial_coalition<G: StateGroups, O: Objective>(
        self,
        game: &StateBasedResponsibilityNonstochasticGame<G, O>,
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
