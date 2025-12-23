use crate::{PrismModel, PrismProperty};
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};
use probabilistic_properties::Property;

mod action_groups;

mod individual_groups;
pub use individual_groups::IndividualGroupExtractionScheme;

mod module_groups;
mod value_groups;

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
