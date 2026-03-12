use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter::once;

pub struct ImpliesOptimisations {}
impl Optimisation for ImpliesOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Operation::Implies = view.current_operation() {
            if let Some(Operation::PushBool(true)) = view.single_operation_from_stack(1) {
                view.replace_operations(
                    2,
                    view.operations_by_range(view.operand_range(0)).into_iter(),
                );
                return OptimisationResult::Applied;
            } else if let Some(Operation::PushBool(false)) = view.single_operation_from_stack(1) {
                view.replace_operations(2, once(Operation::PushBool(true)));
                return OptimisationResult::Applied;
            } else if let Some(Operation::PushBool(true)) = view.single_operation_from_stack(0) {
                view.replace_operations(2, once(Operation::PushBool(true)));
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::ImpliesOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        true_implicant,
        ImpliesOptimisations,
        [
            PushInt(0),
            PushBool(true),
            PushBool(false),
            PushBool(true),
            Conjunction,
            Implies
        ],
        [PushInt(0), PushBool(false), PushBool(true), Conjunction]
    );
    test_optimisation!(
        false_implicant,
        ImpliesOptimisations,
        [
            PushInt(0),
            PushBool(false),
            PushBool(false),
            PushBool(true),
            Conjunction,
            Implies
        ],
        [PushInt(0), PushBool(true)]
    );
    test_optimisation!(
        true_implication,
        ImpliesOptimisations,
        [
            PushInt(0),
            PushBool(false),
            PushBool(true),
            Conjunction,
            PushBool(true),
            Implies
        ],
        [PushInt(0), PushBool(true)]
    );
    test_optimisation!(
        not_applicable_1,
        ImpliesOptimisations,
        [
            PushInt(0),
            PushBool(false),
            PushBool(true),
            Conjunction,
            PushBool(false),
            Implies
        ],
        [
            PushInt(0),
            PushBool(false),
            PushBool(true),
            Conjunction,
            PushBool(false),
            Implies
        ]
    );
}
