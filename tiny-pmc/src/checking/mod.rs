use probabilistic_models::{AtomicProposition, MdpType, ProbabilisticModel};
use probabilistic_properties::{Operator, Path, Property};

pub fn check(model: &ProbabilisticModel<MdpType>, property: &Property<AtomicProposition>) {
    match (&property.operator, &property.path) {
        (Operator::ValueOfPMax, Path::Eventually(AtomicProposition { index })) => {
            probabilistic_model_algorithms::mdp::optimistic_value_iteration(
                &model, *index, 0.000_001,
            );
        }
        _ => panic!("This combination of operator and path formula is not supported"),
    }
}
