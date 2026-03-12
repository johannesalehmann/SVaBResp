use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter::once;

pub struct OneOperandFalseOptimisations {}
impl Optimisation for OneOperandFalseOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        let other_operand =
            if let Some(Operation::PushBool(false)) = view.single_operation_from_stack(0) {
                view.try_operand_range(1)
            } else if let Some(Operation::PushBool(false)) = view.single_operation_from_stack(1) {
                view.try_operand_range(0)
            } else {
                None
            };

        if let Some(other_operand) = other_operand {
            if let Operation::Conjunction = view.current_operation() {
                view.replace_operations(2, once(Operation::PushBool(false)));
                return OptimisationResult::Applied;
            } else if let Operation::Disjunction = view.current_operation() {
                view.replace_operations(2, view.operations_by_range(other_operand).into_iter());
                return OptimisationResult::Applied;
            } else if let Operation::IfAndOnlyIf = view.current_operation() {
                view.replace_operations_and_reprocess(
                    2,
                    view.operations_by_range(other_operand)
                        .into_iter()
                        .chain(once(Operation::NegateBool)),
                );
                return OptimisationResult::Applied;
            } else if let Operation::EqualsBool = view.current_operation() {
                view.replace_operations_and_reprocess(
                    2,
                    view.operations_by_range(other_operand)
                        .into_iter()
                        .chain(once(Operation::NegateBool)),
                );
                return OptimisationResult::Applied;
            } else if let Operation::NotEqualsBool = view.current_operation() {
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
    use super::OneOperandFalseOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    macro_rules! test_one_operand_false {
        ($name:ident, [$($other_ops:expr),*], $operator:expr, [$($result:expr),*]) => {
            paste::paste! {
                test_optimisation!(
                    [<$name >],
                    OneOperandFalseOptimisations,
                    [PushBool(false), PushBool(false), $($other_ops),*, $operator],
                    [PushBool(false), $($result),*]
                );
                test_optimisation!(
                    [<$name _switched>],
                    OneOperandFalseOptimisations,
                    [PushBool(false), $($other_ops),*, PushBool(false), $operator],
                    [PushBool(false), $($result),*]
                );
                test_optimisation!(
                    [<$name _not_applicable>],
                    OneOperandFalseOptimisations,
                    [PushBool(false), PushBool(false), NegateBool, $($other_ops),*, $operator],
                    [PushBool(false), PushBool(false), NegateBool, $($other_ops),*, $operator]
                );
                test_optimisation!(
                    [<$name _switched_not_applicable>],
                    OneOperandFalseOptimisations,
                    [PushBool(false), $($other_ops),*, PushBool(false), NegateBool, $operator],
                    [PushBool(false), $($other_ops),*, PushBool(false), NegateBool, $operator]
                );
            }
        };
    }

    test_one_operand_false!(
        conjunction,
        [PushBool(true), NegateBool],
        Conjunction,
        [PushBool(false)]
    );
    test_one_operand_false!(
        disjunction,
        [PushBool(true), NegateBool],
        Disjunction,
        [PushBool(true), NegateBool]
    );
    test_one_operand_false!(
        if_and_only_if,
        [PushBool(true), NegateBool],
        IfAndOnlyIf,
        [PushBool(true), NegateBool, NegateBool]
    );
    test_one_operand_false!(
        equals,
        [PushBool(true), NegateBool],
        EqualsBool,
        [PushBool(true), NegateBool, NegateBool]
    );
    test_one_operand_false!(
        not_equals,
        [PushBool(true), NegateBool],
        NotEqualsBool,
        [PushBool(true), NegateBool]
    );
}
