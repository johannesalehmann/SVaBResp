use crate::{Action, Distribution, ModelTypes};

pub struct SingleAction<D: Distribution> {
    action: Action<D>,
}

impl<D: Distribution> super::ActionCollection<D> for SingleAction<D> {
    type Builder = Builder<D>;
    type Iter<'a>
        = std::iter::Once<&'a Action<D>>
    where
        D: 'a;
    type IntoIter = std::iter::Once<Action<D>>;

    fn get_builder() -> Self::Builder {
        Builder::new()
    }

    fn get_number_of_actions(&self) -> usize {
        1
    }

    fn get_action(&self, index: usize) -> &Action<D> {
        if index != 0 {
            panic!("Action index out of bounds");
        }
        &self.action
    }
    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        std::iter::once(&self.action)
    }
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.action)
    }
}

pub struct Builder<D: Distribution> {
    action: Option<Action<D>>,
}

impl<D: Distribution> Builder<D> {
    pub fn new() -> Self {
        Self { action: None }
    }
}

impl<D: Distribution> super::Builder<SingleAction<D>, D> for Builder<D> {
    fn add_action(&mut self, action: Action<D>) {
        match &self.action {
            None => self.action = Some(action),
            Some(_) => panic!("Cannot add a second action to a state of this model type"),
        }
    }

    fn finish(self) -> SingleAction<D> {
        match self.action {
            Some(action) => SingleAction { action },
            None => panic!("Must add at least one action to each state in this model type"),
        }
    }
}
