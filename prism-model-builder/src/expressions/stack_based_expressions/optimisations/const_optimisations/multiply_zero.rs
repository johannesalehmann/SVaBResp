use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter;

pub struct MultiplyByZeroOptimisation {}

impl Optimisation for MultiplyByZeroOptimisation {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Operation::MultiplyInt = view.current_operation() {
            if let Some(Operation::PushInt(0)) = view.single_operation_from_stack(0) {
                view.replace_operations(2, iter::once(Operation::PushInt(0)));
                return OptimisationResult::Applied;
            } else if let Some(Operation::PushInt(0)) = view.single_operation_from_stack(1) {
                view.replace_operations(2, iter::once(Operation::PushInt(0)));
                return OptimisationResult::Applied;
            }
        } else if let Operation::MultiplyFloat = view.current_operation() {
            if let Some(Operation::PushFloat(0.0)) = view.single_operation_from_stack(0) {
                view.replace_operations(2, iter::once(Operation::PushFloat(0.0)));
                return OptimisationResult::Applied;
            } else if let Some(Operation::PushFloat(0.0)) = view.single_operation_from_stack(1) {
                view.replace_operations(2, iter::once(Operation::PushFloat(0.0)));
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::MultiplyByZeroOptimisation;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        integer_zero_first,
        MultiplyByZeroOptimisation,
        [PushInt(0), PushInt(3), MultiplyInt],
        [PushInt(0)]
    );
    test_optimisation!(
        integer_zero_second,
        MultiplyByZeroOptimisation,
        [PushInt(3), PushInt(0), MultiplyInt],
        [PushInt(0)]
    );
    test_optimisation!(
        float_zero_first,
        MultiplyByZeroOptimisation,
        [PushFloat(0.0), PushFloat(3.0), MultiplyFloat],
        [PushFloat(0.0)]
    );
    test_optimisation!(
        float_zero_second,
        MultiplyByZeroOptimisation,
        [PushFloat(3.0), PushFloat(0.0), MultiplyFloat],
        [PushFloat(0.0)]
    );

    test_optimisation!(
        integer_zero_first_complex,
        MultiplyByZeroOptimisation,
        [
            PushInt(-5),
            PushInt(0),
            PushInt(3),
            PushInt(5),
            AddInt,
            MultiplyInt
        ],
        [PushInt(-5), PushInt(0)]
    );

    test_optimisation!(
        integer_zero_second_complex,
        MultiplyByZeroOptimisation,
        [
            PushInt(-5),
            PushInt(3),
            PushInt(5),
            AddInt,
            PushInt(0),
            MultiplyInt
        ],
        [PushInt(-5), PushInt(0)]
    );
    test_optimisation!(
        float_zero_first_complex,
        MultiplyByZeroOptimisation,
        [
            PushInt(-5),
            PushFloat(0.0),
            PushFloat(3.0),
            PushFloat(5.0),
            AddInt,
            MultiplyFloat
        ],
        [PushInt(-5), PushFloat(0.0)]
    );

    test_optimisation!(
        float_zero_second_complex,
        MultiplyByZeroOptimisation,
        [
            PushInt(-5),
            PushFloat(3.0),
            PushFloat(5.0),
            AddInt,
            PushFloat(0.0),
            MultiplyFloat
        ],
        [PushInt(-5), PushFloat(0.0)]
    );

    test_optimisation!(
        int_zero_both_complex,
        MultiplyByZeroOptimisation,
        [PushInt(-5), PushInt(0), PushInt(0), MultiplyInt],
        [PushInt(-5), PushInt(0)]
    );
    test_optimisation!(
        float_zero_both_complex,
        MultiplyByZeroOptimisation,
        [PushInt(-5), PushFloat(0.0), PushFloat(0.0), MultiplyFloat],
        [PushInt(-5), PushFloat(0.0)]
    );

    test_optimisation!(
        int_not_applicable,
        MultiplyByZeroOptimisation,
        [PushInt(-5), PushInt(2), PushInt(-2), MultiplyInt],
        [PushInt(-5), PushInt(2), PushInt(-2), MultiplyInt]
    );

    test_optimisation!(
        float_not_applicable,
        MultiplyByZeroOptimisation,
        [PushInt(-5), PushFloat(2.0), PushFloat(-2.0), MultiplyFloat],
        [PushInt(-5), PushFloat(2.0), PushFloat(-2.0), MultiplyFloat]
    );
}
