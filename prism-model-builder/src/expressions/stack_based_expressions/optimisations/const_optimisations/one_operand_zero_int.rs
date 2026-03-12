use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter::once;

pub struct OneOperandZeroIntegerOptimisations {}
impl Optimisation for OneOperandZeroIntegerOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        let other_operand = if let Some(Operation::PushInt(0)) = view.single_operation_from_stack(0)
        {
            view.try_operand_range(1)
        } else if let Some(Operation::PushInt(0)) = view.single_operation_from_stack(1) {
            view.try_operand_range(0)
        } else {
            None
        };

        if let Some(other_operand) = other_operand {
            if let Operation::MultiplyInt = view.current_operation() {
                view.replace_operations(2, once(Operation::PushInt(0)));
                return OptimisationResult::Applied;
            } else if let Operation::AddInt = view.current_operation() {
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
    use super::OneOperandZeroIntegerOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    macro_rules! test_one_operand_one {
        ($name:ident, [$($other_ops:expr),*], $operator:expr, [$($result:expr),*]) => {
            paste::paste! {
                test_optimisation!(
                    [<$name >],
                    OneOperandZeroIntegerOptimisations,
                    [PushInt(0), PushInt(0), $($other_ops),*, $operator],
                    [PushInt(0), $($result),*]
                );
                test_optimisation!(
                    [<$name _switched>],
                    OneOperandZeroIntegerOptimisations,
                    [PushInt(0), $($other_ops),*, PushInt(0), $operator],
                    [PushInt(0), $($result),*]
                );
                test_optimisation!(
                    [<$name _not_applicable>],
                    OneOperandZeroIntegerOptimisations,
                    [PushInt(0), PushInt(1), $($other_ops),*, $operator],
                    [PushInt(0), PushInt(1), $($other_ops),*, $operator]
                );
                test_optimisation!(
                    [<$name _switched_not_applicable>],
                    OneOperandZeroIntegerOptimisations,
                    [PushInt(0), $($other_ops),*, PushInt(1), $operator],
                    [PushInt(0), $($other_ops),*, PushInt(1), $operator]
                );
            }
        };
    }
    test_one_operand_one!(
        multiplication,
        [PushFloat(3.0), PushFloat(2.0), AddFloat, Round],
        MultiplyInt,
        [PushInt(0)]
    );
    test_one_operand_one!(
        addition,
        [PushFloat(3.0), PushFloat(2.0), AddFloat, Round],
        AddInt,
        [PushFloat(3.0), PushFloat(2.0), AddFloat, Round]
    );
    test_optimisation!(
        multiplication_nested,
        OneOperandZeroIntegerOptimisations,
        [
            PushInt(3),
            PushInt(1),
            PushInt(3),
            MultiplyInt,
            PushInt(1),
            PushInt(0),
            MultiplyInt,
            MultiplyInt
        ],
        [PushInt(3), PushInt(0)]
    );
}
