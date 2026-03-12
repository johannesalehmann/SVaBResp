use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter::once;

pub struct SubtractZeroOptimisation {}
impl Optimisation for SubtractZeroOptimisation {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Operation::SubtractInt = view.current_operation() {
            if let Some(Operation::PushInt(0)) = view.single_operation_from_stack(0) {
                view.replace_operations(
                    2,
                    view.operations_by_range(view.operand_range(1)).into_iter(),
                );
                return OptimisationResult::Applied;
            } else if let Some(Operation::PushInt(0)) = view.single_operation_from_stack(1) {
                view.replace_operations_and_reprocess(
                    2,
                    view.operations_by_range(view.operand_range(0))
                        .into_iter()
                        .chain(once(Operation::NegateInt)),
                );
                return OptimisationResult::Applied;
            }
        } else if let Operation::SubtractFloat = view.current_operation() {
            if let Some(Operation::PushFloat(0.0)) = view.single_operation_from_stack(0) {
                view.replace_operations(
                    2,
                    view.operations_by_range(view.operand_range(1)).into_iter(),
                );
                return OptimisationResult::Applied;
            } else if let Some(Operation::PushFloat(0.0)) = view.single_operation_from_stack(1) {
                view.replace_operations_and_reprocess(
                    2,
                    view.operations_by_range(view.operand_range(0))
                        .into_iter()
                        .chain(once(Operation::NegateFloat)),
                );
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::SubtractZeroOptimisation;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        value_minus_zero_integer,
        SubtractZeroOptimisation,
        [
            PushInt(0),
            PushInt(-5),
            PushInt(3),
            AddInt,
            PushInt(0),
            SubtractInt
        ],
        [PushInt(0), PushInt(-5), PushInt(3), AddInt]
    );
    test_optimisation!(
        zero_minus_value_integer,
        SubtractZeroOptimisation,
        [
            PushInt(0),
            PushInt(0),
            PushInt(-5),
            PushInt(3),
            AddInt,
            SubtractInt
        ],
        [PushInt(0), PushInt(-5), PushInt(3), AddInt, NegateInt]
    );
    test_optimisation!(
        not_applicable_integer,
        SubtractZeroOptimisation,
        [
            PushInt(0),
            PushInt(1),
            PushInt(-5),
            PushInt(3),
            AddInt,
            SubtractInt
        ],
        [
            PushInt(0),
            PushInt(1),
            PushInt(-5),
            PushInt(3),
            AddInt,
            SubtractInt
        ]
    );

    test_optimisation!(
        value_minus_zero_float,
        SubtractZeroOptimisation,
        [
            PushFloat(0.0),
            PushFloat(-4.9),
            PushFloat(2.9),
            AddFloat,
            PushFloat(0.0),
            SubtractFloat
        ],
        [PushFloat(0.0), PushFloat(-4.9), PushFloat(2.9), AddFloat]
    );
    test_optimisation!(
        zero_minus_value_float,
        SubtractZeroOptimisation,
        [
            PushFloat(0.0),
            PushFloat(0.0),
            PushFloat(-4.9),
            PushFloat(2.9),
            AddFloat,
            SubtractFloat
        ],
        [
            PushFloat(0.0),
            PushFloat(-4.9),
            PushFloat(2.9),
            AddFloat,
            NegateFloat
        ]
    );
    test_optimisation!(
        not_applicable_float,
        SubtractZeroOptimisation,
        [
            PushFloat(0.0),
            PushFloat(1.2),
            PushFloat(-4.9),
            PushFloat(2.9),
            AddFloat,
            SubtractFloat
        ],
        [
            PushFloat(0.0),
            PushFloat(1.2),
            PushFloat(-4.9),
            PushFloat(2.9),
            AddFloat,
            SubtractFloat
        ]
    );
}
