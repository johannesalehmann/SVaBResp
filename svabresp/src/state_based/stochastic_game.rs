use crate::shapley::{CoalitionSpecifier, CooperativeGame};
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::traits::SolvableStochasticGame;
use probabilistic_models::TwoPlayer;

pub struct StateBasedResponsibilityStochasticGame<G: StateGroups, A: SolvableStochasticGame> {
    solvable: A,
    pub grouping: G,
    always_helping: Vec<usize>,
    always_adversarial: Vec<usize>,
    group_names: super::GroupNames,
}
impl<G: StateGroups, A: SolvableStochasticGame> StateBasedResponsibilityStochasticGame<G, A> {
    pub fn new(
        solvable: A,
        grouping: G,
        always_helping: Vec<usize>,
        always_adversarial: Vec<usize>,
    ) -> Self {
        let group_info = super::GroupNames::from_grouping(&grouping);
        Self {
            solvable,
            grouping,
            always_helping,
            always_adversarial,
            group_names: group_info,
        }
    }

    // TODO: Currently, there is some duplication with the non-stochastic case. Can this be unified without introducing even more confusing traits?
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
        for state in self.grouping.get_dummy_states() {
            self.solvable.set_owner(state, TwoPlayer::PlayerOne);
        }
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
}

impl<G: StateGroups, A: SolvableStochasticGame> CooperativeGame
    for StateBasedResponsibilityStochasticGame<G, A>
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

    fn get_value<C: CoalitionSpecifier>(&mut self, coalition: C) -> f64 {
        self.set_state_owners(coalition);
        self.solvable.maximum_player_1_probability()
    }
}
