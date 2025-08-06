use crate::{Expression, Identifier, MapExpression};
use std::marker::PhantomData;

pub struct MapSpan<S, S2, F: Fn(S) -> S2> {
    pub(crate) map: F,
    phantom_data: PhantomData<(S, S2)>,
}

impl<S, S2, F: Fn(S) -> S2> MapSpan<S, S2, F> {
    pub fn new(map: F) -> Self {
        Self {
            map,
            phantom_data: PhantomData,
        }
    }
}

impl<V, S: Clone, S2: Clone, F: Fn(S) -> S2> MapExpression<V, S, super::Expression<V, S2>>
    for MapSpan<S, S2, F>
{
    fn visit_int(&mut self, val: i64, span: S) -> Expression<V, S2> {
        Expression::Int(val, (self.map)(span))
    }

    fn visit_float(&mut self, val: f64, span: S) -> Expression<V, S2> {
        Expression::Float(val, (self.map)(span))
    }

    fn visit_bool(&mut self, val: bool, span: S) -> Expression<V, S2> {
        Expression::Bool(val, (self.map)(span))
    }

    fn visit_var_or_const(&mut self, name: V, span: S) -> Expression<V, S2> {
        Expression::VarOrConst(name, (self.map)(span))
    }

    fn visit_function(
        &mut self,
        identifier: Identifier<S>,
        arguments: Vec<Expression<V, S2>>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Function(identifier.map_span(&self.map), arguments, (self.map)(span))
    }

    fn visit_minus(&mut self, inner: Expression<V, S2>, span: S) -> Expression<V, S2> {
        Expression::Minus(Box::new(inner), (self.map)(span))
    }

    fn visit_multiplication(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Multiplication(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_division(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Division(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_addition(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Addition(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_subtraction(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Subtraction(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_less_than(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::LessThan(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_less_or_equal(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::LessOrEqual(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_greater_than(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::GreaterThan(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_greater_or_equal(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::GreaterOrEqual(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_equals(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Equals(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_not_equals(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::NotEquals(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_negation(&mut self, inner: Expression<V, S2>, span: S) -> Expression<V, S2> {
        Expression::Negation(Box::new(inner), (self.map)(span))
    }

    fn visit_conjunction(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Conjunction(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_disjunction(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Disjunction(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_if_and_only_if(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::IfAndOnlyIf(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_implies(
        &mut self,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Implies(Box::new(left), Box::new(right), (self.map)(span))
    }

    fn visit_ternary(
        &mut self,
        condition: Expression<V, S2>,
        left: Expression<V, S2>,
        right: Expression<V, S2>,
        span: S,
    ) -> Expression<V, S2> {
        Expression::Ternary(
            Box::new(condition),
            Box::new(left),
            Box::new(right),
            (self.map)(span),
        )
    }
}
