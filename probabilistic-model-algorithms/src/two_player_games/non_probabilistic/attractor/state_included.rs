use super::{AttractorBuffer, AttractorCondition};
use probabilistic_models::{ProbabilisticModel, TwoPlayer, VectorPredecessors};

struct StateIncludedCondition {
    state: usize,
}

impl StateIncludedCondition {
    pub fn new(state: usize) -> Self {
        Self { state }
    }
}

impl AttractorCondition for StateIncludedCondition {
    type Output = bool;

    fn state_attracted(&mut self, index: usize) -> Option<Self::Output> {
        if index == self.state {
            Some(true)
        } else {
            None
        }
    }

    fn result_after_termination(self) -> Self::Output {
        false
    }
}

pub fn attractor_contains_state<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    R1: Iterator<Item = usize>,
>(
    model: &ProbabilisticModel<M>,
    region: R1,
    state: usize,
    attracted_player: TwoPlayer,
) -> bool {
    super::attractor_internal(
        model,
        region,
        StateIncludedCondition::new(state),
        attracted_player,
    )
}

pub fn attractor_contains_state_with_buffer<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    R1: Iterator<Item = usize>,
>(
    model: &ProbabilisticModel<M>,
    region: R1,
    state: usize,
    buffer: &mut AttractorBuffer,
) -> bool {
    super::attractor_internal_with_buffer(model, region, StateIncludedCondition::new(state), buffer)
}
