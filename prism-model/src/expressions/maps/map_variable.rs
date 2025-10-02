use crate::{Expression, Identifier, MapExpression};
use std::marker::PhantomData;

pub struct MapVariable<V, V2, C, F: Fn(V, &mut C) -> V2> {
    pub(crate) map: F,
    pub context: C,
    phantom_data: PhantomData<(V, V2)>,
}
impl<V, V2, C, F: Fn(V, &mut C) -> V2> MapVariable<V, V2, C, F> {
    pub fn new(map: F, context: C) -> Self {
        Self {
            map,
            context,
            phantom_data: PhantomData,
        }
    }
}
#[allow(private_bounds)]
impl<V, V2, C, S: Clone, F: Fn(V, &mut C) -> V2> MapExpression<V, S, Expression<V2, S>>
    for MapVariable<V, V2, C, F>
{
    fn visit_int(&mut self, val: i64, span: S) -> Expression<V2, S> {
        Expression::Int(val, span)
    }
    fn visit_float(&mut self, val: f64, span: S) -> Expression<V2, S> {
        Expression::Float(val, span)
    }
    fn visit_bool(&mut self, val: bool, span: S) -> Expression<V2, S> {
        Expression::Bool(val, span)
    }
    fn visit_var_or_const(&mut self, name: V, span: S) -> Expression<V2, S> {
        Expression::VarOrConst((self.map)(name, &mut self.context), span)
    }
    fn visit_label(&mut self, name: V, span: S) -> Expression<V2, S> {
        Expression::Label((self.map)(name, &mut self.context), span)
    }
    fn visit_function(
        &mut self,
        identifier: Identifier<S>,
        arguments: Vec<Expression<V2, S>>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Function(identifier, arguments, span)
    }
    fn visit_minus(&mut self, inner: Expression<V2, S>, span: S) -> Expression<V2, S> {
        Expression::Minus(Box::new(inner), span)
    }
    fn visit_multiplication(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Multiplication(Box::new(left), Box::new(right), span)
    }
    fn visit_division(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Division(Box::new(left), Box::new(right), span)
    }
    fn visit_addition(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Addition(Box::new(left), Box::new(right), span)
    }
    fn visit_subtraction(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Subtraction(Box::new(left), Box::new(right), span)
    }
    fn visit_less_than(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::LessThan(Box::new(left), Box::new(right), span)
    }
    fn visit_less_or_equal(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::LessOrEqual(Box::new(left), Box::new(right), span)
    }
    fn visit_greater_than(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::GreaterThan(Box::new(left), Box::new(right), span)
    }
    fn visit_greater_or_equal(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::GreaterOrEqual(Box::new(left), Box::new(right), span)
    }
    fn visit_equals(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Equals(Box::new(left), Box::new(right), span)
    }
    fn visit_not_equals(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::NotEquals(Box::new(left), Box::new(right), span)
    }
    fn visit_negation(&mut self, inner: Expression<V2, S>, span: S) -> Expression<V2, S> {
        Expression::Negation(Box::new(inner), span)
    }
    fn visit_conjunction(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Conjunction(Box::new(left), Box::new(right), span)
    }
    fn visit_disjunction(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Disjunction(Box::new(left), Box::new(right), span)
    }
    fn visit_if_and_only_if(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::IfAndOnlyIf(Box::new(left), Box::new(right), span)
    }
    fn visit_implies(
        &mut self,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Implies(Box::new(left), Box::new(right), span)
    }
    fn visit_ternary(
        &mut self,
        condition: Expression<V2, S>,
        left: Expression<V2, S>,
        right: Expression<V2, S>,
        span: S,
    ) -> Expression<V2, S> {
        Expression::Ternary(Box::new(condition), Box::new(left), Box::new(right), span)
    }
}
