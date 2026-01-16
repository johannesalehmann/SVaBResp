use crate::state_based::grouping::GroupsAndAuxiliary;
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, Valuation, VectorPredecessors,
};
use probabilistic_properties::Property;

pub struct IndividualGroupExtractionScheme {}

impl IndividualGroupExtractionScheme {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::GroupExtractionScheme for IndividualGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Property<AtomicProposition, f64>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let mut builder = Self::GroupType::get_builder();

        let relevant_states = super::RelevantStates::compute(game, property);

        for i in 0..game.states.len() {
            let state = &game.states[i];
            if relevant_states.is_relevant(i) {
                let label = format!("{}", state.valuation.displayable(&game.valuation_context));
                builder.add_state(i);
                builder.finish_group(label);
            }
        }

        builder.create_group_from_vec(
            relevant_states.into_dummy_states(),
            "dummy states".to_string(),
        );

        GroupsAndAuxiliary::new(builder.finish())
    }
}
