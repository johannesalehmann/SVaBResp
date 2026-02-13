mod stack_based_expressions;
mod tree_walking_enumerator;

pub use tree_walking_enumerator::TreeWalkingEvaluator;

use prism_model::{Expression, VariableRange, VariableReference};

pub trait Evaluator {
    fn create() -> Self;
    fn evaluate_as_int<V: ValuationSource, S: Clone>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> i64;
    fn evaluate_as_bool<V: ValuationSource, S: Clone>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> bool;
    fn evaluate_as_float<V: ValuationSource, S: Clone>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> f64;
}

pub trait ValuationSource {
    fn get_int(&self, index: VariableReference) -> i64;
    fn get_bool(&self, index: VariableReference) -> bool;
    fn get_float(&self, index: VariableReference) -> f64;
    fn get_type(&self, index: VariableReference) -> VariableType;
}

#[derive(Copy, Clone)]
pub enum VariableType {
    Int,
    Bool,
    Float,
}

impl VariableType {
    pub fn from_range<E, S: Clone>(range: &VariableRange<E, S>) -> Self {
        match range {
            VariableRange::BoundedInt { .. } => VariableType::Int,
            VariableRange::UnboundedInt { .. } => VariableType::Int,
            VariableRange::Boolean { .. } => VariableType::Bool,
            VariableRange::Float { .. } => VariableType::Float,
        }
    }
}
