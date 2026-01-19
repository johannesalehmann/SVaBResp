use crate::{PrismModel, PrismProperty};
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};
use probabilistic_properties::Property;

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

mod relevant_states;

pub use relevant_states::RelevantStates;

pub trait GroupExtractionScheme {
    type GroupType: super::super::grouping::StateGroups;

    #[allow(unused)]
    fn transform_prism(&mut self, prism_model: &mut PrismModel, property: &mut PrismProperty) {}

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Property<AtomicProposition, f64>,
    ) -> GroupsAndAuxiliary<Self::GroupType>;
}

pub struct GroupsAndAuxiliary<G: super::super::grouping::StateGroups> {
    pub groups: G,
    pub always_helping: Vec<usize>,
    pub always_adversarial: Vec<usize>,
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
        always_helping: Vec<usize>,
        always_adversarial: Vec<usize>,
    ) -> Self {
        Self {
            groups,
            always_helping,
            always_adversarial,
        }
    }
}
