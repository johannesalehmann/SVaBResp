use chumsky::prelude::SimpleSpan;
use prism_model::{Expression, Identifier, VariableReference};
use std::fmt::Formatter;

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
pub type PrismQuery = probabilistic_properties::Query<
    Expression<VariableReference, SimpleSpan>,
    Expression<VariableReference, SimpleSpan>,
    Expression<VariableReference, SimpleSpan>,
>;

pub enum CheckerError {
    NoSuitableAlgorithm,
}
impl std::fmt::Debug for CheckerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckerError::NoSuitableAlgorithm => write!(f, "No suitable model-checking algorithm"),
        }
    }
}
