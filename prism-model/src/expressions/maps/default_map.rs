use crate::expressions::MapExpression;
use crate::Identifier;

#[allow(unused_variables)]
pub trait DefaultMapExpression<V, S: Clone, T: Default> {
    fn visit_int(&mut self, val: i64, span: S) -> T {
        T::default()
    }
    fn visit_float(&mut self, val: f64, span: S) -> T {
        T::default()
    }
    fn visit_bool(&mut self, val: bool, span: S) -> T {
        T::default()
    }
    fn visit_var_or_const(&mut self, name: V, span: S) -> T {
        T::default()
    }
    fn visit_function(&mut self, identifier: Identifier<S>, arguments: Vec<T>, span: S) -> T {
        T::default()
    }
    fn visit_minus(&mut self, inner: T, span: S) -> T {
        T::default()
    }
    fn visit_multiplication(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_division(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_addition(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_subtraction(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_less_than(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_less_or_equal(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_greater_than(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_greater_or_equal(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_equals(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_not_equals(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_negation(&mut self, inner: T, span: S) -> T {
        T::default()
    }
    fn visit_conjunction(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_disjunction(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_if_and_only_if(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_implies(&mut self, left: T, right: T, span: S) -> T {
        T::default()
    }
    fn visit_ternary(&mut self, condition: T, left: T, right: T, span: S) -> T {
        T::default()
    }
}

impl<V, S: Clone, T: Default, M: DefaultMapExpression<V, S, T>> MapExpression<V, S, T> for M {
    fn visit_int(&mut self, val: i64, span: S) -> T {
        self.visit_int(val, span)
    }
    fn visit_float(&mut self, val: f64, span: S) -> T {
        self.visit_float(val, span)
    }
    fn visit_bool(&mut self, val: bool, span: S) -> T {
        self.visit_bool(val, span)
    }
    fn visit_var_or_const(&mut self, name: V, span: S) -> T {
        self.visit_var_or_const(name, span)
    }
    fn visit_function(&mut self, identifier: Identifier<S>, arguments: Vec<T>, span: S) -> T {
        self.visit_function(identifier, arguments, span)
    }
    fn visit_minus(&mut self, inner: T, span: S) -> T {
        self.visit_minus(inner, span)
    }
    fn visit_multiplication(&mut self, left: T, right: T, span: S) -> T {
        self.visit_multiplication(left, right, span)
    }
    fn visit_division(&mut self, left: T, right: T, span: S) -> T {
        self.visit_division(left, right, span)
    }
    fn visit_addition(&mut self, left: T, right: T, span: S) -> T {
        self.visit_addition(left, right, span)
    }
    fn visit_subtraction(&mut self, left: T, right: T, span: S) -> T {
        self.visit_subtraction(left, right, span)
    }
    fn visit_less_than(&mut self, left: T, right: T, span: S) -> T {
        self.visit_less_than(left, right, span)
    }
    fn visit_less_or_equal(&mut self, left: T, right: T, span: S) -> T {
        self.visit_less_or_equal(left, right, span)
    }
    fn visit_greater_than(&mut self, left: T, right: T, span: S) -> T {
        self.visit_greater_than(left, right, span)
    }
    fn visit_greater_or_equal(&mut self, left: T, right: T, span: S) -> T {
        self.visit_greater_or_equal(left, right, span)
    }
    fn visit_equals(&mut self, left: T, right: T, span: S) -> T {
        self.visit_equals(left, right, span)
    }
    fn visit_not_equals(&mut self, left: T, right: T, span: S) -> T {
        self.visit_not_equals(left, right, span)
    }
    fn visit_negation(&mut self, inner: T, span: S) -> T {
        self.visit_negation(inner, span)
    }
    fn visit_conjunction(&mut self, left: T, right: T, span: S) -> T {
        self.visit_conjunction(left, right, span)
    }
    fn visit_disjunction(&mut self, left: T, right: T, span: S) -> T {
        self.visit_disjunction(left, right, span)
    }
    fn visit_if_and_only_if(&mut self, left: T, right: T, span: S) -> T {
        self.visit_if_and_only_if(left, right, span)
    }
    fn visit_implies(&mut self, left: T, right: T, span: S) -> T {
        self.visit_implies(left, right, span)
    }
    fn visit_ternary(&mut self, condition: T, left: T, right: T, span: S) -> T {
        self.visit_ternary(condition, left, right, span)
    }
}
