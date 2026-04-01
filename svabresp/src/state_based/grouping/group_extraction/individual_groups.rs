use crate::shapley::ResponsibilityValues;
use crate::state_based::grouping::{GroupsAndAuxiliary, VectorStateGroupBuilder};
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, Valuation, VectorPredecessors,
};
use probabilistic_properties::Query;

pub struct IndividualGroupExtractionScheme {
    restrict_to_relevant_states: bool,
}

impl IndividualGroupExtractionScheme {
    pub fn new() -> Self {
        Self {
            restrict_to_relevant_states: true,
        }
    }
    pub fn including_irrelevant_states() -> Self {
        Self {
            restrict_to_relevant_states: false,
        }
    }

    fn build_groups_with_relevant_states<
        M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>,
    >(
        &self,
        builder: &mut VectorStateGroupBuilder,
        game: &mut ProbabilisticModel<M>,
        property: &Query<i64, f64, AtomicProposition>,
    ) {
        let relevant_states = super::RelevantStates::compute(game, property);

        for i in 0..game.states.len() {
            let state = &game.states[i];
            if relevant_states.is_relevant(i) {
                let label = format!("{}", state.valuation.displayable(&game.valuation_context));
                builder.add_state(i);
                builder.finish_group(label);
            }
        }

        for state in relevant_states.into_dummy_states() {
            builder.add_dummy_state(state);
        }
    }

    fn build_groups_with_all_states<
        M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>,
    >(
        &self,
        builder: &mut VectorStateGroupBuilder,
        game: &mut ProbabilisticModel<M>,
    ) {
        for i in 0..game.states.len() {
            let state = &game.states[i];
            let label = format!("{}", state.valuation.displayable(&game.valuation_context));
            builder.add_state(i);
            builder.finish_group(label);
        }
    }
}

impl super::GroupExtractionScheme for IndividualGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Query<i64, f64, AtomicProposition>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let mut builder = Self::GroupType::get_builder();

        match self.restrict_to_relevant_states {
            true => self.build_groups_with_relevant_states(&mut builder, game, property),
            false => self.build_groups_with_all_states(&mut builder, game),
        };

        GroupsAndAuxiliary::new(builder.finish())
    }

    fn get_syntax_elements(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting> {
        let _ = values;
        None
    }
}
