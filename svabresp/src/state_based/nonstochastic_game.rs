use crate::shapley::{CoalitionSpecifier, MonotoneCooperativeGame, SimpleCooperativeGame};
use crate::state_based::game::{Game, Objective, SolvableGame, StateIdx, WinningRegion};
use crate::state_based::group_ownership::GroupOwnership;
use crate::state_based::grouping::StateGroups;
use probabilistic_models::owners::TwoPlayer;

pub struct StateBasedResponsibilityNonstochasticGame<G: StateGroups, O: Objective> {
    solvable: SolvableGame<O>,
    ownership: GroupOwnership<G>,
}

impl<G: StateGroups, O: Objective> StateBasedResponsibilityNonstochasticGame<G, O> {
    pub fn new(
        solvable: SolvableGame<O>,
        grouping: G,
        always_helping: Vec<StateIdx>,
        always_adversarial: Vec<StateIdx>,
    ) -> Self {
        Self {
            solvable,
            ownership: GroupOwnership::new(grouping, always_helping, always_adversarial),
        }
    }

    pub fn map_grouping<G2: StateGroups, F: Fn(G) -> G2>(
        self,
        map: F,
    ) -> StateBasedResponsibilityNonstochasticGame<G2, O> {
        StateBasedResponsibilityNonstochasticGame {
            solvable: self.solvable,
            ownership: self.ownership.map_grouping(map),
        }
    }

    pub fn into_grouping(self) -> G {
        self.ownership.grouping
    }

    pub fn get_grouping(&self) -> &G {
        &self.ownership.grouping
    }

    pub fn get_game(&self) -> &Game {
        self.solvable.game()
    }

    pub fn set_state_owners<C: CoalitionSpecifier>(&mut self, coalition: C) {
        self.ownership.set_state_owners(coalition, &mut self.solvable)
    }

    pub fn set_auxiliary_state_owners(&mut self) {
        self.ownership.set_auxiliary_state_owners(&mut self.solvable)
    }

    pub fn set_group_owners(&mut self, group_index: usize, owner: TwoPlayer) {
        self.ownership
            .set_group_owners(group_index, owner, &mut self.solvable)
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
        self.ownership.group_count()
    }

    fn player_descriptions(&self) -> &Self::PlayerDescriptions {
        self.ownership.group_names()
    }

    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions {
        self.ownership.group_names_mut()
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
