mod action_vector;
pub use action_vector::ActionVector;

mod single_action;
pub use single_action::SingleAction;

use crate::{Action, ModelTypes};

pub trait ActionCollection<M: ModelTypes>: Sized {
    type Builder: Builder<Self, M>;
    fn get_builder() -> Self::Builder;
    fn get_number_of_actions(&self) -> usize;
    fn get_action(&self, index: usize) -> &super::Action<M>;
}

pub trait Builder<A, M: ModelTypes> {
    fn add_action(&mut self, action: Action<M>);
    fn finish(self) -> A;
}
