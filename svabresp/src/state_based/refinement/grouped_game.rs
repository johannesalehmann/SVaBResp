use super::PlayerPartition;
use crate::shapley::{
    CoalitionSpecifier, MonotoneCooperativeGame, PlayerDescriptions, SimpleCooperativeGame,
};
use crate::state_based::{StateBasedResponsibilityNonstochasticGame, grouping::StateGroups};
use probabilistic_models::owners::TwoPlayer;
use crate::state_based::game::{Objective, WinningRegion};

pub struct GroupedGame<'a, G: StateGroups, O: Objective> {
    game: &'a mut StateBasedResponsibilityNonstochasticGame<G, O>,
    partition: &'a PlayerPartition,
    player_description: GroupedGamePlayerDescriptions,
}

impl<'a, G: StateGroups, O: Objective> GroupedGame<'a, G, O> {
    pub fn new(
        game: &'a mut StateBasedResponsibilityNonstochasticGame<G, O>,
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
                    self.game.set_group_owners(entry, TwoPlayer::Eve);
                }
            } else {
                for &entry in &player.players {
                    self.game.set_group_owners(entry, TwoPlayer::Adam);
                }
            }
        }
    }

    pub fn get_winning_region<C: CoalitionSpecifier>(
        &mut self,
        coalition: C,
    ) -> WinningRegion {
        self.set_owners(coalition);

        self.game.get_winning_region_with_current_owners()
    }
}

impl<'a, G: StateGroups, O: Objective> SimpleCooperativeGame
    for GroupedGame<'a, G, O>
{
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

    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool {
        self.set_owners(coalition);

        let result = self.game.is_winning_with_current_owners();
        result
    }
}

impl<'a, G: StateGroups, O: Objective> MonotoneCooperativeGame
    for GroupedGame<'a, G, O>
{
}

#[derive(Clone)]
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
