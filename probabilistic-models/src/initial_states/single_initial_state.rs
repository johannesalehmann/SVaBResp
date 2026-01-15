use super::{InitialStates, InitialStatesBuilder};

pub struct SingleInitialState {
    initial_state: usize,
}

impl InitialStates for SingleInitialState {
    type Builder = SingleInitialStateBuilder;
    type Iter<'a>
        = std::iter::Once<&'a usize>
    where
        Self: 'a;

    fn get_builder() -> Self::Builder {
        SingleInitialStateBuilder { state: None }
    }

    fn count(&self) -> usize {
        1
    }

    fn get(&self, index: usize) -> usize {
        if index == 0 {
            self.initial_state
        } else {
            panic!(
                "Initial state index {} out of range (number of initial states: {})",
                index, 1
            );
        }
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        std::iter::once(&self.initial_state)
    }
}

pub struct SingleInitialStateBuilder {
    state: Option<usize>,
}

impl SingleInitialStateBuilder {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl InitialStatesBuilder<SingleInitialState> for SingleInitialStateBuilder {
    fn add_by_index(&mut self, index: usize) {
        if self.state.is_none() {
            self.state = Some(index);
        } else {
            panic!("Model type does not support multiple initial states");
        }
    }

    fn finish(self) -> SingleInitialState {
        if let Some(index) = self.state {
            SingleInitialState {
                initial_state: index,
            }
        } else {
            panic!("Model type requires at least one initial state");
        }
    }
}
