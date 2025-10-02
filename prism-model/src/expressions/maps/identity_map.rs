use crate::expressions::MapExpression;
use crate::{Expression, Identifier};

pub(crate) trait Private {}

#[allow(private_bounds)]
pub trait IdentityMapExpression<V, S: Clone>: Private {
    fn visit_int(&mut self, val: i64, span: S) -> Expression<V, S> {
        Expression::Int(val, span)
    }
    fn visit_float(&mut self, val: f64, span: S) -> Expression<V, S> {
        Expression::Float(val, span)
    }
    fn visit_bool(&mut self, val: bool, span: S) -> Expression<V, S> {
        Expression::Bool(val, span)
    }
    fn visit_var_or_const(&mut self, name: V, span: S) -> Expression<V, S> {
        Expression::VarOrConst(name, span)
    }
    fn visit_label(&mut self, name: V, span: S) -> Expression<V, S> {
        Expression::Label(name, span)
    }
    fn visit_function(
        &mut self,
        identifier: Identifier<S>,
        arguments: Vec<Expression<V, S>>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Function(identifier, arguments, span)
    }
    fn visit_minus(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        Expression::Minus(Box::new(inner), span)
    }
    fn visit_multiplication(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Multiplication(Box::new(left), Box::new(right), span)
    }
    fn visit_division(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Division(Box::new(left), Box::new(right), span)
    }
    fn visit_addition(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Addition(Box::new(left), Box::new(right), span)
    }
    fn visit_subtraction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Subtraction(Box::new(left), Box::new(right), span)
    }
    fn visit_less_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::LessThan(Box::new(left), Box::new(right), span)
    }
    fn visit_less_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::LessOrEqual(Box::new(left), Box::new(right), span)
    }
    fn visit_greater_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::GreaterThan(Box::new(left), Box::new(right), span)
    }
    fn visit_greater_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::GreaterOrEqual(Box::new(left), Box::new(right), span)
    }
    fn visit_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Equals(Box::new(left), Box::new(right), span)
    }
    fn visit_not_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::NotEquals(Box::new(left), Box::new(right), span)
    }
    fn visit_negation(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        Expression::Negation(Box::new(inner), span)
    }
    fn visit_conjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Conjunction(Box::new(left), Box::new(right), span)
    }
    fn visit_disjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Disjunction(Box::new(left), Box::new(right), span)
    }
    fn visit_if_and_only_if(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::IfAndOnlyIf(Box::new(left), Box::new(right), span)
    }
    fn visit_implies(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Implies(Box::new(left), Box::new(right), span)
    }
    fn visit_ternary(
        &mut self,
        condition: Expression<V, S>,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        Expression::Ternary(Box::new(condition), Box::new(left), Box::new(right), span)
    }
}

impl<V, S: Clone, M: IdentityMapExpression<V, S>> MapExpression<V, S, Expression<V, S>> for M {
    fn visit_int(&mut self, val: i64, span: S) -> Expression<V, S> {
        self.visit_int(val, span)
    }
    fn visit_float(&mut self, val: f64, span: S) -> Expression<V, S> {
        self.visit_float(val, span)
    }
    fn visit_bool(&mut self, val: bool, span: S) -> Expression<V, S> {
        self.visit_bool(val, span)
    }
    fn visit_var_or_const(&mut self, name: V, span: S) -> Expression<V, S> {
        self.visit_var_or_const(name, span)
    }
    fn visit_label(&mut self, name: V, span: S) -> Expression<V, S> {
        self.visit_label(name, span)
    }
    fn visit_function(
        &mut self,
        identifier: Identifier<S>,
        arguments: Vec<Expression<V, S>>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_function(identifier, arguments, span)
    }
    fn visit_minus(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        self.visit_minus(inner, span)
    }
    fn visit_multiplication(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_multiplication(left, right, span)
    }
    fn visit_division(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_division(left, right, span)
    }
    fn visit_addition(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_addition(left, right, span)
    }
    fn visit_subtraction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_subtraction(left, right, span)
    }
    fn visit_less_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_less_than(left, right, span)
    }
    fn visit_less_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_less_or_equal(left, right, span)
    }
    fn visit_greater_than(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_greater_than(left, right, span)
    }
    fn visit_greater_or_equal(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_greater_or_equal(left, right, span)
    }
    fn visit_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_equals(left, right, span)
    }
    fn visit_not_equals(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_not_equals(left, right, span)
    }
    fn visit_negation(&mut self, inner: Expression<V, S>, span: S) -> Expression<V, S> {
        self.visit_negation(inner, span)
    }
    fn visit_conjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_conjunction(left, right, span)
    }
    fn visit_disjunction(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_disjunction(left, right, span)
    }
    fn visit_if_and_only_if(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_if_and_only_if(left, right, span)
    }
    fn visit_implies(
        &mut self,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_implies(left, right, span)
    }
    fn visit_ternary(
        &mut self,
        condition: Expression<V, S>,
        left: Expression<V, S>,
        right: Expression<V, S>,
        span: S,
    ) -> Expression<V, S> {
        self.visit_ternary(condition, left, right, span)
    }
}
