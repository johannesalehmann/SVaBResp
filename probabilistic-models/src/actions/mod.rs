mod action_vector;
pub use action_vector::ActionVector;

mod single_action;
pub use single_action::SingleAction;

use crate::{Action, Distribution, ModelTypes};

pub trait ActionCollection<D: Distribution>: Sized {
    type Builder: Builder<Self, D>;
    type Iter<'a>: Iterator<Item = &'a Action<D>>
    where
        D: 'a,
        Self: 'a;
    type IntoIter: Iterator<Item = Action<D>>;
    fn get_builder() -> Self::Builder;
    fn get_number_of_actions(&self) -> usize;
    fn get_action(&self, index: usize) -> &super::Action<D>;

    fn iter<'a>(&'a self) -> Self::Iter<'a>;
    fn into_iter(self) -> Self::IntoIter;
}

pub trait Builder<A, D: Distribution> {
    fn add_action(&mut self, action: Action<D>);
    fn finish(self) -> A;
}
