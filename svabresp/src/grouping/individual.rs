use super::{Grouping, GroupingScheme, StateGroup};
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::Index;
use probabilistic_models::traits::ReadStateSpace;
use tiny_pmc::{API, OptionalModel};

/// Creates one group per state. Groups are named after the state's valuation if the model has
/// valuations, and after the state's index otherwise.
pub struct IndividualGrouping;

impl GroupingScheme for IndividualGrouping {
    fn create_groups(
        &mut self,
        model: &mut OptionalModel,
        _query: &ProcessedQuery<API>,
    ) -> Grouping {
        let states = probabilistic_models::for_concrete_base!(model.as_ref(), |m| m.states());
        let valuations = model.state_valuations.as_ref();

        let groups = states
            .into_iter()
            .map(|state| {
                let name = match valuations {
                    Some(valuations) => format!("({})", valuations.entry(state)),
                    None => format!("s{}", state.raw()),
                };
                StateGroup {
                    name,
                    states: vec![state],
                }
            })
            .collect();
        Grouping::new(groups)
    }
}
