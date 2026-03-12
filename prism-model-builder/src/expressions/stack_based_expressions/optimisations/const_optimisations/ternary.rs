use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};

pub struct TernaryOptimisation {}
impl Optimisation for TernaryOptimisation {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        let is_ternary = match view.current_operation() {
            Operation::TernaryInt => true,
            Operation::TernaryFloat => true,
            Operation::TernaryBool => true,
            _ => false,
        };
        if is_ternary {
            if let Some(Operation::PushBool(true)) = view.single_operation_from_stack(2) {
                view.replace_operations(
                    3,
                    view.operations_by_range(view.operand_range(1)).into_iter(),
                );
                return OptimisationResult::Applied;
            } else if let Some(Operation::PushBool(false)) = view.single_operation_from_stack(2) {
                view.replace_operations(
                    3,
                    view.operations_by_range(view.operand_range(0)).into_iter(),
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
    use super::TernaryOptimisation;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        guard_true,
        TernaryOptimisation,
        [
            PushBool(true),
            PushBool(true),
            PushInt(3),
            PushInt(2),
            AddInt,
            PushFloat(3.1),
            Round,
            TernaryInt
        ],
        [PushBool(true), PushInt(3), PushInt(2), AddInt]
    );
    test_optimisation!(
        guard_false,
        TernaryOptimisation,
        [
            PushBool(false),
            PushBool(false),
            PushInt(3),
            PushInt(2),
            AddInt,
            PushFloat(3.1),
            Round,
            TernaryInt
        ],
        [PushBool(false), PushFloat(3.1), Round]
    );
    test_optimisation!(
        not_applicable,
        TernaryOptimisation,
        [
            PushBool(false),
            PushBool(true),
            NegateBool,
            PushInt(3),
            PushInt(2),
            AddInt,
            PushFloat(3.1),
            Round,
            TernaryInt
        ],
        [
            PushBool(false),
            PushBool(true),
            NegateBool,
            PushInt(3),
            PushInt(2),
            AddInt,
            PushFloat(3.1),
            Round,
            TernaryInt
        ]
    );
}
