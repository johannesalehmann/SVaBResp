use crate::expressions::stack_based_expressions::StackBasedExpression;

pub struct OptimisationCollection {
    optimisations: Vec<Box<dyn Optimisation>>,
}

pub trait Optimisation {}

pub fn apply_optimisations<V>(
    expression: &mut StackBasedExpression<V>,
    optimisations: &OptimisationCollection,
) {
}
