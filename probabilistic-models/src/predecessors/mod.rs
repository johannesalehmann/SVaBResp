mod non_tracked_predecessors;
pub use non_tracked_predecessors::{NonTrackedPredecessors, NonTrackedPredecessorsBuilder};

mod vector_predecessors;
pub use vector_predecessors::{VectorPredecessors, VectorPredecessorsBuilder};

#[derive(Clone)]
pub struct Predecessor {
    pub from: usize,
    pub action_index: usize,
    pub probability: f64,
}

pub trait Predecessors: Sized {
    type Iter<'a>: Iterator<Item = Predecessor>
    where
        Self: 'a;
    type Builder: PredecessorsBuilder<Self>;
    fn iter<'a>(&'a self) -> Self::Iter<'a>;
}

pub trait PredecessorsBuilder<P> {
    fn create() -> Self;
    fn add(&mut self, predecessor: Predecessor);
    fn finish(self) -> P;
}
