use probabilistic_model_algorithms::two_player_games::non_probabilistic::winning_region;
use probabilistic_models::{
    ActionCollection, AtomicProposition, ModelTypes, TwoPlayer, VectorPredecessors,
};
use probabilistic_properties::Property;
use std::collections::HashSet;

pub struct RelevantStates {
    relevant_states: HashSet<usize>,
    dummy_states: Vec<usize>,
}

impl RelevantStates {
    pub fn compute<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        model: &mut probabilistic_models::ProbabilisticModel<M>,
        property: &Property<AtomicProposition, f64>,
    ) -> Self {
        for state in &mut model.states {
            state.owner = TwoPlayer::PlayerOne;
        }
        let max_winning = winning_region(model, property);

        for state in &mut model.states {
            state.owner = TwoPlayer::PlayerTwo;
        }
        let min_winning = winning_region(model, property);

        let mut relevant_states = HashSet::new();
        let mut dummy_states = Vec::new();
        for i in 0..model.states.len() {
            let state = &model.states[i];
            if state.actions.get_number_of_actions() > 1
                && max_winning.contains(i)
                && !min_winning.contains(i)
            {
                relevant_states.insert(i);
            } else {
                dummy_states.push(i);
            }
        }
        Self {
            relevant_states,
            dummy_states,
        }
    }

    pub fn is_relevant(&self, index: usize) -> bool {
        self.relevant_states.contains(&index)
    }

    pub fn into_dummy_states(self) -> Vec<usize> {
        self.dummy_states
    }
}
