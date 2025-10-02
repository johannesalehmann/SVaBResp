use chumsky::span::SimpleSpan;
use prism_model::{Identifier, VariableReference};

pub type PrismModel = prism_model::Model<(), Identifier<SimpleSpan>, VariableReference, SimpleSpan>;
pub type Expression = prism_model::Expression<VariableReference, SimpleSpan>;
pub enum HighLevelModel {
    Prism(PrismModel),
}

pub enum HighLevelProperty {
    PMaxReach(StateDescriptor),
    PMinReach(StateDescriptor),
    PReach(StateDescriptor),
}

pub enum StateDescriptor {
    Expression(prism_model::Expression<VariableReference, SimpleSpan>),
}
