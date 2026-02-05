use chumsky::prelude::SimpleSpan;
use prism_model::{Expression, Identifier, VariableReference};

pub mod building;
pub mod checking;
pub mod parsing;

pub type PrismModel = prism_model::Model<
    (),
    Identifier<SimpleSpan>,
    Expression<VariableReference, SimpleSpan>,
    VariableReference,
    SimpleSpan,
>;
pub type PrismProperty = probabilistic_properties::Property<
    Expression<VariableReference, SimpleSpan>,
    Expression<VariableReference, SimpleSpan>,
>;
