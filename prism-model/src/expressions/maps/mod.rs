mod default_map;
pub use default_map::DefaultMapExpression;

pub mod identity_map;
pub use identity_map::IdentityMapExpression;

pub mod map_span;

pub mod map_variable;

use super::{Expression, Identifier};

pub trait MapExpression<V, S: Clone, T> {
    fn visit_int(&mut self, val: i64, span: S) -> T;
    fn visit_float(&mut self, val: f64, span: S) -> T;
    fn visit_bool(&mut self, val: bool, span: S) -> T;
    fn visit_var_or_const(&mut self, name: V, span: S) -> T;
    fn visit_function(&mut self, identifier: Identifier<S>, arguments: Vec<T>, span: S) -> T;
    fn visit_minus(&mut self, inner: T, span: S) -> T;
    fn visit_multiplication(&mut self, left: T, right: T, span: S) -> T;
    fn visit_division(&mut self, left: T, right: T, span: S) -> T;
    fn visit_addition(&mut self, left: T, right: T, span: S) -> T;
    fn visit_subtraction(&mut self, left: T, right: T, span: S) -> T;
    fn visit_less_than(&mut self, left: T, right: T, span: S) -> T;
    fn visit_less_or_equal(&mut self, left: T, right: T, span: S) -> T;
    fn visit_greater_than(&mut self, left: T, right: T, span: S) -> T;
    fn visit_greater_or_equal(&mut self, left: T, right: T, span: S) -> T;
    fn visit_equals(&mut self, left: T, right: T, span: S) -> T;
    fn visit_not_equals(&mut self, left: T, right: T, span: S) -> T;
    fn visit_negation(&mut self, inner: T, span: S) -> T;
    fn visit_conjunction(&mut self, left: T, right: T, span: S) -> T;
    fn visit_disjunction(&mut self, left: T, right: T, span: S) -> T;
    fn visit_if_and_only_if(&mut self, left: T, right: T, span: S) -> T;
    fn visit_implies(&mut self, left: T, right: T, span: S) -> T;
    fn visit_ternary(&mut self, condition: T, left: T, right: T, span: S) -> T;
}

impl<V, S: Clone> Expression<V, S> {
    pub fn visit<T, M: MapExpression<V, S, T>>(self, m: &mut M) -> T {
        match self {
            Expression::Int(val, s) => m.visit_int(val, s),
            Expression::Float(val, s) => m.visit_float(val, s),
            Expression::Bool(val, s) => m.visit_bool(val, s),
            Expression::VarOrConst(name, s) => m.visit_var_or_const(name, s),
            Expression::Function(identifier, arguments, s) => {
                let mapped_args = arguments
                    .into_iter()
                    .map(|a| a.visit(m))
                    .collect::<Vec<_>>();
                m.visit_function(identifier, mapped_args, s)
            }
            Expression::Minus(inner, s) => {
                let inner = inner.visit(m);
                m.visit_minus(inner, s)
            }
            Expression::Multiplication(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_multiplication(lhs, rhs, s)
            }
            Expression::Division(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_division(lhs, rhs, s)
            }
            Expression::Addition(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_addition(lhs, rhs, s)
            }
            Expression::Subtraction(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_subtraction(lhs, rhs, s)
            }
            Expression::LessThan(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_less_than(lhs, rhs, s)
            }
            Expression::LessOrEqual(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_less_or_equal(lhs, rhs, s)
            }
            Expression::GreaterThan(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_greater_than(lhs, rhs, s)
            }
            Expression::GreaterOrEqual(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_greater_or_equal(lhs, rhs, s)
            }
            Expression::Equals(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_equals(lhs, rhs, s)
            }
            Expression::NotEquals(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_not_equals(lhs, rhs, s)
            }
            Expression::Negation(inner, s) => {
                let inner = inner.visit(m);
                m.visit_negation(inner, s)
            }
            Expression::Conjunction(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_conjunction(lhs, rhs, s)
            }
            Expression::Disjunction(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_disjunction(lhs, rhs, s)
            }
            Expression::IfAndOnlyIf(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_if_and_only_if(lhs, rhs, s)
            }
            Expression::Implies(lhs, rhs, s) => {
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_implies(lhs, rhs, s)
            }
            Expression::Ternary(condition, lhs, rhs, s) => {
                let condition = condition.visit(m);
                let lhs = lhs.visit(m);
                let rhs = rhs.visit(m);
                m.visit_ternary(condition, lhs, rhs, s)
            }
        }
    }
}
