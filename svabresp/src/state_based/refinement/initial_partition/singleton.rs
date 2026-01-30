use crate::shapley::SimpleCooperativeGame;
use crate::state_based::StateBasedResponsibilityGame;
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::{
    InitialPartitionProvider, PlayerPartition, PlayerPartitionEntry,
};
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;

pub struct SingletonInitialPartition {}

impl SingletonInitialPartition {
    pub fn new() -> Self {
        Self {}
    }
}

impl InitialPartitionProvider for SingletonInitialPartition {
    fn get_initial_coalition<G: StateGroups, A: SolvableGame>(
        self,
        game: &StateBasedResponsibilityGame<G, A>,
    ) -> PlayerPartition {
        let mut players = Vec::with_capacity(game.get_player_count());
        for i in 0..game.get_player_count() {
            players.push(i);
        }
        let entry = PlayerPartitionEntry::with_players(players);
        PlayerPartition {
            entries: vec![entry],
        }
    }
}
