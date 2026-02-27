use probabilistic_models::{ModelTypes, ProbabilisticModel};

mod map;
pub use map::*;

mod vec_of_vecs;
pub use vec_of_vecs::*;

pub trait BuildableScc {
    type BuilderType: SccBuilder<Self>;

    fn builder<M: ModelTypes>(model: &ProbabilisticModel<M>) -> Self::BuilderType;
}

pub trait SccBuilder<S: ?Sized> {
    fn add_scc(&mut self) -> usize;
    fn add_to_scc(&mut self, state_index: usize, scc_index: usize);
    fn mark_non_trivial(&mut self, scc_index: usize);

    fn finish(self) -> S;
}
