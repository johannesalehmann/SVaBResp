use probabilistic_models::{
    AtomicProposition, DistributionVector, ModelTypes, ProbabilisticModel, SingleAction,
    SinglePlayer, VectorPredecessors,
};
use probabilistic_properties::Query;

pub fn check_markov_chain<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = SingleAction<DistributionVector>,
            Owners = SinglePlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    query: Query<i64, f64, AtomicProposition>,
) -> Result<f64, super::CheckerError> {
    let _ = (model, query);
    Err(super::CheckerError::NoSuitableAlgorithm)
}
