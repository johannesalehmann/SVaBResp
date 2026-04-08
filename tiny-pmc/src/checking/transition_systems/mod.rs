use probabilistic_models::{
    ActionVector, AtomicProposition, ModelTypes, ProbabilisticModel, SinglePlayer,
    SingleStateDistribution, VectorPredecessors,
};
use probabilistic_properties::Query;

pub fn check_transition_system<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = SingleStateDistribution,
            ActionCollection = ActionVector<SingleStateDistribution>,
            Owners = SinglePlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    query: &Query<i64, f64, AtomicProposition>,
) -> Result<f64, super::CheckerError> {
    let _ = (model, query);
    Err(super::CheckerError::NoSuitableAlgorithm)
}
