use probabilistic_models::{AtomicProposition, MdpType, ProbabilisticModel};
use probabilistic_properties::{
    Path, ProbabilityConstraint, ProbabilityKind, ProbabilityOperator, Property,
};

pub fn check(model: &ProbabilisticModel<MdpType>, property: &Property<AtomicProposition, f64>) {
    match (&property.operator, &property.path) {
        (
            ProbabilityOperator {
                kind: ProbabilityKind::PMax,
                constraint: ProbabilityConstraint::ValueOf,
            },
            Path::Eventually(AtomicProposition { index }),
        ) => {
            probabilistic_model_algorithms::mdp::optimistic_value_iteration(
                &model, *index, 0.000_001,
            );
        }
        _ => panic!("This combination of operator and path formula is not supported"),
    }
}
