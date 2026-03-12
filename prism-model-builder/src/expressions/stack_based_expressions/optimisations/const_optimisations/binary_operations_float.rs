use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use crate::expressions::stack_based_expressions::Operation::{PushBool, PushFloat};
use std::iter::once;

pub struct BinaryFloatOptimisations {}
impl Optimisation for BinaryFloatOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Some(PushFloat(i1)) = view.single_operation_from_stack(1) {
            if let Some(PushFloat(i2)) = view.single_operation_from_stack(0) {
                if let Operation::MultiplyFloat = view.current_operation() {
                    view.replace_operations(2, once(PushFloat(i1 * i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::DivideFloat = view.current_operation() {
                    view.replace_operations(2, once(PushFloat(i1 / i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::AddFloat = view.current_operation() {
                    view.replace_operations(2, once(PushFloat(i1 + i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::SubtractFloat = view.current_operation() {
                    view.replace_operations(2, once(PushFloat(i1 - i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::LessThanFloat = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 < i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::LessOrEqualFloat = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 <= i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::GreaterThanFloat = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 > i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::GreaterOrEqualFloat = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 >= i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::EqualsFloat = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 == i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::NotEqualsFloat = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 != i2)));
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
    use super::BinaryFloatOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        multiply,
        BinaryFloatOptimisations,
        [PushInt(17), PushFloat(5.0), PushFloat(3.0), MultiplyFloat],
        [PushInt(17), PushFloat(15.0)]
    );
    test_optimisation!(
        divide,
        BinaryFloatOptimisations,
        [PushInt(17), PushFloat(15.0), PushFloat(5.0), DivideFloat],
        [PushInt(17), PushFloat(3.0)]
    );
    test_optimisation!(
        add,
        BinaryFloatOptimisations,
        [PushInt(17), PushFloat(15.0), PushFloat(5.0), AddFloat],
        [PushInt(17), PushFloat(20.0)]
    );
    test_optimisation!(
        subtract,
        BinaryFloatOptimisations,
        [PushInt(17), PushFloat(20.0), PushFloat(5.0), SubtractFloat],
        [PushInt(17), PushFloat(15.0)]
    );

    macro_rules! test_comparison {
        ($name:ident, $op:expr, $val_lt:expr, $val_eq:expr, $val_gt:expr) => {
            paste::paste! {
                test_optimisation!(
                    [<$name _lt>],
                    BinaryFloatOptimisations,
                    [PushInt(17), PushFloat(15.0), PushFloat(20.0), $op],
                    [PushInt(17), PushBool($val_lt)]
                );

                test_optimisation!(
                    [<$name _eq>],
                    BinaryFloatOptimisations,
                    [PushInt(17), PushFloat(15.0), PushFloat(15.0), $op],
                    [PushInt(17), PushBool($val_eq)]
                );

                test_optimisation!(
                    [<$name _gt>],
                    BinaryFloatOptimisations,
                    [PushInt(17), PushFloat(20.0), PushFloat(15.0), $op],
                    [PushInt(17), PushBool($val_gt)]
                );

                test_optimisation!(
                    [<$name _not_applicable_first>],
                    BinaryFloatOptimisations,
                    [PushInt(17), PushFloat(5.0), NegateFloat, PushFloat(15.0), $op],
                    [PushInt(17), PushFloat(5.0), NegateFloat, PushFloat(15.0), $op]
                );

                test_optimisation!(
                    [<$name _not_applicable_second>],
                    BinaryFloatOptimisations,
                    [PushInt(17), PushFloat(5.0), PushFloat(15.0), NegateFloat, $op],
                    [PushInt(17), PushFloat(5.0), PushFloat(15.0), NegateFloat, $op]
                );
            }
        };
    }

    test_comparison!(less_than, LessThanFloat, true, false, false);
    test_comparison!(less_or_equals, LessOrEqualFloat, true, true, false);
    test_comparison!(greater_than, GreaterThanFloat, false, false, true);
    test_comparison!(greater_or_equals, GreaterOrEqualFloat, false, true, true);
    test_comparison!(equals, EqualsFloat, false, true, false);
    test_comparison!(not_equals, NotEqualsFloat, true, false, true);
}
