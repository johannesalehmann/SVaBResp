use crate::regions::{MutableStateRegion, StateRegion};
use probabilistic_models::{Predecessors, ProbabilisticModel, VectorPredecessors};
use std::collections::{HashMap, HashSet};

pub fn attractor<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors>, // TODO: Make this generic over different predecessor collections
    R1: Iterator<Item = usize>,
    R2: MutableStateRegion, // TODO: Switch to hash-map based implementation
>(
    model: &ProbabilisticModel<M>,
    region: R1,
) -> R2 {
    let mut open = Vec::new();
    let mut all = HashSet::new();
    let mut result = R2::create(model.states.len());

    for state in region {
        open.push(state);
        all.insert(state);
        result.add_state(state);
    }

    while let Some(next) = open.pop() {
        let state = &model.states[next];
        for predecessor in state.predecessors.iter() {
            if !all.contains(&predecessor.from) {
                open.push(predecessor.from);
                all.insert(predecessor.from);
                result.add_state(predecessor.from);
            }
        }
    }

    result
}
