use crate::state_based::game::{ApIdx, Game, StateIdx};
use crate::{PrismModel, PrismProperty};

mod action_groups;
pub use action_groups::ActionGroupExtractionScheme;

mod individual_groups;
pub use individual_groups::IndividualGroupExtractionScheme;

mod module_groups;
pub use module_groups::ModuleGroupExtractionScheme;

mod value_groups;
pub use value_groups::ValueGroupExtractionScheme;

mod label_groups;
pub use label_groups::LabelGroupExtractionScheme;
use probabilistic_properties::Query;

mod relevant_states;

use crate::shapley::{ResponsibilityValues, SwitchingPairCollection};
pub use relevant_states::RelevantStates;

pub trait GroupExtractionScheme {
    type GroupType: super::super::grouping::StateGroups;

    #[allow(unused)]
    fn transform_prism(
        &mut self,
        prism_model: &mut PrismModel,
        property: &mut PrismProperty,
        character_to_line: &prism_parser::CharacterToLineMap,
    ) {
    }

    // TODO: Pass game by &mut reference again.
    fn create_groups(
        &mut self,
        game: Game,
        property: &Query<i64, f64, ApIdx>,
    ) -> (Game, GroupsAndAuxiliary<Self::GroupType>);

    fn get_syntax_elements<S: AsRef<str>>(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
        switching_pairs: &SwitchingPairCollection,
        group_names: &[S],
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting>;
}

pub struct GroupsAndAuxiliary<G: super::super::grouping::StateGroups> {
    pub groups: G,
    pub always_helping: Vec<StateIdx>,
    pub always_adversarial: Vec<StateIdx>,
}

impl<G: super::super::grouping::StateGroups> GroupsAndAuxiliary<G> {
    pub fn new(groups: G) -> Self {
        Self {
            groups,
            always_helping: Vec::new(),
            always_adversarial: Vec::new(),
        }
    }
    pub fn with_auxiliary(
        groups: G,
        always_helping: Vec<StateIdx>,
        always_adversarial: Vec<StateIdx>,
    ) -> Self {
        Self {
            groups,
            always_helping,
            always_adversarial,
        }
    }
}
