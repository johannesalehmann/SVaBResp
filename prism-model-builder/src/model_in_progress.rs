use probabilistic_models::{
    ActionCollection, AtomicPropositions, Builder, InitialStates, InitialStatesBuilder, ModelTypes,
    Predecessors, PredecessorsBuilder, ProbabilisticModel, State,
};
use std::collections::HashMap;

pub struct StateInProgress<M: ModelTypes> {
    pub valuation: M::Valuation,
    pub actions: <M::ActionCollection as ActionCollection<M::Distribution>>::Builder,
    pub atomic_propositions: M::AtomicPropositions,
    pub predecessors: <M::Predecessors as Predecessors>::Builder,
}

pub struct ModelInProgress<M: ModelTypes> {
    states: Vec<StateInProgress<M>>,
    valuation_to_state: HashMap<M::Valuation, usize>,

    initial_states: Vec<usize>,

    action_names: Vec<String>,
    action_name_indices: HashMap<String, usize>,

    atomic_proposition_count: usize,
}

impl<M: ModelTypes> ModelInProgress<M> {
    pub fn new(atomic_proposition_count: usize) -> Self {
        Self {
            states: Vec::new(),
            valuation_to_state: HashMap::new(),

            initial_states: Vec::new(),

            action_names: Vec::new(),
            action_name_indices: HashMap::new(),

            atomic_proposition_count,
        }
    }
    pub fn get_unnamed_action_name_index(&mut self) -> usize {
        self.get_action_name_index("unnamed")
    }

    pub fn get_action_name_index(&mut self, name: &str) -> usize {
        if let Some(&index) = self.action_name_indices.get(name) {
            index
        } else {
            let index = self.action_names.len();
            self.action_names.push(name.to_string());
            self.action_name_indices.insert(name.to_string(), index);
            index
        }
    }

    pub fn get_state(&self, index: usize) -> &StateInProgress<M> {
        &self.states[index]
    }

    pub fn get_state_mut(&mut self, index: usize) -> &mut StateInProgress<M> {
        &mut self.states[index]
    }

    pub fn get_state_index(&self, valuation: &M::Valuation) -> Option<usize> {
        self.valuation_to_state.get(&valuation).copied()
    }

    pub fn add_state(&mut self, valuation: M::Valuation) -> usize {
        let index = self.states.len();
        let action_builder: <M::ActionCollection as ActionCollection<M::Distribution>>::Builder =
            M::ActionCollection::get_builder();
        let atomic_propositions = <M::AtomicPropositions>::get_empty(self.atomic_proposition_count);
        let predecessors = <M::Predecessors as Predecessors>::Builder::create();
        self.valuation_to_state.insert(valuation.clone(), index);
        self.states.push(StateInProgress {
            valuation,
            actions: action_builder,
            atomic_propositions,
            predecessors,
        });
        index
    }

    pub fn add_initial_state(&mut self, index: usize) {
        self.initial_states.push(index);
    }

    pub fn into_model(
        self,
        valuation_context: <M::Valuation as probabilistic_models::Valuation>::ContextType,
    ) -> ProbabilisticModel<M> {
        let mut initial_states_builder = M::InitialStates::get_builder();
        for initial_state in self.initial_states {
            initial_states_builder.add_by_index(initial_state)
        }
        let initial_states = initial_states_builder.finish();

        let mut result = ProbabilisticModel::new(
            initial_states,
            valuation_context,
            self.atomic_proposition_count,
        );
        result.action_names = self.action_names;
        for state_in_progress in self.states.into_iter() {
            let state = State {
                valuation: state_in_progress.valuation,
                actions: state_in_progress.actions.finish(),
                atomic_propositions: state_in_progress.atomic_propositions,
                owner: <M::Owners as probabilistic_models::Owners>::default_owner(),
                predecessors: state_in_progress.predecessors.finish(),
            };
            result.states.push(state);
        }

        result
    }
}
