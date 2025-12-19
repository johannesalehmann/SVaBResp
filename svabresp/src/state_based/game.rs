use crate::shapley::{
    CoalitionSpecifier, MonotoneCooperativeGame, PlayerDescriptions, SimpleCooperativeGame,
};
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::TwoPlayer;

pub struct StateBasedResponsibilityGame<G: StateGroups, A: SolvableGame> {
    solvable: A,
    grouping: G,
    group_info: GroupInfo,
}

impl<'a, G: StateGroups, A: SolvableGame> StateBasedResponsibilityGame<G, A> {
    pub fn new(solvable: A, grouping: G) -> Self {
        let group_info = GroupInfo::from_grouping(&grouping);
        Self {
            solvable,
            grouping,
            group_info,
        }
    }
}

impl<G: StateGroups, A: SolvableGame> SimpleCooperativeGame for StateBasedResponsibilityGame<G, A> {
    type PlayerDescriptions = GroupInfo;

    fn get_player_count(&self) -> usize {
        self.grouping.get_count()
    }

    fn player_descriptions(&self) -> &Self::PlayerDescriptions {
        &self.group_info
    }

    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions {
        &mut self.group_info
    }

    fn into_player_descriptions(self) -> Self::PlayerDescriptions {
        self.group_info
    }

    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool {
        for i in 0..self.grouping.get_count() {
            if coalition.is_in_coalition(i) {
                for state in self.grouping.get_states(i) {
                    self.solvable.set_owner(state, TwoPlayer::PlayerOne);
                }
            } else {
                for state in self.grouping.get_states(i) {
                    self.solvable.set_owner(state, TwoPlayer::PlayerTwo);
                }
            }
        }

        self.solvable.get_winner() == TwoPlayer::PlayerOne
    }
}

impl<G: StateGroups, A: SolvableGame> MonotoneCooperativeGame
    for StateBasedResponsibilityGame<G, A>
{
}

pub struct GroupInfo {
    names: Vec<String>,
}

impl GroupInfo {
    pub fn from_grouping<G: StateGroups>(groups: &G) -> Self {
        let mut names = Vec::with_capacity(groups.get_count());
        for g in 0..groups.get_count() {
            names.push(groups.get_label(g))
        }
        Self { names }
    }
}

impl PlayerDescriptions for GroupInfo {
    type IntoIter = std::vec::IntoIter<String>;
    type PlayerType = String;

    fn get_player_description(&self, index: usize) -> &Self::PlayerType {
        &self.names[index]
    }

    fn into_iterator(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.names)
    }
}
