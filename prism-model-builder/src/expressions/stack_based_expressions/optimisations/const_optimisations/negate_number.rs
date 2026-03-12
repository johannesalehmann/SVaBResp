use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use prism_model::VariableReference;
use std::iter;

pub struct NegateNumberOptimisation {}

impl Optimisation for NegateNumberOptimisation {
    fn apply(&self, view: &mut OperationView<VariableReference>) -> OptimisationResult {
        if let Operation::NegateInt = view.current_operation() {
            if let Some(Operation::PushInt(i)) = view.single_operation_from_stack(0) {
                view.replace_operations(1, iter::once(Operation::PushInt(-i)));
                return OptimisationResult::Applied;
            }
        } else if let Operation::NegateFloat = view.current_operation() {
            if let Some(Operation::PushFloat(f)) = view.single_operation_from_stack(0) {
                view.replace_operations(1, iter::once(Operation::PushFloat(-f)));
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::NegateNumberOptimisation;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        integer_applicable_complex,
        NegateNumberOptimisation,
        [PushBool(true), PushInt(3), NegateInt],
        [PushBool(true), PushInt(-3)]
    );
    test_optimisation!(
        integer_applicable_chained,
        NegateNumberOptimisation,
        [PushBool(true), PushInt(3), NegateInt, NegateInt],
        [PushBool(true), PushInt(3)]
    );
    test_optimisation!(
        integer_not_applicable,
        NegateNumberOptimisation,
        [
            PushBool(true),
            PushInt(3),
            PushInt(2),
            MultiplyInt,
            NegateInt
        ],
        [
            PushBool(true),
            PushInt(3),
            PushInt(2),
            MultiplyInt,
            NegateInt
        ]
    );
    test_optimisation!(
        float_applicable_complex,
        NegateNumberOptimisation,
        [PushBool(true), PushFloat(3.0), NegateFloat],
        [PushBool(true), PushFloat(-3.0)]
    );
    test_optimisation!(
        float_applicable_chained,
        NegateNumberOptimisation,
        [PushBool(true), PushFloat(3.0), NegateFloat, NegateFloat],
        [PushBool(true), PushFloat(3.0)]
    );
    test_optimisation!(
        float_not_applicable,
        NegateNumberOptimisation,
        [
            PushBool(true),
            PushFloat(3.0),
            PushFloat(2.0),
            MultiplyFloat,
            NegateFloat
        ],
        [
            PushBool(true),
            PushFloat(3.0),
            PushFloat(2.0),
            MultiplyFloat,
            NegateFloat
        ]
    );
}
