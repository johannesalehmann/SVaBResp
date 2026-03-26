use probabilistic_models::{
    ActionVector, AtomicProposition, ModelTypes, ProbabilisticModel, SingleStateDistribution,
    TwoPlayer, VectorPredecessors,
};
use probabilistic_properties::Query;

pub fn check_nonstochastic_game<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = SingleStateDistribution,
            ActionCollection = ActionVector<SingleStateDistribution>,
            Owners = TwoPlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    query: Query<i64, f64, AtomicProposition>,
) -> Result<f64, super::CheckerError> {
    let _ = (model, query);
    Err(super::CheckerError::NoSuitableAlgorithm)
}
