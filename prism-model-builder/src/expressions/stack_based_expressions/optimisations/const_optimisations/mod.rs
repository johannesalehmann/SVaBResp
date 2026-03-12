use crate::expressions::stack_based_expressions::optimisations::Optimisation;
use probabilistic_models::Valuation;

mod binary_operations_boolean;
mod binary_operations_float;
mod binary_operations_integer;
mod functions;
mod implies;
mod int_to_float;
mod multiply_zero;
mod negate_bool;
mod negate_number;
mod one_operand_false;
mod one_operand_one_float;
mod one_operand_one_int;
mod one_operand_true;
mod one_operand_zero_float;
mod one_operand_zero_int;
mod push_const;
mod subtract_zero;
mod ternary;

pub fn get_const_optimisations<V: Valuation>(
    variable_info: &crate::variables::ModelVariableInfo<V>,
) -> Vec<Box<dyn Optimisation + '_>> {
    vec![
        Box::new(push_const::PushConstOptimisation { variable_info }),
        Box::new(int_to_float::IntToFloatOptimisation {}),
        Box::new(binary_operations_boolean::BinaryBooleanOptimisations {}),
        Box::new(binary_operations_float::BinaryFloatOptimisations {}),
        Box::new(binary_operations_integer::BinaryIntegerOptimisations {}),
        Box::new(negate_bool::NegateBoolOptimisation {}),
        Box::new(negate_number::NegateNumberOptimisation {}),
        Box::new(functions::FunctionOptimisations {}),
        Box::new(implies::ImpliesOptimisations {}),
        Box::new(multiply_zero::MultiplyByZeroOptimisation {}),
        Box::new(one_operand_false::OneOperandFalseOptimisations {}),
        Box::new(one_operand_one_float::OneOperandOneFloatOptimisations {}),
        Box::new(one_operand_one_int::OneOperandOneIntegerOptimisations {}),
        Box::new(one_operand_true::OneOperandTrueOptimisations {}),
        Box::new(one_operand_zero_float::OneOperandZeroFloatOptimisations {}),
        Box::new(one_operand_zero_int::OneOperandZeroIntegerOptimisations {}),
        Box::new(subtract_zero::SubtractZeroOptimisation {}),
        Box::new(ternary::TernaryOptimisation {}),
    ]
}
