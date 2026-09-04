pub use num_bigint;
pub use num_rational;
pub use num_traits;

mod responsibility_task;
pub use responsibility_task::*;

pub mod shapley;

pub mod state_based;
pub mod syntax_highlighting;

pub use prism_model_builder::UserProvidedConstValue;

use prism_model::{Expression, FullSpan, Identifier, VariableReference};

type PrismModel = prism_model::Model<
    VariableReference,
    FullSpan,
    Expression<VariableReference, FullSpan>,
    Identifier<FullSpan>,
>;
type PrismProperty = probabilistic_properties::Query<
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
    Expression<VariableReference, FullSpan>,
>;
