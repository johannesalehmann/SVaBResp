use super::StateIdx;
use probabilistic_models::typed_index_collections::To1;

pub struct WinningRegion {
    states: To1<StateIdx, bool>,
    size: usize,
}

impl WinningRegion {
    pub fn new(states: To1<StateIdx, bool>) -> Self {
        let size = states.true_values().into_iter().count();
        Self { states, size }
    }

    pub fn contains(&self, state: StateIdx) -> bool {
        self.states[state]
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn model_state_count(&self) -> usize {
        self.states.len()
    }

    pub fn states(&self) -> &To1<StateIdx, bool> {
        &self.states
    }
}
