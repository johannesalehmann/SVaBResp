use crate::shapley::{ResponsibilityValues, SwitchingPairCollection};
use crate::state_based::game::{ApIdx, Game};
use crate::state_based::grouping::{GroupsAndAuxiliary, VectorStateGroupBuilder};
use probabilistic_models::traits::{ReadStateSpace, ReadValuations};
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

    fn build_groups_with_relevant_states(
        &self,
        builder: &mut VectorStateGroupBuilder,
        game: &mut Game,
        property: &Query<i64, f64, ApIdx>,
    ) {
        let relevant_states = super::RelevantStates::compute(game, property);

        for state in game.states() {
            if relevant_states.is_relevant(state) {
                let label = format!("({})", game.state_valuation(state));
                builder.add_state(state);
                builder.finish_group(label);
            }
        }

        for state in relevant_states.into_dummy_states() {
            builder.add_dummy_state(state);
        }
    }

    fn build_groups_with_all_states(&self, builder: &mut VectorStateGroupBuilder, game: &mut Game) {
        for state in game.states() {
            let label = format!("({})", game.state_valuation(state));
            builder.add_state(state);
            builder.finish_group(label);
        }
    }
}

impl super::GroupExtractionScheme for IndividualGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn create_groups(
        &mut self,
        mut game: Game,
        property: &Query<i64, f64, ApIdx>,
    ) -> (Game, GroupsAndAuxiliary<Self::GroupType>) {
        let mut builder = Self::GroupType::get_builder();

        match self.restrict_to_relevant_states {
            true => self.build_groups_with_relevant_states(&mut builder, &mut game, property),
            false => self.build_groups_with_all_states(&mut builder, &mut game),
        };

        (game, GroupsAndAuxiliary::new(builder.finish()))
    }

    fn get_syntax_elements<S: AsRef<str>>(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
        switching_pairs: &SwitchingPairCollection,
        player_names: &[S],
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting> {
        let _ = (values, switching_pairs, player_names);
        None
    }
}
