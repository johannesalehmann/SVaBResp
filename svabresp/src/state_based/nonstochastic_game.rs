use crate::shapley::{CoalitionSpecifier, MonotoneCooperativeGame, SimpleCooperativeGame};
use crate::state_based::game::{Game, Objective, SolvableGame, StateIdx, WinningRegion};
use crate::state_based::grouping::StateGroups;
use probabilistic_models::owners::TwoPlayer;

pub struct StateBasedResponsibilityNonstochasticGame<G: StateGroups, O: Objective> {
    solvable: SolvableGame<O>,
    pub grouping: G,
    always_helping: Vec<StateIdx>,
    always_adversarial: Vec<StateIdx>,
    group_names: super::GroupNames,
}

impl<G: StateGroups, O: Objective> StateBasedResponsibilityNonstochasticGame<G, O> {
    pub fn new(
        solvable: SolvableGame<O>,
        grouping: G,
        always_helping: Vec<StateIdx>,
        always_adversarial: Vec<StateIdx>,
    ) -> Self {
        let group_names = super::GroupNames::from_grouping(&grouping);
        Self {
            solvable,
            grouping,
            always_helping,
            always_adversarial,
            group_names,
        }
    }

    pub fn map_grouping<G2: StateGroups, F: Fn(G) -> G2>(
        self,
        map: F,
    ) -> StateBasedResponsibilityNonstochasticGame<G2, O> {
        let grouping = map(self.grouping);
        let group_names = super::GroupNames::from_grouping(&grouping);

        StateBasedResponsibilityNonstochasticGame {
            solvable: self.solvable,
            grouping,
            always_helping: self.always_helping,
            always_adversarial: self.always_adversarial,
            group_names,
        }
    }

    pub fn get_grouping(&self) -> &G {
        &self.grouping
    }

    pub fn get_game(&self) -> &Game {
        self.solvable.game()
    }

    pub fn set_state_owners<C: CoalitionSpecifier>(&mut self, coalition: C) {
        self.set_auxiliary_state_owners();
        for i in 0..self.grouping.get_count() {
            if coalition.is_in_coalition(i) {
                self.set_group_owners(i, TwoPlayer::Eve);
            } else {
                self.set_group_owners(i, TwoPlayer::Adam);
            }
        }
    }

    pub fn set_auxiliary_state_owners(&mut self) {
        for state in self.grouping.get_dummy_states() {
            self.solvable.set_owner(state, TwoPlayer::Eve);
        }
        for &state in &self.always_helping {
            self.solvable.set_owner(state, TwoPlayer::Eve);
        }
        for &state in &self.always_adversarial {
            self.solvable.set_owner(state, TwoPlayer::Adam);
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
        self.solvable.winner() == TwoPlayer::Eve
    }

    pub fn get_winning_region<C: CoalitionSpecifier>(&mut self, coalition: C) -> WinningRegion {
        self.set_state_owners(coalition);
        self.solvable.winning_region()
    }

    pub fn get_winning_region_with_current_owners(&mut self) -> WinningRegion {
        self.solvable.winning_region()
    }
}

impl<G: StateGroups, O: Objective> SimpleCooperativeGame
    for StateBasedResponsibilityNonstochasticGame<G, O>
{
    type PlayerDescriptions = super::GroupNames;

    fn get_player_count(&self) -> usize {
        self.grouping.get_count()
    }

    fn player_descriptions(&self) -> &Self::PlayerDescriptions {
        &self.group_names
    }

    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions {
        &mut self.group_names
    }

    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool {
        self.set_state_owners(coalition);
        self.solvable.winner() == TwoPlayer::Eve
    }
}

impl<G: StateGroups, O: Objective> MonotoneCooperativeGame
    for StateBasedResponsibilityNonstochasticGame<G, O>
{
}
