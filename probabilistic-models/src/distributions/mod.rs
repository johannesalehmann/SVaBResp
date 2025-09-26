mod distribution_vector;
pub use distribution_vector::DistributionVector;

mod single_state_distribution;
pub use single_state_distribution::SingleStateDistribution;

pub trait Distribution: Sized {
    type Builder: DistributionBuilder<Self>;
    fn get_builder() -> Self::Builder;
    fn number_of_successors(&self) -> usize;
    fn get_successor(&self, index: usize) -> Successor;
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
