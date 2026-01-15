use probabilistic_models::{
    ActionCollection, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};

pub struct AttractorBuffer {
    counts: Vec<Count>,
    pub open_list: Vec<usize>,
}

struct Count {
    current: u32,
    default: u32,
}

impl Count {
    fn with_default(default: u32) -> Self {
        Self {
            current: 0,
            default,
        }
    }

    fn reset_reaching_player(&mut self) {
        self.current = 1;
    }
    fn reset_avoiding_player(&mut self) {
        self.current = self.default;
    }
}

impl AttractorBuffer {
    pub fn create<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        model: &ProbabilisticModel<M>,
    ) -> Self {
        let mut counts = Vec::with_capacity(model.states.len());
        for state in &model.states {
            counts.push(Count::with_default(
                state.actions.get_number_of_actions() as u32
            ))
        }
        Self {
            counts,
            open_list: Vec::new(),
        }
    }

    pub fn reset_owner_counts<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        reaching_player: TwoPlayer,
    ) {
        for (index, state) in model.states.iter().enumerate() {
            if state.owner == reaching_player {
                self.reset_reaching_player(index)
            } else {
                self.reset_avoiding_player(index)
            }
        }
    }

    pub fn reset_reaching_player(&mut self, index: usize) {
        self.counts[index].reset_reaching_player()
    }

    pub fn reset_avoiding_player(&mut self, index: usize) {
        self.counts[index].reset_avoiding_player()
    }

    pub fn get_value(&self, index: usize) -> u32 {
        self.counts[index].current
    }

    pub fn set_value(&mut self, index: usize, value: u32) {
        self.counts[index].current = value
    }
}
