use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::{DefaultIx, StateIndex};
use tiny_pmc::{API, OptionalModel, PrismModel, PrismQuery};

mod individual;
pub use individual::IndividualGrouping;

pub type StateIdx = StateIndex<DefaultIx>;

pub struct StateGroup {
    pub name: String,
    pub states: Vec<StateIdx>,
}

pub trait GroupingScheme {
    fn requires_prism(&self) -> bool {
        false
    }

    fn transform_prism(&mut self, _model: &mut PrismModel, _query: &mut PrismQuery) {}

    fn create_groups(
        &mut self,
        model: &OptionalModel,
        query: &ProcessedQuery<API>,
    ) -> Vec<StateGroup>;
}
