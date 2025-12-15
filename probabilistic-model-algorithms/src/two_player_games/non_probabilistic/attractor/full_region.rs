use super::{AttractorBuffer, AttractorCondition};
use crate::regions::MutableStateRegion;
use probabilistic_models::{ProbabilisticModel, TwoPlayer, VectorPredecessors};

struct FullRegionAttractorCondition<R: MutableStateRegion> {
    result: R,
}

impl<R: MutableStateRegion> FullRegionAttractorCondition<R> {
    pub fn new(model_size: usize) -> Self {
        Self {
            result: R::create(model_size),
        }
    }
}

impl<R: MutableStateRegion> AttractorCondition for FullRegionAttractorCondition<R> {
    type Output = R;

    fn state_attracted(&mut self, index: usize) -> Option<Self::Output> {
        self.result.add_state(index);
        None
    }

    fn result_after_termination(self) -> Self::Output {
        self.result
    }
}

pub fn attractor<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    R1: Iterator<Item = usize>,
    R2: MutableStateRegion,
>(
    model: &ProbabilisticModel<M>,
    region: R1,
) -> R2 {
    super::attractor_internal(
        model,
        region,
        FullRegionAttractorCondition::new(model.states.len()),
    )
}

pub fn attractor_with_buffer<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    R1: Iterator<Item = usize>,
    R2: MutableStateRegion,
>(
    model: &ProbabilisticModel<M>,
    region: R1,
    buffer: &mut AttractorBuffer,
) -> R2 {
    super::attractor_internal_with_buffer(
        model,
        region,
        FullRegionAttractorCondition::new(model.states.len()),
        buffer,
    )
}
