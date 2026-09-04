use crate::shapley::{CoalitionSpecifier, CooperativeGame};
use crate::state_based::game::{Game, SolvableValueGame, StateIdx, ValueObjective};
use crate::state_based::group_ownership::GroupOwnership;
use crate::state_based::grouping::StateGroups;

pub struct StateBasedResponsibilityStochasticGame<G: StateGroups, O: ValueObjective> {
    solvable: SolvableValueGame<O>,
    ownership: GroupOwnership<G>,
}

impl<G: StateGroups, O: ValueObjective> StateBasedResponsibilityStochasticGame<G, O> {
    pub fn new(
        solvable: SolvableValueGame<O>,
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
    ) -> StateBasedResponsibilityStochasticGame<G2, O> {
        StateBasedResponsibilityStochasticGame {
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
}

impl<G: StateGroups, O: ValueObjective> CooperativeGame
    for StateBasedResponsibilityStochasticGame<G, O>
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

    fn get_value<C: CoalitionSpecifier>(&mut self, coalition: C) -> f64 {
        self.ownership
            .set_state_owners(coalition, &mut self.solvable);
        self.solvable.value()
    }
}
