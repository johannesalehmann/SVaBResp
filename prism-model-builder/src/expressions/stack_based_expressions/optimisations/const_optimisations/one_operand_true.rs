use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter::once;

pub struct OneOperandTrueOptimisations {}
impl Optimisation for OneOperandTrueOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        let other_operand =
            if let Some(Operation::PushBool(true)) = view.single_operation_from_stack(0) {
                view.try_operand_range(1)
            } else if let Some(Operation::PushBool(true)) = view.single_operation_from_stack(1) {
                view.try_operand_range(0)
            } else {
                None
            };

        if let Some(other_operand) = other_operand {
            if let Operation::Conjunction = view.current_operation() {
                view.replace_operations(2, view.operations_by_range(other_operand).into_iter());
                return OptimisationResult::Applied;
            } else if let Operation::Disjunction = view.current_operation() {
                view.replace_operations(2, once(Operation::PushBool(true)));
                return OptimisationResult::Applied;
            } else if let Operation::IfAndOnlyIf = view.current_operation() {
                view.replace_operations(2, view.operations_by_range(other_operand).into_iter());
                return OptimisationResult::Applied;
            } else if let Operation::EqualsBool = view.current_operation() {
                view.replace_operations(2, view.operations_by_range(other_operand).into_iter());
                return OptimisationResult::Applied;
            } else if let Operation::NotEqualsBool = view.current_operation() {
                view.replace_operations_and_reprocess(
                    2,
                    view.operations_by_range(other_operand)
                        .into_iter()
                        .chain(once(Operation::NegateBool)),
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
    use super::OneOperandTrueOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;
    macro_rules! test_one_operand_true {
        ($name:ident, [$($other_ops:expr),*], $operator:expr, [$($result:expr),*]) => {
            paste::paste! {
                test_optimisation!(
                    [<$name >],
                    OneOperandTrueOptimisations,
                    [PushBool(true), PushBool(true), $($other_ops),*, $operator],
                    [PushBool(true), $($result),*]
                );
                test_optimisation!(
                    [<$name _switched>],
                    OneOperandTrueOptimisations,
                    [PushBool(true), $($other_ops),*, PushBool(true), $operator],
                    [PushBool(true), $($result),*]
                );
                test_optimisation!(
                    [<$name _not_applicable>],
                    OneOperandTrueOptimisations,
                    [PushBool(true), PushBool(true), NegateBool, $($other_ops),*, $operator],
                    [PushBool(true), PushBool(true), NegateBool, $($other_ops),*, $operator]
                );
                test_optimisation!(
                    [<$name _switched_not_applicable>],
                    OneOperandTrueOptimisations,
                    [PushBool(true), $($other_ops),*, PushBool(true), NegateBool, $operator],
                    [PushBool(true), $($other_ops),*, PushBool(true), NegateBool, $operator]
                );
            }
        };
    }

    test_one_operand_true!(
        conjunction,
        [PushBool(true), NegateBool],
        Conjunction,
        [PushBool(true), NegateBool]
    );
    test_one_operand_true!(
        disjunction,
        [PushBool(true), NegateBool],
        Disjunction,
        [PushBool(true)]
    );
    test_one_operand_true!(
        iff,
        [PushBool(true), NegateBool],
        IfAndOnlyIf,
        [PushBool(true), NegateBool]
    );
    test_one_operand_true!(
        eq,
        [PushBool(true), NegateBool],
        EqualsBool,
        [PushBool(true), NegateBool]
    );
    test_one_operand_true!(
        neq,
        [PushBool(true), NegateBool],
        NotEqualsBool,
        [PushBool(true), NegateBool, NegateBool]
    );
}
