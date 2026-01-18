use super::PlayerPartition;
use crate::shapley::{CoalitionSpecifier, PlayerDescriptions, SimpleCooperativeGame};
use crate::state_based::{StateBasedResponsibilityGame, grouping::StateGroups};
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::TwoPlayer;

pub struct GroupedGame<'a, G: StateGroups, A: SolvableGame> {
    game: &'a mut StateBasedResponsibilityGame<G, A>,
    partition: &'a PlayerPartition,
    player_description: GroupedGamePlayerDescriptions,
}

impl<'a, G: StateGroups, A: SolvableGame> GroupedGame<'a, G, A> {
    pub fn new(
        game: &'a mut StateBasedResponsibilityGame<G, A>,
        partition: &'a PlayerPartition,
    ) -> Self {
        let players = GroupedGamePlayerDescriptions::new(partition.entries.len());
        Self {
            game,
            partition,
            player_description: players,
        }
    }

    fn set_owners<C: CoalitionSpecifier>(&mut self, coalition: C) {
        self.game.set_auxiliary_state_owners();
        for (player_index, player) in self.partition.entries.iter().enumerate() {
            if coalition.is_in_coalition(player_index) {
                for &entry in &player.players {
                    self.game.set_group_owners(entry, TwoPlayer::PlayerOne);
                }
            } else {
                for &entry in &player.players {
                    self.game.set_group_owners(entry, TwoPlayer::PlayerTwo);
                }
            }
        }
    }

    pub fn get_winning_region<C: CoalitionSpecifier>(
        &mut self,
        coalition: C,
    ) -> A::WinningRegionType {
        self.set_owners(coalition);

        self.game.get_winning_region_with_current_owners()
    }
}

impl<'a, G: StateGroups, A: SolvableGame> SimpleCooperativeGame for GroupedGame<'a, G, A> {
    type PlayerDescriptions = GroupedGamePlayerDescriptions;

    fn get_player_count(&self) -> usize {
        self.partition.entries.len()
    }

    fn player_descriptions(&self) -> &Self::PlayerDescriptions {
        &self.player_description
    }

    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions {
        &mut self.player_description
    }

    fn into_player_descriptions(self) -> Self::PlayerDescriptions {
        self.player_description
    }

    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool {
        self.set_owners(coalition);

        self.game.is_winning_with_current_owners()
    }
}

pub struct GroupedGamePlayerDescriptions {
    players: Vec<usize>,
}

impl GroupedGamePlayerDescriptions {
    pub fn new(player_count: usize) -> Self {
        Self {
            players: (0..player_count).collect(),
        }
    }
}

impl PlayerDescriptions for GroupedGamePlayerDescriptions {
    type IntoIter = std::vec::IntoIter<usize>;
    type PlayerType = usize;

    fn get_player_description(&self, index: usize) -> &Self::PlayerType {
        &self.players[index]
    }

    fn into_iterator(self) -> Self::IntoIter {
        self.players.into_iter()
    }
}
