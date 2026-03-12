use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter;

pub struct IntToFloatOptimisation {}
impl Optimisation for IntToFloatOptimisation {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Operation::IntToFloat = view.current_operation() {
            if let Some(Operation::PushInt(i)) = view.single_operation_from_stack(0) {
                view.replace_operations(1, iter::once(Operation::PushFloat(*i as f64)));
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::IntToFloatOptimisation;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        applicable,
        IntToFloatOptimisation,
        [PushInt(0), PushInt(3), IntToFloat],
        [PushInt(0), PushFloat(3.0)]
    );
    test_optimisation!(
        not_applicable,
        IntToFloatOptimisation,
        [PushInt(0), PushInt(3), PushInt(2), AddInt, IntToFloat],
        [PushInt(0), PushInt(3), PushInt(2), AddInt, IntToFloat]
    );
}
