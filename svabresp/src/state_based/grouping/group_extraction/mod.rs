use crate::{PrismModel, PrismProperty};
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};
use probabilistic_properties::Property;

mod action_groups;

mod individual_groups;
pub use individual_groups::IndividualGroupExtractionScheme;

mod module_groups;
pub use module_groups::ModuleExtractionScheme;

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
    ) -> Self::GroupType;
}
