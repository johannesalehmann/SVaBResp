mod distribution_vector;
pub use distribution_vector::DistributionVector;

mod single_state_distribution;
use crate::Action;
pub use single_state_distribution::SingleStateDistribution;

pub trait Distribution: Sized {
    type Builder: DistributionBuilder<Self>;
    type Iter<'a>: Iterator<Item = &'a Successor>
    where
        Self: 'a;
    fn get_builder() -> Self::Builder;
    fn number_of_successors(&self) -> usize;
    fn get_successor(&self, index: usize) -> Successor;
    fn iter<'a>(&'a self) -> Self::Iter<'a>;
}

#[derive(Copy, Clone)]
pub struct Successor {
    pub index: usize,
    pub probability: f64,
}

pub trait DistributionBuilder<D> {
    fn add_successor(&mut self, successor: Successor);
    fn finish(self) -> D;
}
