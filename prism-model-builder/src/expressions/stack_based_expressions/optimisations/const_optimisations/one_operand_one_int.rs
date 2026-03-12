use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};

pub struct OneOperandOneIntegerOptimisations {}
impl Optimisation for OneOperandOneIntegerOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        let other_operand = if let Some(Operation::PushInt(1)) = view.single_operation_from_stack(0)
        {
            view.try_operand_range(1)
        } else if let Some(Operation::PushInt(1)) = view.single_operation_from_stack(1) {
            view.try_operand_range(0)
        } else {
            None
        };

        if let Some(other_operand) = other_operand {
            if let Operation::MultiplyInt = view.current_operation() {
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
    use super::OneOperandOneIntegerOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    macro_rules! test_one_operand_one {
        ($name:ident, [$($other_ops:expr),*], $operator:expr, [$($result:expr),*]) => {
            paste::paste! {
                test_optimisation!(
                    [<$name >],
                    OneOperandOneIntegerOptimisations,
                    [PushInt(1), PushInt(1), $($other_ops),*, $operator],
                    [PushInt(1), $($result),*]
                );
                test_optimisation!(
                    [<$name _switched>],
                    OneOperandOneIntegerOptimisations,
                    [PushInt(1), $($other_ops),*, PushInt(1), $operator],
                    [PushInt(1), $($result),*]
                );
                test_optimisation!(
                    [<$name _not_applicable>],
                    OneOperandOneIntegerOptimisations,
                    [PushInt(1), PushInt(2), $($other_ops),*, $operator],
                    [PushInt(1), PushInt(2), $($other_ops),*, $operator]
                );
                test_optimisation!(
                    [<$name _switched_not_applicable>],
                    OneOperandOneIntegerOptimisations,
                    [PushInt(1), $($other_ops),*, PushInt(2), $operator],
                    [PushInt(1), $($other_ops),*, PushInt(2), $operator]
                );
            }
        };
    }
    test_one_operand_one!(
        multiplication,
        [PushFloat(3.0), PushFloat(2.0), AddFloat, Round],
        MultiplyInt,
        [PushFloat(3.0), PushFloat(2.0), AddFloat, Round]
    );
    test_optimisation!(
        multiplication_nested,
        OneOperandOneIntegerOptimisations,
        [
            PushInt(3),
            PushInt(1),
            PushInt(3),
            PushInt(1),
            MultiplyInt,
            MultiplyInt
        ],
        [PushInt(3), PushInt(3)]
    );
}
