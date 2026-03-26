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
        if non_determinism == NonDeterminismKind::Minimise {
            panic!("PMin properties are currently not supported – but this should be easy to add");
        }
        if let StateFormula::Expression(ap) = *condition {
            probabilistic_model_algorithms::value_iteration::optimistic_value_iteration(
                model, ap.index, 0.000_001,
            );
        }
    }

    Err(super::CheckerError::NoSuitableAlgorithm)
}
