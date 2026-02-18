use crate::ExpressionContext;
use crate::expressions::VariableType;
use crate::expressions::stack_based_expressions::{StackBasedExpression, SubExpressionProvider};
use crate::variables::const_valuations::ConstValuations;
use crate::variables::valuation_map::{ValuationMap, ValuationMapEntry};
use prism_model::{VariableManager, VariableReference};

pub struct VariableDetail {
    pub bounds: Option<(i64, i64)>,
    pub variable_type: VariableType,
}

pub struct VariableDetails {
    details: Vec<VariableDetail>,
}
impl VariableDetails {
    pub fn new<S: Clone, SE: SubExpressionProvider>(
        variables: &VariableManager<StackBasedExpression<VariableReference>, S>,
        valuation_map: &ValuationMap,
        const_values: &ConstValuations,
        expression_context: &mut ExpressionContext<SE>,
    ) -> Self {
        let mut details = Vec::new();
        let const_value_source = super::ConstOnlyValuationSource::new(valuation_map, const_values);

        for (i, variable) in variables.variables.iter().enumerate() {
            if let ValuationMapEntry::Var(_) = valuation_map[i] {
                let bounds = match &variable.range {
                    prism_model::VariableRange::BoundedInt { min, max, .. } => {
                        let min = expression_context.evaluate_int(min, &const_value_source);
                        let max = expression_context.evaluate_int(max, &const_value_source);
                        Some((min, max))
                    }
                    _ => None,
                };

                let variable_type = VariableType::from_range(&variable.range);

                details.push(VariableDetail {
                    bounds,
                    variable_type,
                })
            }
        }

        Self { details }
    }
}

impl std::ops::Index<usize> for VariableDetails {
    type Output = VariableDetail;

    fn index(&self, index: usize) -> &Self::Output {
        &self.details[index]
    }
}
