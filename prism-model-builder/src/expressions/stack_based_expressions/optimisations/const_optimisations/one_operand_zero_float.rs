use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter::once;

pub struct OneOperandZeroFloatOptimisations {}
impl Optimisation for OneOperandZeroFloatOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        let other_operand =
            if let Some(Operation::PushFloat(0.0)) = view.single_operation_from_stack(0) {
                view.try_operand_range(1)
            } else if let Some(Operation::PushFloat(0.0)) = view.single_operation_from_stack(1) {
                view.try_operand_range(0)
            } else {
                None
            };

        if let Some(other_operand) = other_operand {
            if let Operation::MultiplyFloat = view.current_operation() {
                view.replace_operations(2, once(Operation::PushFloat(0.0)));
                return OptimisationResult::Applied;
            } else if let Operation::AddFloat = view.current_operation() {
                view.replace_operations(2, view.operations_by_range(other_operand).into_iter());
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::OneOperandZeroFloatOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;
    macro_rules! test_one_operand_one {
        ($name:ident, [$($other_ops:expr),*], $operator:expr, [$($result:expr),*]) => {
            paste::paste! {
                test_optimisation!(
                    [<$name >],
                    OneOperandZeroFloatOptimisations,
                    [PushFloat(0.0), PushFloat(0.0), $($other_ops),*, $operator],
                    [PushFloat(0.0), $($result),*]
                );
                test_optimisation!(
                    [<$name _switched>],
                    OneOperandZeroFloatOptimisations,
                    [PushFloat(0.0), $($other_ops),*, PushFloat(0.0), $operator],
                    [PushFloat(0.0), $($result),*]
                );
                test_optimisation!(
                    [<$name _not_applicable>],
                    OneOperandZeroFloatOptimisations,
                    [PushFloat(0.0), PushFloat(0.1), $($other_ops),*, $operator],
                    [PushFloat(0.0), PushFloat(0.1), $($other_ops),*, $operator]
                );
                test_optimisation!(
                    [<$name _switched_not_applicable>],
                    OneOperandZeroFloatOptimisations,
                    [PushFloat(0.0), $($other_ops),*, PushFloat(0.1), $operator],
                    [PushFloat(0.0), $($other_ops),*, PushFloat(0.1), $operator]
                );
            }
        };
    }
    test_one_operand_one!(
        multiplication,
        [PushFloat(3.0), PushFloat(2.0), AddFloat],
        MultiplyFloat,
        [PushFloat(0.0)]
    );
    test_one_operand_one!(
        addition,
        [PushFloat(3.0), PushFloat(2.0), AddFloat],
        AddFloat,
        [PushFloat(3.0), PushFloat(2.0), AddFloat]
    );
    test_optimisation!(
        multiplication_nested,
        OneOperandZeroFloatOptimisations,
        [
            PushInt(3),
            PushFloat(1.0),
            PushFloat(3.0),
            MultiplyFloat,
            PushFloat(1.0),
            PushFloat(0.0),
            MultiplyFloat,
            MultiplyFloat
        ],
        [PushInt(3), PushFloat(0.0)]
    );
}
