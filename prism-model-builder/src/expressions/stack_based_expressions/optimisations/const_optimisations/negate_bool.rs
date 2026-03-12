use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use prism_model::VariableReference;
use std::iter;

pub struct NegateBoolOptimisation {}

impl Optimisation for NegateBoolOptimisation {
    fn apply(&self, view: &mut OperationView<VariableReference>) -> OptimisationResult {
        if let Operation::NegateBool = view.current_operation() {
            if let Some(Operation::PushBool(b)) = view.single_operation_from_stack(0) {
                view.replace_operations(1, iter::once(Operation::PushBool(!b)));
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::NegateBoolOptimisation;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        true_complex,
        NegateBoolOptimisation,
        [PushBool(true), PushBool(false), NegateBool],
        [PushBool(true), PushBool(true)]
    );
    test_optimisation!(
        false_complex,
        NegateBoolOptimisation,
        [PushBool(true), PushBool(true), NegateBool],
        [PushBool(true), PushBool(false)]
    );
    test_optimisation!(
        nested,
        NegateBoolOptimisation,
        [PushBool(true), PushBool(true), NegateBool, NegateBool],
        [PushBool(true), PushBool(true)]
    );
    test_optimisation!(
        not_applicable,
        NegateBoolOptimisation,
        [
            PushBool(true),
            PushBool(true),
            PushBool(false),
            Conjunction,
            NegateBool
        ],
        [
            PushBool(true),
            PushBool(true),
            PushBool(false),
            Conjunction,
            NegateBool
        ]
    );
}
