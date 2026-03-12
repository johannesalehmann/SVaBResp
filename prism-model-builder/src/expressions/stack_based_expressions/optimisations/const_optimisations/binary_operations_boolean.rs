use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use crate::expressions::stack_based_expressions::Operation::PushBool;
use std::iter::once;

pub struct BinaryBooleanOptimisations {}
impl Optimisation for BinaryBooleanOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Some(PushBool(b1)) = view.single_operation_from_stack(1) {
            if let Some(PushBool(b2)) = view.single_operation_from_stack(0) {
                let b1 = *b1;
                let b2 = *b2;
                if let Operation::Conjunction = view.current_operation() {
                    view.replace_operations(2, once(PushBool(b1 && b2)));
                    return OptimisationResult::Applied;
                } else if let Operation::Disjunction = view.current_operation() {
                    view.replace_operations(2, once(PushBool(b1 || b2)));
                    return OptimisationResult::Applied;
                } else if let Operation::IfAndOnlyIf = view.current_operation() {
                    view.replace_operations(2, once(PushBool(b1 == b2)));
                    return OptimisationResult::Applied;
                } else if let Operation::Implies = view.current_operation() {
                    view.replace_operations(2, once(PushBool(!b1 || b2)));
                    return OptimisationResult::Applied;
                } else if let Operation::EqualsBool = view.current_operation() {
                    view.replace_operations(2, once(PushBool(b1 == b2)));
                    return OptimisationResult::Applied;
                } else if let Operation::NotEqualsBool = view.current_operation() {
                    view.replace_operations(2, once(PushBool(b1 != b2)));
                    return OptimisationResult::Applied;
                }
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::BinaryBooleanOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    macro_rules! test_binary_boolean {
        ($name:ident, $op1:expr, $op2:expr, $conj:expr, $disj:expr, $iff:expr, $implies:expr, $eq:expr, $neq:expr) => {
            paste::paste! {
                test_optimisation!(
                    $name,
                    BinaryBooleanOptimisations,
                    [PushInt(0), PushBool($op1), PushBool($op2), Conjunction],
                    [PushInt(0), PushBool($conj)]
                );
                test_optimisation!(
                    [<$name _not_applicable_1>],
                    BinaryBooleanOptimisations,
                    [PushInt(0), PushBool($op1), NegateBool, PushBool($op2), Conjunction],
                    [PushInt(0), PushBool($op1), NegateBool, PushBool($op2), Conjunction]
                );
                test_optimisation!(
                    [<$name _not_applicable_2>],
                    BinaryBooleanOptimisations,
                    [PushInt(0), PushBool($op1), PushBool($op2), NegateBool, Conjunction],
                    [PushInt(0), PushBool($op1), PushBool($op2), NegateBool, Conjunction]
                );
            }
        };
    }

    test_binary_boolean!(
        false_false,
        false,
        false,
        false,
        false,
        true,
        true,
        true,
        false
    );
    test_binary_boolean!(
        false_true, false, true, false, true, false, true, false, true
    );
    test_binary_boolean!(
        true_false, true, false, false, true, false, false, false, true
    );
    test_binary_boolean!(true_true, true, true, true, true, true, true, true, false);
}
