use probabilistic_models::{
    ActionCollection, AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, Valuation,
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

    fn create_groups<M: ModelTypes<Owners = TwoPlayer>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &mut Property<AtomicProposition, f64>,
    ) -> Self::GroupType {
        let mut builder = Self::GroupType::get_builder();

        // TODO: Determine states from which the property can both be fulfilled and refuted. Only those states may have positive responsibility

        for i in 0..game.states.len() {
            let state = &game.states[i];
            if state.actions.get_number_of_actions() == 1 {
                builder.add_state(i);
            }
        }
        builder.finish_group("Dummy states".to_string());

        for i in 0..game.states.len() {
            let state = &game.states[i];
            if state.actions.get_number_of_actions() > 1 {
                let label = format!("{}", state.valuation.displayable(&game.valuation_context));
                builder.add_state(i);
                builder.finish_group(label);
            }
        }

        builder.finish()
    }
}
