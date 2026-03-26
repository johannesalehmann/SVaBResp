use probabilistic_models::{
    ActionVector, AtomicProposition, DistributionVector, ModelTypes, ProbabilisticModel, TwoPlayer,
    VectorPredecessors,
};
use probabilistic_properties::Query;

pub fn check_stochastic_game<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = TwoPlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    query: Query<i64, f64, AtomicProposition>,
) -> Result<f64, super::CheckerError> {
    let _ = (model, query);
    Err(super::CheckerError::NoSuitableAlgorithm)
}
