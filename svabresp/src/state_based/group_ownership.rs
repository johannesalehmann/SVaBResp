use crate::shapley::CoalitionSpecifier;
use crate::state_based::GroupNames;
use crate::state_based::game::{Objective, ValueObjective};
use crate::state_based::game::{SolvableGame, SolvableValueGame, StateIdx};
use crate::state_based::grouping::StateGroups;
use probabilistic_models::owners::TwoPlayer;
// Abstracts over how non-stochastic and stochastic games handle owners. Non-stochastic ones don't
// modify them in the game, instead directly adjusting the solver's context, whereas stochastic ones
// change them in the game.
pub trait SetStateOwner {
    fn set_owner(&mut self, state: StateIdx, owner: TwoPlayer);
}

impl<O: Objective> SetStateOwner for SolvableGame<O> {
    fn set_owner(&mut self, state: StateIdx, owner: TwoPlayer) {
        SolvableGame::set_owner(self, state, owner)
    }
}

impl<O: ValueObjective> SetStateOwner for SolvableValueGame<O> {
    fn set_owner(&mut self, state: StateIdx, owner: TwoPlayer) {
        SolvableValueGame::set_owner(self, state, owner)
    }
}

pub struct GroupOwnership<G: StateGroups> {
    pub grouping: G,
    always_helping: Vec<StateIdx>,
    always_adversarial: Vec<StateIdx>,
    group_names: GroupNames,
}

impl<G: StateGroups> GroupOwnership<G> {
    pub fn new(
        grouping: G,
        always_helping: Vec<StateIdx>,
        always_adversarial: Vec<StateIdx>,
    ) -> Self {
        let group_names = GroupNames::from_grouping(&grouping);
        Self {
            grouping,
            always_helping,
            always_adversarial,
            group_names,
        }
    }

    pub fn map_grouping<G2: StateGroups, F: Fn(G) -> G2>(self, map: F) -> GroupOwnership<G2> {
        GroupOwnership::new(
            map(self.grouping),
            self.always_helping,
            self.always_adversarial,
        )
    }

    pub fn group_count(&self) -> usize {
        self.grouping.get_count()
    }

    pub fn group_names(&self) -> &GroupNames {
        &self.group_names
    }

    pub fn group_names_mut(&mut self) -> &mut GroupNames {
        &mut self.group_names
    }

    pub fn set_state_owners<C: CoalitionSpecifier, S: SetStateOwner>(
        &self,
        coalition: C,
        target: &mut S,
    ) {
        self.set_auxiliary_state_owners(target);
        for i in 0..self.grouping.get_count() {
            if coalition.is_in_coalition(i) {
                self.set_group_owners(i, TwoPlayer::Eve, target);
            } else {
                self.set_group_owners(i, TwoPlayer::Adam, target);
            }
        }
    }

    pub fn set_auxiliary_state_owners<S: SetStateOwner>(&self, target: &mut S) {
        for state in self.grouping.get_dummy_states() {
            target.set_owner(state, TwoPlayer::Eve);
        }
        for &state in &self.always_helping {
            target.set_owner(state, TwoPlayer::Eve);
        }
        for &state in &self.always_adversarial {
            target.set_owner(state, TwoPlayer::Adam);
        }
    }

    pub fn set_group_owners<S: SetStateOwner>(
        &self,
        group_index: usize,
        owner: TwoPlayer,
        target: &mut S,
    ) {
        for state in self.grouping.get_states(group_index) {
            target.set_owner(state, owner);
        }
    }
}
