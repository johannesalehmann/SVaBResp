mod buffer;
pub use buffer::AttractorBuffer;

mod full_region;
pub use full_region::{attractor, attractor_with_buffer};

mod state_included;
pub use state_included::{attractor_contains_state, attractor_contains_state_with_buffer};

use probabilistic_models::{Predecessors, ProbabilisticModel, TwoPlayer, VectorPredecessors};

trait AttractorCondition {
    type Output;

    fn state_attracted(&mut self, index: usize) -> Option<Self::Output>;
    fn result_after_termination(self) -> Self::Output;
}

fn attractor_internal<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>, // TODO: Make this generic over different predecessor collections
    R1: Iterator<Item = usize>,
    C: AttractorCondition,
>(
    model: &ProbabilisticModel<M>,
    region: R1,
    condition: C,
    attracted_player: TwoPlayer,
) -> C::Output {
    let mut buffer = AttractorBuffer::create(&model);
    buffer.reset_owner_counts(model, attracted_player);
    attractor_internal_with_buffer(model, region, condition, &mut buffer)
}

// When calling this method, ensure that owner counts in the buffer are up-to-date, either by calling .reset_owner_counts(...) on the buffer or by setting them manually.
fn attractor_internal_with_buffer<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>, // TODO: Make this generic over different predecessor collections
    R1: Iterator<Item = usize>,
    C: AttractorCondition,
>(
    model: &ProbabilisticModel<M>,
    region: R1,
    mut condition: C,
    buffer: &mut AttractorBuffer,
) -> C::Output {
    buffer.open_list.clear();

    for state in region {
        if let Some(result) = condition.state_attracted(state) {
            return result;
        }
        buffer.open_list.push(state);
        buffer.set_value(state, 0);
    }

    while let Some(next) = buffer.open_list.pop() {
        let state = &model.states[next];
        for predecessor in state.predecessors.iter() {
            let count = buffer.get_value(predecessor.from);
            if count > 0 {
                buffer.set_value(predecessor.from, count - 1);
                if count == 1 {
                    if let Some(result) = condition.state_attracted(predecessor.from) {
                        return result;
                    }
                    buffer.open_list.push(predecessor.from);
                }
            }
        }
    }

    condition.result_after_termination()
}
