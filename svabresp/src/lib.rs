pub use num_bigint;
pub use num_rational;
pub use num_traits;

mod responsibility_task;
pub mod shapley;
pub mod state_based;
pub use responsibility_task::*;

pub use prism_model_builder::UserProvidedConstValue;

use chumsky::span::SimpleSpan;
use prism_model::{Expression, Identifier, VariableReference};

type PrismModel = prism_model::Model<
    (),
    Identifier<SimpleSpan>,
    Expression<VariableReference, SimpleSpan>,
    VariableReference,
    SimpleSpan,
>;
type PrismProperty = probabilistic_properties::Property<
    Expression<VariableReference, SimpleSpan>,
    Expression<VariableReference, SimpleSpan>,
>;
