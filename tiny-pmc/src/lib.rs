use chumsky::prelude::SimpleSpan;
use prism_model::{Identifier, VariableReference};

pub mod building;
pub mod parsing;

pub type PrismModel = prism_model::Model<(), Identifier<SimpleSpan>, VariableReference, SimpleSpan>;
pub type Property = prism_model::Property<VariableReference, SimpleSpan>;
