use crate::{Action, ModelTypes};

pub struct SingleAction<M: ModelTypes> {
    action: Action<M>,
}

impl<M: ModelTypes> super::ActionCollection<M> for SingleAction<M> {
    type Builder = Builder<M>;

    fn get_builder() -> Self::Builder {
        Builder::new()
    }

    fn get_number_of_actions(&self) -> usize {
        1
    }

    fn get_action(&self, index: usize) -> &Action<M> {
        if index != 0 {
            panic!("Action index out of bounds");
        }
        &self.action
    }
}

pub struct Builder<M: ModelTypes> {
    action: Option<Action<M>>,
}

impl<M: ModelTypes> Builder<M> {
    pub fn new() -> Self {
        Self { action: None }
    }
}

impl<M: ModelTypes> super::Builder<SingleAction<M>, M> for Builder<M> {
    fn add_action(&mut self, action: Action<M>) {
        match &self.action {
            None => self.action = Some(action),
            Some(_) => panic!("Cannot add a second action to a state of this model type"),
        }
    }

    fn finish(self) -> SingleAction<M> {
        match self.action {
            Some(action) => SingleAction { action },
            None => panic!("Must add at least one action to each state in this model type"),
        }
    }
}
