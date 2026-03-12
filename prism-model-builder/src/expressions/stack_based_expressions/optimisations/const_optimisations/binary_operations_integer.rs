use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use crate::expressions::stack_based_expressions::Operation::{PushBool, PushFloat, PushInt};
use std::iter::once;

pub struct BinaryIntegerOptimisations {}
impl Optimisation for BinaryIntegerOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Some(PushInt(i1)) = view.single_operation_from_stack(1) {
            if let Some(PushInt(i2)) = view.single_operation_from_stack(0) {
                if let Operation::MultiplyInt = view.current_operation() {
                    view.replace_operations(2, once(PushInt(i1 * i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::DivideInt = view.current_operation() {
                    view.replace_operations(2, once(PushFloat(*i1 as f64 / *i2 as f64)));
                    return OptimisationResult::Applied;
                } else if let Operation::AddInt = view.current_operation() {
                    view.replace_operations(2, once(PushInt(i1 + i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::SubtractInt = view.current_operation() {
                    view.replace_operations(2, once(PushInt(i1 - i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::LessThanInt = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 < i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::LessOrEqualInt = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 <= i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::GreaterThanInt = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 > i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::GreaterOrEqualInt = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 >= i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::EqualsInt = view.current_operation() {
                    view.replace_operations(2, once(PushBool(i1 == i2)));
                    return OptimisationResult::Applied;
                } else if let Operation::NotEqualsInt = view.current_operation() {
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
    use super::BinaryIntegerOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        multiply,
        BinaryIntegerOptimisations,
        [PushInt(17), PushInt(5), PushInt(3), MultiplyInt],
        [PushInt(17), PushInt(15)]
    );
    test_optimisation!(
        divide,
        BinaryIntegerOptimisations,
        [PushInt(17), PushInt(15), PushInt(5), DivideInt],
        [PushInt(17), PushFloat(3.0)]
    );
    test_optimisation!(
        add,
        BinaryIntegerOptimisations,
        [PushInt(17), PushInt(15), PushInt(5), AddInt],
        [PushInt(17), PushInt(20)]
    );
    test_optimisation!(
        subtract,
        BinaryIntegerOptimisations,
        [PushInt(17), PushInt(20), PushInt(5), SubtractInt],
        [PushInt(17), PushInt(15)]
    );

    macro_rules! test_comparison {
        ($name:ident, $op:expr, $val_lt:expr, $val_eq:expr, $val_gt:expr) => {
            paste::paste! {
                test_optimisation!(
                    [<$name _lt>],
                    BinaryIntegerOptimisations,
                    [PushInt(17), PushInt(15), PushInt(20), $op],
                    [PushInt(17), PushBool($val_lt)]
                );

                test_optimisation!(
                    [<$name _eq>],
                    BinaryIntegerOptimisations,
                    [PushInt(17), PushInt(15), PushInt(15), $op],
                    [PushInt(17), PushBool($val_eq)]
                );

                test_optimisation!(
                    [<$name _gt>],
                    BinaryIntegerOptimisations,
                    [PushInt(17), PushInt(20), PushInt(15), $op],
                    [PushInt(17), PushBool($val_gt)]
                );

                test_optimisation!(
                    [<$name _not_applicable_first>],
                    BinaryIntegerOptimisations,
                    [PushInt(17), PushInt(5), NegateInt, PushInt(15), $op],
                    [PushInt(17), PushInt(5), NegateInt, PushInt(15), $op]
                );

                test_optimisation!(
                    [<$name _not_applicable_second>],
                    BinaryIntegerOptimisations,
                    [PushInt(17), PushInt(5), PushInt(15), NegateInt, $op],
                    [PushInt(17), PushInt(5), PushInt(15), NegateInt, $op]
                );
            }
        };
    }

    test_comparison!(less_than, LessThanInt, true, false, false);
    test_comparison!(less_or_equals, LessOrEqualInt, true, true, false);
    test_comparison!(greater_than, GreaterThanInt, false, false, true);
    test_comparison!(greater_or_equals, GreaterOrEqualInt, false, true, true);
    test_comparison!(equals, EqualsInt, false, true, false);
    test_comparison!(not_equals, NotEqualsInt, true, false, true);
}
