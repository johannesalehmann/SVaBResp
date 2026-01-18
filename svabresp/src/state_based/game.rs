use crate::shapley::{
    CoalitionSpecifier, MonotoneCooperativeGame, PlayerDescriptions, SimpleCooperativeGame,
};
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::SolvableGame;
use probabilistic_models::TwoPlayer;

pub struct StateBasedResponsibilityGame<G: StateGroups, A: SolvableGame> {
    solvable: A,
    grouping: G,
    always_helping: Vec<usize>,
    always_adversarial: Vec<usize>,
    group_info: GroupInfo,
}

impl<'a, G: StateGroups, A: SolvableGame> StateBasedResponsibilityGame<G, A> {
    pub fn new(
        solvable: A,
        grouping: G,
        always_helping: Vec<usize>,
        always_adversarial: Vec<usize>,
    ) -> Self {
        let group_info = GroupInfo::from_grouping(&grouping);
        Self {
            solvable,
            grouping,
            always_helping,
            always_adversarial,
            group_info,
        }
    }

    pub fn map_grouping<G2: StateGroups, F: Fn(G) -> G2>(
        self,
        map: F,
    ) -> StateBasedResponsibilityGame<G2, A> {
        let grouping = map(self.grouping);
        let group_info = GroupInfo::from_grouping(&grouping);

        StateBasedResponsibilityGame {
            solvable: self.solvable,
            grouping,
            always_helping: self.always_helping,
            always_adversarial: self.always_adversarial,
            group_info,
        }
    }

    pub fn get_grouping(&self) -> &G {
        &self.grouping
    }

    pub fn get_solvable(&self) -> &A {
        &self.solvable
    }

    pub fn set_state_owners<C: CoalitionSpecifier>(&mut self, coalition: C) {
        self.set_auxiliary_state_owners();
        for i in 0..self.grouping.get_count() {
            if coalition.is_in_coalition(i) {
                self.set_group_owners(i, TwoPlayer::PlayerOne);
            } else {
                self.set_group_owners(i, TwoPlayer::PlayerTwo);
            }
        }
    }

    pub fn set_auxiliary_state_owners(&mut self) {
        for &state in &self.always_helping {
            self.solvable.set_owner(state, TwoPlayer::PlayerOne);
        }
        for &state in &self.always_adversarial {
            self.solvable.set_owner(state, TwoPlayer::PlayerTwo);
        }
    }

    pub fn set_group_owners(&mut self, group_index: usize, owner: TwoPlayer) {
        for state in self.grouping.get_states(group_index) {
            self.solvable.set_owner(state, owner);
        }
    }

    // TODO: It is a bit of a hack that these functions exist, as they break the abstraction. It
    // would be nicer if this were handled by passing a suitable CoalitionSpecifier to the main
    // function instead of setting the owners explicitly.
    pub fn is_winning_with_current_owners(&mut self) -> bool {
        self.solvable.get_winner() == TwoPlayer::PlayerOne
    }

    pub fn get_winning_region<C: CoalitionSpecifier>(
        &mut self,
        coalition: C,
    ) -> A::WinningRegionType {
        self.set_state_owners(coalition);
        self.solvable.get_winning_region()
    }

    pub fn get_winning_region_with_current_owners(&mut self) -> A::WinningRegionType {
        self.solvable.get_winning_region()
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
        self.set_state_owners(coalition);
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
