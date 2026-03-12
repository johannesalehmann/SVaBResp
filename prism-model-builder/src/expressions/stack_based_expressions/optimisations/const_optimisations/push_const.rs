use super::super::super::Operation;
use super::super::{OperationView, Optimisation, OptimisationResult};
use crate::expressions::stack_based_expressions::Operation::{PushBool, PushFloat, PushInt};
use crate::variables::{ConstValuation, ModelVariableInfo};
use probabilistic_models::Valuation;
use std::iter::once;

pub struct PushConstOptimisation<'a, V: Valuation> {
    pub variable_info: &'a ModelVariableInfo<V>,
}
impl<'a, V: Valuation> Optimisation for PushConstOptimisation<'a, V> {
    fn apply(
        &self,
        view: &mut OperationView<prism_model::VariableReference>,
    ) -> OptimisationResult {
        if let Operation::PushVarOrConstInt(reference) = view.current_operation() {
            if let Some(ConstValuation::Int(i)) = self.variable_info.value_of_const(*reference) {
                view.replace_operations(0, once(PushInt(i)));
                return OptimisationResult::Applied;
            }
        } else if let Operation::PushVarOrConstFloat(reference) = view.current_operation() {
            if let Some(ConstValuation::Float(f)) = self.variable_info.value_of_const(*reference) {
                view.replace_operations(0, once(PushFloat(f)));
                return OptimisationResult::Applied;
            }
        } else if let Operation::PushVarOrConstBool(reference) = view.current_operation() {
            if let Some(ConstValuation::Bool(b)) = self.variable_info.value_of_const(*reference) {
                view.replace_operations(0, once(PushBool(b)));
                return OptimisationResult::Applied;
            }
        }
        OptimisationResult::NotApplied
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_optimisation;
    use super::PushConstOptimisation;
    use crate::expressions::stack_based_expressions::Operation::*;
    use crate::variables::ModelVariableInfo;
    use prism_model::VariableReference;
    use probabilistic_models::ValuationVector;

    test_optimisation!(
        integer_zero_first,
        {
            PushConstOptimisation {
                variable_info: &ModelVariableInfo::<ValuationVector>::with_mock_values(),
            }
        },
        [
            PushInt(0),
            PushVarOrConstFloat(VariableReference::new(0)),
            PushVarOrConstInt(VariableReference::new(1)),
            PushVarOrConstInt(VariableReference::new(2)),
            PushVarOrConstBool(VariableReference::new(3)),
            PushVarOrConstBool(VariableReference::new(4)),
            PushVarOrConstFloat(VariableReference::new(5)),
            PushFloat(12.34)
        ],
        [
            PushInt(0),
            PushVarOrConstFloat(VariableReference::new(0)),
            PushVarOrConstInt(VariableReference::new(1)),
            PushInt(-5),
            PushBool(true),
            PushVarOrConstBool(VariableReference::new(4)),
            PushFloat(1.23),
            PushFloat(12.34)
        ]
    );
}
