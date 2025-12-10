use crate::{Action, Distribution, ModelTypes};

pub struct ActionVector<D: Distribution> {
    actions: Vec<Action<D>>,
}

impl<D: Distribution> super::ActionCollection<D> for ActionVector<D> {
    type Builder = Builder<D>;
    type Iter<'a>
        = std::slice::Iter<'a, Action<D>>
    where
        D: 'a;
    type IntoIter = std::vec::IntoIter<Action<D>>;

    fn get_builder() -> Self::Builder {
        Builder::new()
    }

    fn get_number_of_actions(&self) -> usize {
        self.actions.len()
    }

    fn get_action(&self, index: usize) -> &Action<D> {
        &self.actions[index]
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.actions.iter()
    }

    fn into_iter(self) -> Self::IntoIter {
        self.actions.into_iter()
    }
}

pub struct Builder<D: Distribution> {
    actions: ActionVector<D>,
}

impl<D: Distribution> Builder<D> {
    pub fn new() -> Self {
        Self {
            actions: ActionVector {
                actions: Vec::new(),
            },
        }
    }
}

impl<D: Distribution> super::Builder<ActionVector<D>, D> for Builder<D> {
    fn add_action(&mut self, action: Action<D>) {
        self.actions.actions.push(action)
    }

    fn finish(self) -> ActionVector<D> {
        self.actions
    }
}
