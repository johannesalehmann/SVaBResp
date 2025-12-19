mod valuation_vector;

use std::fmt::Formatter;
pub use valuation_vector::ValuationVector;

pub trait Valuation: Sized + PartialEq + Clone + std::hash::Hash + Eq {
    type ContextType;
    type ContextBuilderType: ContextBuilder<Self::ContextType>;

    type BuilderType: ValuationBuilder<Self>;

    fn get_context_builder() -> Self::ContextBuilderType;
    fn get_builder(context: &Self::ContextType) -> Self::BuilderType;

    fn evaluate_bounded_int(&self, index: usize) -> i64;
    fn evaluate_bool(&self, index: usize) -> bool;
    fn evaluate_unbounded_int(&self, index: usize) -> i64;
    fn evaluate_float(&self, index: usize) -> f64;
    fn set_bounded_int(&mut self, index: usize, value: i64);
    fn set_bool(&mut self, index: usize, value: bool);
    fn set_unbounded_int(&mut self, index: usize, value: i64);
    fn set_float(&mut self, index: usize, value: f64);

    /// Not intended to be called directly. Instead, call valuation.displayable(context), the result of which implements std::fmt::Display
    fn format(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        context: &Self::ContextType,
    ) -> Result<(), std::fmt::Error>;
    fn displayable<'a, 'b>(
        &'a self,
        context: &'b Self::ContextType,
    ) -> DisplayableValuation<'a, 'b, Self> {
        DisplayableValuation {
            valuation: self,
            context,
        }
    }
}

pub struct DisplayableValuation<'a, 'b, V: Valuation> {
    valuation: &'a V,
    context: &'b V::ContextType,
}

impl<'a, 'b, V: Valuation> std::fmt::Display for DisplayableValuation<'a, 'b, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.valuation.format(f, self.context)
    }
}

pub trait ContextBuilder<C> {
    fn register_bounded_int(&mut self, name: String, min: i64, max: i64);
    fn register_bool(&mut self, name: String);
    fn register_unbounded_int(&mut self, name: String);
    fn register_float(&mut self, name: String);
    fn finish(self) -> C;
}

pub trait ValuationBuilder<V> {
    fn add_bounded_int(&mut self, value: i64);
    fn add_bool(&mut self, value: bool);
    fn add_int(&mut self, value: i64);
    fn add_float(&mut self, value: f64);
    fn finish(self) -> V;
}

#[derive(Copy, Clone, PartialEq)]
pub enum VariableType {
    BoundedInt,
    UnboundedInt,
    Bool,
    Float,
}

impl VariableType {
    pub fn is_int(self) -> bool {
        match self {
            VariableType::BoundedInt => true,
            VariableType::UnboundedInt => true,
            VariableType::Bool => false,
            VariableType::Float => false,
        }
    }
    pub fn is_bool(self) -> bool {
        match self {
            VariableType::BoundedInt => false,
            VariableType::UnboundedInt => false,
            VariableType::Bool => true,
            VariableType::Float => false,
        }
    }
    pub fn is_float(self) -> bool {
        match self {
            VariableType::BoundedInt => false,
            VariableType::UnboundedInt => false,
            VariableType::Bool => false,
            VariableType::Float => true,
        }
    }
}
