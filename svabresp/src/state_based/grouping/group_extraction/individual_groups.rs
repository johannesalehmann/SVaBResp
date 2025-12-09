use probabilistic_models::{AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer};
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

        builder.finish()
    }
}
