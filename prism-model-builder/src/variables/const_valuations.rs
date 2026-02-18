use crate::expressions::ValuationSource;
use crate::expressions::stack_based_expressions::{StackBasedExpression, SubExpressionProvider};
use crate::{ExpressionContext, UserProvidedConstValue};
use prism_model::{VariableManager, VariableReference};
use std::collections::HashMap;

pub struct ConstValuations {
    valuations: Vec<ConstValuation>,
}

impl ConstValuations {
    pub fn new<S: Clone, SE: SubExpressionProvider>(
        variables: &VariableManager<StackBasedExpression<VariableReference>, S>,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
        expression_context: &mut ExpressionContext<SE>,
    ) -> Self {
        let mut valuations = Vec::new();
        for var in &variables.variables {
            if var.is_constant {
                valuations.push(Self::compute_initial_const_value(
                    variables,
                    user_provided_consts,
                    &var,
                    expression_context,
                ));
            }
        }
        Self { valuations }
    }

    fn compute_initial_const_value<S: Clone, SE: SubExpressionProvider>(
        variables: &VariableManager<StackBasedExpression<VariableReference>, S>,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
        var: &prism_model::VariableInfo<StackBasedExpression<VariableReference>, S>,
        expression_context: &mut ExpressionContext<SE>,
    ) -> ConstValuation {
        if let Some(value) = user_provided_consts.get(&var.name.name) {
            Self::process_user_initial_value(&var, value)
        } else {
            Self::evaluate_initial_expression(
                variables,
                user_provided_consts,
                var,
                expression_context,
            )
        }
    }

    fn process_user_initial_value<S: Clone>(
        var: &prism_model::VariableInfo<StackBasedExpression<VariableReference>, S>,
        value: &UserProvidedConstValue,
    ) -> ConstValuation {
        use crate::VariableRange;
        match (&var.range, value) {
            (VariableRange::BoundedInt { .. }, UserProvidedConstValue::Int(i)) => {
                ConstValuation::Int(*i)
            }
            (VariableRange::UnboundedInt { .. }, UserProvidedConstValue::Int(i)) => {
                ConstValuation::Int(*i)
            }
            (VariableRange::Boolean { .. }, UserProvidedConstValue::Bool(b)) => {
                ConstValuation::Bool(*b)
            }
            (VariableRange::Float { .. }, UserProvidedConstValue::Float(f)) => {
                ConstValuation::Float(*f)
            }
            _ => panic!("Incompatible value assigned to constant"),
        }
    }

    fn evaluate_initial_expression<S: Clone, SE: SubExpressionProvider>(
        variables: &VariableManager<StackBasedExpression<VariableReference>, S>,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
        var: &prism_model::VariableInfo<StackBasedExpression<VariableReference>, S>,
        expression_context: &mut ExpressionContext<SE>,
    ) -> ConstValuation {
        let value_source = ConstRecursiveEvaluator::new(variables, user_provided_consts);
        let initial = var
            .initial_value
            .as_ref()
            .expect("Consts must have an initial value expression");
        use crate::VariableRange;
        match var.range {
            VariableRange::BoundedInt { .. } | VariableRange::UnboundedInt { .. } => {
                ConstValuation::Int(expression_context.evaluate_int(initial, &value_source))
            }
            VariableRange::Boolean { .. } => {
                ConstValuation::Bool(expression_context.evaluate_bool(initial, &value_source))
            }
            VariableRange::Float { .. } => {
                ConstValuation::Float(expression_context.evaluate_float(initial, &value_source))
            }
        }
    }
}

impl std::ops::Index<usize> for ConstValuations {
    type Output = ConstValuation;

    fn index(&self, index: usize) -> &Self::Output {
        &self.valuations[index]
    }
}

pub enum ConstValuation {
    Int(i64),
    Bool(bool),
    Float(f64),
}

impl ConstValuation {
    pub fn as_int(&self) -> i64 {
        match self {
            ConstValuation::Int(i) => *i,
            _ => panic!("Cannot evaluate this value as integer"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            ConstValuation::Bool(b) => *b,
            _ => panic!("Cannot evaluate this value as boolean"),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            ConstValuation::Float(f) => *f,
            _ => panic!("Cannot evaluate this value as float"),
        }
    }
}

struct ConstRecursiveEvaluator<'a, 'b, S: Clone> {
    variables: &'a VariableManager<StackBasedExpression<VariableReference>, S>,
    const_values: &'b HashMap<String, UserProvidedConstValue>,
}

impl<'a, 'b, S: Clone> ConstRecursiveEvaluator<'a, 'b, S> {
    pub fn new(
        variables: &'a VariableManager<StackBasedExpression<VariableReference>, S>,
        const_values: &'b HashMap<String, UserProvidedConstValue>,
    ) -> Self {
        Self {
            variables,
            const_values,
        }
    }
}

impl<'a, 'b, S: Clone> ValuationSource for ConstRecursiveEvaluator<'a, 'b, S> {
    fn get_int(&self, index: VariableReference) -> i64 {
        let var = self.variables.get(&index).unwrap();
        if !var.is_constant {
            panic!("Const depends on non-constant value");
        }
        if let Some(value) = self.const_values.get(&var.name.name) {
            match value {
                UserProvidedConstValue::Int(i) => *i,
                UserProvidedConstValue::Bool(_) => {
                    panic!("Integer constant assigned boolean value")
                }
                UserProvidedConstValue::Float(_) => {
                    panic!("Integer constant assigned float value")
                }
            }
        } else {
            var.initial_value
                .as_ref()
                .expect("Constant without initial value")
                .evaluate_as_int(self)
        }
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        let var = self.variables.get(&index).unwrap();
        if !var.is_constant {
            panic!("Const depends on non-constant value");
        }
        if let Some(value) = self.const_values.get(&var.name.name) {
            match value {
                UserProvidedConstValue::Int(_) => {
                    panic!("Boolean constant assigned integer value")
                }
                UserProvidedConstValue::Bool(b) => *b,
                UserProvidedConstValue::Float(_) => {
                    panic!("Boolean constant assigned float value")
                }
            }
        } else {
            var.initial_value
                .as_ref()
                .expect("Constant without initial value")
                .evaluate_as_bool(self)
        }
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        let var = self.variables.get(&index).unwrap();
        if !var.is_constant {
            panic!("Const depends on non-constant value");
        }
        if let Some(value) = self.const_values.get(&var.name.name) {
            match value {
                UserProvidedConstValue::Int(_) => {
                    panic!("Float constant assigned integer value")
                }
                UserProvidedConstValue::Bool(_) => {
                    panic!("Float constant assigned boolean value")
                }
                UserProvidedConstValue::Float(f) => *f,
            }
        } else {
            var.initial_value
                .as_ref()
                .expect("Constant without initial value")
                .evaluate_as_float(self)
        }
    }

    fn get_type(&self, index: VariableReference) -> crate::VariableType {
        use crate::VariableType;
        let var = self.variables.get(&index).unwrap();
        VariableType::from_range(&var.range)
    }
}
