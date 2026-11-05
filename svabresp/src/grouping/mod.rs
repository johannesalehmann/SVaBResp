use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::valuations::{Type, Valuations};
use probabilistic_models::{DefaultIx, StateIndex, ValuationClassEntryIndex};
use tiny_pmc::{API, OptionalModel, PrismModel, PrismQuery};

mod actions;
pub use actions::ActionGrouping;

mod individual;
pub use individual::IndividualGrouping;

mod labels;
pub use labels::LabelGrouping;

mod modules;
pub use modules::ModuleGrouping;

mod variables;
pub use variables::VariableGrouping;

pub type StateIdx = StateIndex<DefaultIx>;

pub struct StateGroup {
    pub name: String,
    pub states: Vec<StateIdx>,
}

/// The result of applying a grouping scheme to a model.
pub struct Grouping {
    pub groups: Vec<StateGroup>,
    pub always_helping: Vec<StateIdx>,
    pub always_adversarial: Vec<StateIdx>,
}

impl Grouping {
    pub fn new(groups: Vec<StateGroup>) -> Self {
        Self {
            groups,
            always_helping: Vec::new(),
            always_adversarial: Vec::new(),
        }
    }
}

pub trait GroupingScheme {
    fn requires_prism(&self) -> bool {
        false
    }

    fn transform_prism(&mut self, _model: &mut PrismModel, _query: &mut PrismQuery) {}

    /// Computes the groups of the built model. Schemes may modify the model, e.g. to add auxiliary
    /// states.
    fn create_groups(&mut self, model: &mut OptionalModel, query: &ProcessedQuery<API>)
    -> Grouping;
}

/// Looks up a variable by name in a model whose states all share one valuation class.
fn variable(
    valuations: &Valuations<DefaultIx>,
    name: &str,
) -> Option<(ValuationClassEntryIndex<DefaultIx>, Type)> {
    let mut classes = valuations.classes().into_iter();
    let class = classes
        .next()
        .expect("The model does not have any valuation class");
    assert!(
        classes.next().is_none(),
        "Grouping requires a model whose states all share one valuation class"
    );
    let class = valuations.class(class);
    let index = class.index_by_name(name)?;
    Some((index, class.get(index).variable_type))
}
