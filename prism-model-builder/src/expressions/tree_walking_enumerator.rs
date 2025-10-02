use crate::expressions::ValuationSource;
use prism_model::{Expression, Identifier, VariableReference};
use std::ops::{Add, Div, Mul, Sub};

pub struct TreeWalkingEvaluator {}

impl TreeWalkingEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    fn evaluate<V: ValuationSource, S: Clone>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> Value {
        match expression {
            Expression::Int(val, _) => Value::Int(*val),
            Expression::Float(val, _) => Value::Float(*val),
            Expression::Bool(val, _) => Value::Bool(*val),
            Expression::VarOrConst(id, _) => {
                // TODO: This is just temporary -- we need to identify the type of a variable reference before-hand
                Value::Int(valuations.get_int(*id))
            }
            Expression::Label(id, _) => {
                panic!("Cannot evaluate expression containing label. They must only occur in objectives")
            }
            Expression::Function(name, params, _) => {
                self.evaluate_function(name, params, valuations)
            }
            Expression::Minus(inner, _) => {
                let inner = self.evaluate(inner, valuations);
                if inner.is_int() {
                    Value::Int(-inner.as_int())
                } else {
                    Value::Float(-inner.as_float())
                }
            }
            Expression::Multiplication(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, i64::mul, f64::mul)
            }
            Expression::Division(lhs, rhs, _) => self.binary_operation(
                lhs,
                rhs,
                valuations,
                |lhs: i64, rhs: i64| (lhs as f64 / rhs as f64),
                f64::div,
            ),
            Expression::Addition(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, i64::add, f64::add)
            }
            Expression::Subtraction(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, i64::sub, f64::sub)
            }
            Expression::LessThan(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, |l, r| l < r, |l, r| l < r)
            }
            Expression::LessOrEqual(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, |l, r| l <= r, |l, r| l <= r)
            }
            Expression::GreaterThan(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, |l, r| l > r, |l, r| l > r)
            }
            Expression::GreaterOrEqual(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, |l, r| l >= r, |l, r| l >= r)
            }
            Expression::Equals(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, |l, r| l == r, |l, r| l == r)
            }
            Expression::NotEquals(lhs, rhs, _) => {
                self.binary_operation(lhs, rhs, valuations, |l, r| l != r, |l, r| l != r)
            }
            Expression::Negation(inner, _) => {
                let inner = self.evaluate(inner, valuations);
                if inner.is_bool() {
                    Value::Bool(!inner.as_bool())
                } else {
                    panic!("Incorrect type for negation")
                }
            }
            Expression::Conjunction(lhs, rhs, _) => {
                self.binary_boolean_operation(lhs, rhs, valuations, |l, r| l && r)
            }
            Expression::Disjunction(lhs, rhs, _) => {
                self.binary_boolean_operation(lhs, rhs, valuations, |l, r| l || r)
            }
            Expression::IfAndOnlyIf(lhs, rhs, _) => {
                self.binary_boolean_operation(lhs, rhs, valuations, |l, r| l == r)
            }
            Expression::Implies(lhs, rhs, _) => {
                self.binary_boolean_operation(lhs, rhs, valuations, |l, r| !l || r)
            }
            Expression::Ternary(condition, lhs, rhs, _) => {
                let condition = self.evaluate(condition, valuations);
                if condition.is_bool() {
                    if condition.as_bool() {
                        self.evaluate(lhs, valuations)
                    } else {
                        self.evaluate(rhs, valuations)
                    }
                } else {
                    panic!("Incorrect type for the condition of the ternary operation")
                }
            }
        }
    }

    fn evaluate_function<V: ValuationSource, S: Clone>(
        &self,
        name: &Identifier<S>,
        params: &Vec<Expression<VariableReference, S>>,
        valuations: &V,
    ) -> Value {
        match &name.name[..] {
            "min" => self.min_or_max(params, valuations, "min", i64::min, f64::min),
            "max" => self.min_or_max(params, valuations, "max", i64::max, f64::max),
            "floor" => self.round(params, valuations, "floor", f64::floor),
            "ceil" => self.round(params, valuations, "ceil", f64::ceil),
            "round" => self.round(params, valuations, "round", |f| {
                if f < 0.0 {
                    -(-f).round() // Ensure that -n.5 is rounded to -n and not to -(n+1)
                } else {
                    f.round()
                }
            }),
            "pow" => {
                if params.len() != 2 {
                    panic!("Function `pow` expects exactly 2 arguments");
                }
                let v0 = self.evaluate(&params[0], valuations);
                let v1 = self.evaluate(&params[1], valuations);
                if v0.is_int() && v1.is_int() {
                    Value::Int(v0.as_int().pow(v1.as_int() as u32))
                } else {
                    Value::Float(v0.as_float().powf(v1.as_float()))
                }
            }
            "mod" => {
                if params.len() != 2 {
                    panic!("Function `mod` expects exactly 2 arguments");
                }
                let v0 = self.evaluate(&params[0], valuations);
                let v1 = self.evaluate(&params[1], valuations);
                if v0.is_int() && v1.is_int() {
                    Value::Int(v0.as_int().rem_euclid(v1.as_int().into()))
                } else {
                    panic!("Incorrect types for function `mod`.")
                }
            }
            "log" => {
                if params.len() != 2 {
                    panic!("Function `log` expects exactly 2 arguments");
                }
                let v0 = self.evaluate(&params[0], valuations);
                let v1 = self.evaluate(&params[1], valuations);

                Value::Float(v0.as_float().log(v1.as_float()))
            }
            name => panic!("Unknown function `{}`", name),
        }
    }

    fn min_or_max<
        V: ValuationSource,
        S: Clone,
        FI: Fn(i64, i64) -> i64,
        FF: Fn(f64, f64) -> f64,
    >(
        &self,
        params: &Vec<Expression<VariableReference, S>>,
        valuations: &V,
        name: &'static str,
        i_op: FI,
        f_op: FF,
    ) -> Value {
        if params.len() < 2 {
            panic!("Function `{}` expects at least two parameters", name)
        };
        let values = params
            .iter()
            .map(|p| self.evaluate(p, valuations))
            .collect::<Vec<_>>();
        if values.iter().all(|v| v.is_int()) {
            let mut smallest = values[0].as_int();
            for value in values[1..].iter() {
                smallest = i_op(smallest, value.as_int());
            }
            Value::Int(smallest)
        } else {
            let mut smallest = values[0].as_float();
            for value in values[1..].iter() {
                smallest = f_op(smallest, value.as_float());
            }
            Value::Float(smallest)
        }
    }

    fn round<V: ValuationSource, S: Clone, FF: Fn(f64) -> f64>(
        &self,
        params: &Vec<Expression<VariableReference, S>>,
        valuations: &V,
        name: &'static str,
        op: FF,
    ) -> Value {
        if params.len() != 1 {
            panic!("Function `{}` expects exactly one parameters", name);
        }
        let inner_value = self.evaluate(&params[0], valuations).as_float();
        Value::Int(op(inner_value) as i64)
    }

    fn binary_operation<
        V: ValuationSource,
        S: Clone,
        IOut: Into<Value>,
        FOut: Into<Value>,
        FI: Fn(i64, i64) -> IOut,
        FF: Fn(f64, f64) -> FOut,
    >(
        &self,
        lhs: &Expression<VariableReference, S>,
        rhs: &Expression<VariableReference, S>,
        valuations: &V,
        i_op: FI,
        f_op: FF,
    ) -> Value {
        let lhs = self.evaluate(lhs, valuations);
        let rhs = self.evaluate(rhs, valuations);
        if lhs.is_int() && rhs.is_int() {
            i_op(lhs.as_int(), rhs.as_int()).into()
        } else {
            f_op(lhs.as_float(), rhs.as_float()).into().into()
        }
    }

    fn binary_boolean_operation<
        V: ValuationSource,
        S: Clone,
        TB: Into<Value>,
        FB: Fn(bool, bool) -> TB,
    >(
        &self,
        lhs: &Expression<VariableReference, S>,
        rhs: &Expression<VariableReference, S>,
        valuations: &V,
        b_op: FB,
    ) -> Value {
        let lhs = self.evaluate(lhs, valuations);
        let rhs = self.evaluate(rhs, valuations);
        if lhs.is_bool() && rhs.is_bool() {
            b_op(lhs.as_bool(), rhs.as_bool()).into()
        } else {
            panic!("Incorrect types");
        }
    }
}

#[derive(Clone)]
enum Value {
    Int(i64),
    Bool(bool),
    Float(f64),
}

impl Into<Value> for i64 {
    fn into(self) -> Value {
        Value::Int(self)
    }
}
impl Into<Value> for bool {
    fn into(self) -> Value {
        Value::Bool(self)
    }
}
impl Into<Value> for f64 {
    fn into(self) -> Value {
        Value::Float(self)
    }
}

impl Value {
    fn is_int(&self) -> bool {
        match self {
            Self::Int(_) => true,
            _ => false,
        }
    }
    fn is_float(&self) -> bool {
        match self {
            Self::Int(_) => true,
            Self::Float(_) => true,
            _ => false,
        }
    }
    fn is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false,
        }
    }

    fn as_int(&self) -> i64 {
        match self {
            Self::Int(val) => *val,
            _ => panic!("Cannot evaluate this type as int"),
        }
    }
    fn as_float(&self) -> f64 {
        match self {
            Self::Int(val) => *val as f64,
            Self::Float(val) => *val,
            _ => panic!("Cannot evaluate this type as float"),
        }
    }
    fn as_bool(&self) -> bool {
        match self {
            Self::Bool(val) => *val,
            _ => panic!("Cannot evaluate this type as bool"),
        }
    }
}

impl super::Evaluator for TreeWalkingEvaluator {
    fn create() -> Self {
        Self::new()
    }

    fn evaluate_as_int<V: ValuationSource, S: Clone>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> i64 {
        self.evaluate(expression, valuations).as_int()
    }

    fn evaluate_as_bool<V: ValuationSource, S: Clone>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> bool {
        self.evaluate(expression, valuations).as_bool()
    }

    fn evaluate_as_float<V: ValuationSource, S: Clone>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> f64 {
        self.evaluate(expression, valuations).as_float()
    }
}
