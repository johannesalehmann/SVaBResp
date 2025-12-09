use crate::regions::{
    InvertedStateRegion, OrderedVectorStateRegion, StateRegion, VectorStateRegion,
};
use probabilistic_models::probabilistic_properties::Path;
use probabilistic_models::{
    AtomicProposition, Predecessors, ProbabilisticModel, VectorPredecessors,
};

mod attractor;

pub fn winning_region<M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors>>(
    model: &ProbabilisticModel<M>,
    property: probabilistic_models::probabilistic_properties::Property<AtomicProposition, f64>,
) -> InvertedStateRegion<OrderedVectorStateRegion> {
    todo!()
    // match property.path {
    //     Path::Eventually(ap) => safety_winning_region(model, ap),
    // }
}

pub fn safety_winning_region<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors>,
>(
    model: &ProbabilisticModel<M>,
    bad_states_ap: AtomicProposition,
) -> InvertedStateRegion<OrderedVectorStateRegion> {
    let target = model.get_states_with_ap(bad_states_ap);

    let attractor: VectorStateRegion = attractor::attractor(model, target.iter().cloned()); // TODO: Use more efficient region data structure

    attractor.sorted().inverted()
}

pub fn safety_is_winning<M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors>>(
    model: &ProbabilisticModel<M>,
    bad_states_ap: AtomicProposition,
    state: usize,
) -> bool {
    let region = safety_winning_region(model, bad_states_ap);
    region.is_set(state)
}

pub fn reachability_winning_region<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors>,
>(
    model: &ProbabilisticModel<M>,
    target_states_ap: AtomicProposition,
) -> VectorStateRegion {
    let target = model.get_states_with_ap(target_states_ap);

    attractor::attractor(model, target.iter().cloned())
}

pub fn reachability_is_winning<
    M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors>,
>(
    model: &ProbabilisticModel<M>,
    bad_states_ap: AtomicProposition,
    state: usize,
) -> bool {
    let region = reachability_winning_region(model, bad_states_ap);
    region.is_set(state)
}
