use crate::{Action, ModelTypes};

pub struct ActionVector<M: ModelTypes> {
    actions: Vec<Action<M>>,
}

impl<M: ModelTypes> super::ActionCollection<M> for ActionVector<M> {
    type Builder = Builder<M>;
    type Iter<'a>
        = std::slice::Iter<'a, Action<M>>
    where
        M: 'a;
    type IntoIter = std::vec::IntoIter<Action<M>>;

    fn get_builder() -> Self::Builder {
        Builder::new()
    }

    fn get_number_of_actions(&self) -> usize {
        self.actions.len()
    }

    fn get_action(&self, index: usize) -> &Action<M> {
        &self.actions[index]
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.actions.iter()
    }

    fn into_iter(self) -> Self::IntoIter {
        self.actions.into_iter()
    }
}

pub struct Builder<M: ModelTypes> {
    actions: ActionVector<M>,
}

impl<M: ModelTypes> Builder<M> {
    pub fn new() -> Self {
        Self {
            actions: ActionVector {
                actions: Vec::new(),
            },
        }
    }
}

impl<M: ModelTypes> super::Builder<ActionVector<M>, M> for Builder<M> {
    fn add_action(&mut self, action: Action<M>) {
        self.actions.actions.push(action)
    }

    fn finish(self) -> ActionVector<M> {
        self.actions
    }
}
