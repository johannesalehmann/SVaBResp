use probabilistic_models::{
    ActionVector, AtomicProposition, DistributionVector, ModelTypes, ProbabilisticModel,
    SinglePlayer, VectorPredecessors,
};
use probabilistic_properties::{NonDeterminismKind, PathFormula, Query, StateFormula};

pub fn check_mdp<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = SinglePlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    query: Query<i64, f64, AtomicProposition>,
) -> Result<f64, super::CheckerError> {
    if let Query::ProbabilityValue {
        non_determinism: Some(non_determinism),
        path: PathFormula::Eventually { condition },
    } = query
    {
        if let StateFormula::Expression(ap) = *condition {
            if non_determinism == NonDeterminismKind::Minimise {
                probabilistic_model_algorithms::value_iteration::mdp::optimistic_value_iteration_minimise(
                    model, ap.index, 0.000_001,
                );
            } else if non_determinism == NonDeterminismKind::Maximise {
                probabilistic_model_algorithms::value_iteration::mdp::optimistic_value_iteration_maximise(
                    model, ap.index, 0.000_001,
                );
            }
        }
    }

    Err(super::CheckerError::NoSuitableAlgorithm)
}
