use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use std::iter::once;

pub struct FunctionOptimisations {}

impl FunctionOptimisations {
    fn extract_int(op: &Operation<prism_model::VariableReference>) -> Option<i64> {
        match op {
            Operation::PushInt(val) => Some(*val),
            _ => None,
        }
    }

    fn extract_float(op: &Operation<prism_model::VariableReference>) -> Option<f64> {
        match op {
            Operation::PushFloat(val) => Some(*val),
            _ => None,
        }
    }

    fn fold_n_parameters<
        T,
        E: Fn(&Operation<prism_model::VariableReference>) -> Option<T>,
        F: Fn(T, T) -> T,
        R: Fn(T) -> Operation<prism_model::VariableReference>,
    >(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
        n: usize,
        initial: T,
        extract: E,
        fold: F,
        replacement: R,
    ) -> OptimisationResult {
        let mut accumulator = initial;
        let mut all_constant = true;
        for i in 0..n {
            if let Some(operation) = view.single_operation_from_stack(i) {
                if let Some(val) = extract(operation) {
                    accumulator = fold(accumulator, val);
                } else {
                    all_constant = false;
                    break;
                }
            } else {
                all_constant = false;
                break;
            }
        }
        if all_constant {
            view.replace_operations(n, once(replacement(accumulator)));
            return OptimisationResult::Applied;
        }
        OptimisationResult::NotApplied
    }
}
impl Optimisation for FunctionOptimisations {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Operation::MinInt(n) = view.current_operation() {
            if self.fold_n_parameters(
                view,
                *n,
                i64::MAX,
                Self::extract_int,
                |x, y| x.min(y),
                |min| Operation::PushInt(min),
            ) == OptimisationResult::Applied
            {
                return OptimisationResult::Applied;
            }
        } else if let Operation::MinFloat(n) = view.current_operation() {
            if self.fold_n_parameters(
                view,
                *n,
                f64::MAX,
                Self::extract_float,
                |x, y| x.min(y),
                |min| Operation::PushFloat(min),
            ) == OptimisationResult::Applied
            {
                return OptimisationResult::Applied;
            }
        } else if let Operation::MaxInt(n) = view.current_operation() {
            if self.fold_n_parameters(
                view,
                *n,
                i64::MIN,
                Self::extract_int,
                |x, y| x.max(y),
                |max| Operation::PushInt(max),
            ) == OptimisationResult::Applied
            {
                return OptimisationResult::Applied;
            }
        } else if let Operation::MaxFloat(n) = view.current_operation() {
            if self.fold_n_parameters(
                view,
                *n,
                f64::MIN,
                Self::extract_float,
                |x, y| x.max(y),
                |max| Operation::PushFloat(max),
            ) == OptimisationResult::Applied
            {
                return OptimisationResult::Applied;
            }
        } else if let Operation::Floor = view.current_operation() {
            if let Some(Operation::PushFloat(val)) = view.single_operation_from_stack(0) {
                view.replace_operations(1, once(Operation::PushInt(val.floor() as i64)));
                return OptimisationResult::Applied;
            }
        } else if let Operation::Ceil = view.current_operation() {
            if let Some(Operation::PushFloat(val)) = view.single_operation_from_stack(0) {
                view.replace_operations(1, once(Operation::PushInt(val.ceil() as i64)));
                return OptimisationResult::Applied;
            }
        } else if let Operation::Round = view.current_operation() {
            if let Some(Operation::PushFloat(val)) = view.single_operation_from_stack(0) {
                let rounded = if val.fract() == -0.5 {
                    val.ceil()
                } else {
                    val.round()
                };
                view.replace_operations(1, once(Operation::PushInt(rounded as i64)));
                return OptimisationResult::Applied;
            }
        } else if let Operation::PowInt = view.current_operation() {
            if let Some(Operation::PushInt(i1)) = view.single_operation_from_stack(1) {
                if let Some(Operation::PushInt(i2)) = view.single_operation_from_stack(0) {
                    view.replace_operations(2, once(Operation::PushInt(i1.pow(*i2 as u32))));
                    return OptimisationResult::Applied;
                }
            }
        } else if let Operation::PowFloat = view.current_operation() {
            if let Some(Operation::PushFloat(f1)) = view.single_operation_from_stack(1) {
                if let Some(Operation::PushFloat(f2)) = view.single_operation_from_stack(0) {
                    view.replace_operations(2, once(Operation::PushFloat(f1.powf(*f2))));
                    return OptimisationResult::Applied;
                }
            }
        } else if let Operation::Mod = view.current_operation() {
            if let Some(Operation::PushInt(i1)) = view.single_operation_from_stack(1) {
                if let Some(Operation::PushInt(i2)) = view.single_operation_from_stack(0) {
                    view.replace_operations(2, once(Operation::PushInt(i1.rem_euclid(*i2))));
                    return OptimisationResult::Applied;
                }
            }
        } else if let Operation::LogFloat = view.current_operation() {
            if let Some(Operation::PushFloat(f1)) = view.single_operation_from_stack(1) {
                if let Some(Operation::PushFloat(f2)) = view.single_operation_from_stack(0) {
                    view.replace_operations(2, once(Operation::PushFloat(f1.log(*f2))));
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
    use super::FunctionOptimisations;
    use crate::expressions::stack_based_expressions::Operation::*;

    test_optimisation!(
        min_int_two_values_1,
        FunctionOptimisations,
        [PushInt(11), PushInt(18), PushInt(21), MinInt(2)],
        [PushInt(11), PushInt(18)]
    );
    test_optimisation!(
        min_int_two_values_2,
        FunctionOptimisations,
        [PushInt(-20), PushInt(-5), PushInt(-7), MinInt(2)],
        [PushInt(-20), PushInt(-7)]
    );
    test_optimisation!(
        min_int_two_values_not_applicable_1,
        FunctionOptimisations,
        [PushInt(-20), PushInt(-5), NegateInt, PushInt(-7), MinInt(2)],
        [PushInt(-20), PushInt(-5), NegateInt, PushInt(-7), MinInt(2)]
    );
    test_optimisation!(
        min_int_two_values_not_applicable_2,
        FunctionOptimisations,
        [PushInt(-20), PushInt(-5), PushInt(-7), NegateInt, MinInt(2)],
        [PushInt(-20), PushInt(-5), PushInt(-7), NegateInt, MinInt(2)]
    );

    test_optimisation!(
        min_int_five_values_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushInt(-5),
            PushInt(-4),
            PushInt(-7),
            PushInt(-7),
            PushInt(3),
            MinInt(5)
        ],
        [PushInt(-20), PushInt(-7)]
    );
    test_optimisation!(
        min_int_five_values_2,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushInt(-5),
            PushInt(-4),
            PushInt(-7),
            PushInt(-7),
            PushInt(-21),
            MinInt(5)
        ],
        [PushInt(-20), PushInt(-21)]
    );
    test_optimisation!(
        min_int_five_values_not_applicable,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushInt(-5),
            PushInt(-4),
            NegateInt,
            PushInt(-7),
            PushInt(-7),
            PushInt(-21),
            MinInt(5)
        ],
        [
            PushInt(-20),
            PushInt(-5),
            PushInt(-4),
            NegateInt,
            PushInt(-7),
            PushInt(-7),
            PushInt(-21),
            MinInt(5)
        ]
    );

    test_optimisation!(
        min_float_two_values_1,
        FunctionOptimisations,
        [PushInt(11), PushFloat(18.2), PushFloat(21.6), MinFloat(2)],
        [PushInt(11), PushFloat(18.2)]
    );
    test_optimisation!(
        min_float_two_values_2,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.9), PushFloat(-5.1), MinFloat(2)],
        [PushInt(-20), PushFloat(-5.1)]
    );
    test_optimisation!(
        min_float_two_values_not_applicable_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-4.9),
            NegateFloat,
            PushFloat(-5.1),
            MinFloat(2)
        ],
        [
            PushInt(-20),
            PushFloat(-4.9),
            NegateFloat,
            PushFloat(-5.1),
            MinFloat(2)
        ]
    );
    test_optimisation!(
        min_float_two_values_not_applicable_2,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-4.9),
            PushFloat(-5.1),
            NegateFloat,
            MinFloat(2)
        ],
        [
            PushInt(-20),
            PushFloat(-4.9),
            PushFloat(-5.1),
            NegateFloat,
            MinFloat(2)
        ]
    );

    test_optimisation!(
        min_float_five_values_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-5.1),
            PushFloat(-4.8),
            PushFloat(-7.1),
            PushFloat(-7.1),
            PushFloat(3.14),
            MinFloat(5)
        ],
        [PushInt(-20), PushFloat(-7.1)]
    );
    test_optimisation!(
        min_float_five_values_2,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-5.1),
            PushFloat(-4.8),
            PushFloat(-7.1),
            PushFloat(-7.1),
            PushFloat(-21.0),
            MinFloat(5)
        ],
        [PushInt(-20), PushFloat(-21.0)]
    );
    test_optimisation!(
        min_float_five_values_not_applicable,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-5.1),
            PushFloat(-4.8),
            NegateFloat,
            PushFloat(-7.1),
            PushFloat(-7.1),
            PushFloat(-21.0),
            MinFloat(5)
        ],
        [
            PushInt(-20),
            PushFloat(-5.1),
            PushFloat(-4.8),
            NegateFloat,
            PushFloat(-7.1),
            PushFloat(-7.1),
            PushFloat(-21.0),
            MinFloat(5)
        ]
    );

    test_optimisation!(
        max_int_two_values_1,
        FunctionOptimisations,
        [PushInt(11), PushInt(18), PushInt(21), MaxInt(2)],
        [PushInt(11), PushInt(21)]
    );
    test_optimisation!(
        max_int_two_values_2,
        FunctionOptimisations,
        [PushInt(-20), PushInt(-5), PushInt(-7), MaxInt(2)],
        [PushInt(-20), PushInt(-5)]
    );
    test_optimisation!(
        max_int_two_values_not_applicable_1,
        FunctionOptimisations,
        [PushInt(-20), PushInt(-5), NegateInt, PushInt(-7), MaxInt(2)],
        [PushInt(-20), PushInt(-5), NegateInt, PushInt(-7), MaxInt(2)]
    );
    test_optimisation!(
        max_int_two_values_not_applicable_2,
        FunctionOptimisations,
        [PushInt(-20), PushInt(-5), PushInt(-7), NegateInt, MaxInt(2)],
        [PushInt(-20), PushInt(-5), PushInt(-7), NegateInt, MaxInt(2)]
    );

    test_optimisation!(
        max_int_five_values_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushInt(-5),
            PushInt(-4),
            PushInt(7),
            PushInt(4),
            PushInt(3),
            MaxInt(5)
        ],
        [PushInt(-20), PushInt(7)]
    );
    test_optimisation!(
        max_int_five_values_2,
        FunctionOptimisations,
        [
            PushInt(0),
            PushInt(-5),
            PushInt(-4),
            PushInt(-7),
            PushInt(-7),
            PushInt(-21),
            MaxInt(5)
        ],
        [PushInt(0), PushInt(-4)]
    );
    test_optimisation!(
        max_int_five_values_not_applicable,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushInt(-5),
            PushInt(-4),
            NegateInt,
            PushInt(-7),
            PushInt(-7),
            PushInt(-21),
            MaxInt(5)
        ],
        [
            PushInt(-20),
            PushInt(-5),
            PushInt(-4),
            NegateInt,
            PushInt(-7),
            PushInt(-7),
            PushInt(-21),
            MaxInt(5)
        ]
    );

    test_optimisation!(
        max_float_two_values_1,
        FunctionOptimisations,
        [PushInt(11), PushFloat(18.2), PushFloat(21.6), MaxFloat(2)],
        [PushInt(11), PushFloat(21.6)]
    );
    test_optimisation!(
        max_float_two_values_2,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.9), PushFloat(-5.1), MaxFloat(2)],
        [PushInt(-20), PushFloat(-4.9)]
    );
    test_optimisation!(
        max_float_two_values_not_applicable_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-4.9),
            NegateFloat,
            PushFloat(-5.1),
            MaxFloat(2)
        ],
        [
            PushInt(-20),
            PushFloat(-4.9),
            NegateFloat,
            PushFloat(-5.1),
            MaxFloat(2)
        ]
    );
    test_optimisation!(
        max_float_two_values_not_applicable_2,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-4.9),
            PushFloat(-5.1),
            NegateFloat,
            MaxFloat(2)
        ],
        [
            PushInt(-20),
            PushFloat(-4.9),
            PushFloat(-5.1),
            NegateFloat,
            MaxFloat(2)
        ]
    );

    test_optimisation!(
        max_float_five_values_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(5.1),
            PushFloat(-4.8),
            PushFloat(-7.1),
            PushFloat(-7.1),
            PushFloat(3.14),
            MaxFloat(5)
        ],
        [PushInt(-20), PushFloat(5.1)]
    );
    test_optimisation!(
        max_float_five_values_2,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-5.1),
            PushFloat(44.8),
            PushFloat(-7.1),
            PushFloat(7.1),
            PushFloat(21.0),
            MaxFloat(5)
        ],
        [PushInt(-20), PushFloat(44.8)]
    );
    test_optimisation!(
        max_float_five_values_not_applicable,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(-5.1),
            PushFloat(-4.8),
            NegateFloat,
            PushFloat(-7.1),
            PushFloat(-7.1),
            PushFloat(-21.0),
            MaxFloat(5)
        ],
        [
            PushInt(-20),
            PushFloat(-5.1),
            PushFloat(-4.8),
            NegateFloat,
            PushFloat(-7.1),
            PushFloat(-7.1),
            PushFloat(-21.0),
            MaxFloat(5)
        ]
    );

    test_optimisation!(
        floor_positive,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(4.9), Floor],
        [PushInt(-20), PushInt(4)]
    );
    test_optimisation!(
        floor_integer,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(17.0), Floor],
        [PushInt(-20), PushInt(17)]
    );
    test_optimisation!(
        floor_negative,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.3), Floor],
        [PushInt(-20), PushInt(-5)]
    );
    test_optimisation!(
        floor_not_applicable,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.3), NegateFloat, Floor],
        [PushInt(-20), PushFloat(-4.3), NegateFloat, Floor]
    );

    test_optimisation!(
        ceil_positive,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(4.2), Ceil],
        [PushInt(-20), PushInt(5)]
    );
    test_optimisation!(
        ceil_integer,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(17.0), Ceil],
        [PushInt(-20), PushInt(17)]
    );
    test_optimisation!(
        ceil_negative,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.9), Ceil],
        [PushInt(-20), PushInt(-4)]
    );
    test_optimisation!(
        ceil_not_applicable,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.3), NegateFloat, Ceil],
        [PushInt(-20), PushFloat(-4.3), NegateFloat, Ceil]
    );

    test_optimisation!(
        round_down_positive,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(4.2), Round],
        [PushInt(-20), PushInt(4)]
    );
    test_optimisation!(
        round_middle_positive,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(4.5), Round],
        [PushInt(-20), PushInt(5)]
    );
    test_optimisation!(
        round_up_positive,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(4.8), Round],
        [PushInt(-20), PushInt(5)]
    );
    test_optimisation!(
        round_zero,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(0.0), Round],
        [PushInt(-20), PushInt(0)]
    );
    test_optimisation!(
        round_down_negative,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.2), Round],
        [PushInt(-20), PushInt(-4)]
    );
    test_optimisation!(
        round_middle_negative,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.5), Round],
        [PushInt(-20), PushInt(-4)]
    );
    test_optimisation!(
        round_up_negative,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.8), Round],
        [PushInt(-20), PushInt(-5)]
    );
    test_optimisation!(
        round_not_applicable,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(-4.3), NegateFloat, Round],
        [PushInt(-20), PushFloat(-4.3), NegateFloat, Round]
    );

    test_optimisation!(
        pow_int,
        FunctionOptimisations,
        [PushInt(-20), PushInt(2), PushInt(5), PowInt],
        [PushInt(-20), PushInt(32)]
    );
    test_optimisation!(
        pow_int_not_applicable_1,
        FunctionOptimisations,
        [PushInt(-20), PushInt(2), NegateInt, PushInt(5), PowInt],
        [PushInt(-20), PushInt(2), NegateInt, PushInt(5), PowInt]
    );
    test_optimisation!(
        pow_int_not_applicable_2,
        FunctionOptimisations,
        [PushInt(-20), PushInt(2), PushInt(5), NegateInt, PowInt],
        [PushInt(-20), PushInt(2), PushInt(5), NegateInt, PowInt]
    );

    test_optimisation!(
        pow_float,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(0.5), PushFloat(3.0), PowFloat],
        [PushInt(-20), PushFloat(0.125)]
    );
    test_optimisation!(
        pow_float_not_applicable_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(0.5),
            NegateFloat,
            PushFloat(3.0),
            PowFloat
        ],
        [
            PushInt(-20),
            PushFloat(0.5),
            NegateFloat,
            PushFloat(3.0),
            PowFloat
        ]
    );
    test_optimisation!(
        pow_float_not_applicable_2,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(0.5),
            PushFloat(3.0),
            NegateFloat,
            PowFloat
        ],
        [
            PushInt(-20),
            PushFloat(0.5),
            PushFloat(3.0),
            NegateFloat,
            PowFloat
        ]
    );

    test_optimisation!(
        mod_positive,
        FunctionOptimisations,
        [PushInt(-20), PushInt(17), PushInt(10), Mod],
        [PushInt(-20), PushInt(7)]
    );
    test_optimisation!(
        mod_negative,
        FunctionOptimisations,
        [PushInt(-20), PushInt(-17), PushInt(10), Mod],
        [PushInt(-20), PushInt(3)]
    );
    test_optimisation!(
        mod_not_applicable_1,
        FunctionOptimisations,
        [PushInt(-20), PushInt(17), NegateInt, PushInt(10), Mod],
        [PushInt(-20), PushInt(17), NegateInt, PushInt(10), Mod]
    );
    test_optimisation!(
        mod_not_applicable_2,
        FunctionOptimisations,
        [PushInt(-20), PushInt(17), PushInt(10), NegateInt, Mod],
        [PushInt(-20), PushInt(17), PushInt(10), NegateInt, Mod]
    );

    test_optimisation!(
        log,
        FunctionOptimisations,
        [PushInt(-20), PushFloat(2.25), PushFloat(1.5), LogFloat],
        [PushInt(-20), PushFloat(2.0)]
    );
    test_optimisation!(
        log_not_applicable_1,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(2.25),
            NegateFloat,
            PushFloat(1.5),
            LogFloat
        ],
        [
            PushInt(-20),
            PushFloat(2.25),
            NegateFloat,
            PushFloat(1.5),
            LogFloat
        ]
    );
    test_optimisation!(
        log_not_applicable_2,
        FunctionOptimisations,
        [
            PushInt(-20),
            PushFloat(2.25),
            PushFloat(1.5),
            NegateFloat,
            LogFloat
        ],
        [
            PushInt(-20),
            PushFloat(2.25),
            PushFloat(1.5),
            NegateFloat,
            LogFloat
        ]
    );
}
