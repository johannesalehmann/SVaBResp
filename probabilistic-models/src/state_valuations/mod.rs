mod valuation_vector;
pub use valuation_vector::ValuationVector;

pub trait Valuation: Sized + PartialEq + Clone {
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
}

pub trait ContextBuilder<C> {
    fn register_bounded_int(&mut self, min: i64, max: i64);
    fn register_bool(&mut self);
    fn register_unbounded_int(&mut self);
    fn register_float(&mut self);
    fn finish(self) -> C;
}

pub trait ValuationBuilder<V> {
    fn add_bounded_int(&mut self, value: i64);
    fn add_bool(&mut self, value: bool);
    fn add_int(&mut self, value: i64);
    fn add_float(&mut self, value: f64);
    fn finish(self) -> V;
}
