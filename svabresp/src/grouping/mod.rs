use tiny_pmc::{PrismModel, PrismQuery};

pub trait GroupingScheme {
    fn requires_prism(&self) -> bool {
        false
    }

    fn transform_prism(&mut self, _model: &mut PrismModel, _query: &mut PrismQuery) {}
}

pub struct IndividualGrouping;

impl GroupingScheme for IndividualGrouping {}
