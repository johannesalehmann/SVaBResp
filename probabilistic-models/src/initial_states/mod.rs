mod single_initial_state;
pub use single_initial_state::{SingleInitialState, SingleInitialStateBuilder};

pub trait InitialStates: Sized {
    type Builder: InitialStatesBuilder<Self>;
    type Iter<'a>: Iterator<Item = &'a usize>
    where
        Self: 'a;
    fn get_builder() -> Self::Builder;
    fn count(&self) -> usize;
    fn get(&self, index: usize) -> usize;
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait InitialStatesBuilder<D> {
    fn add_by_index(&mut self, index: usize);

    fn finish(self) -> D;
}