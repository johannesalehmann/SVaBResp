use probabilistic_models::{Distribution, ProbabilisticModel, SingleStateDistribution};

pub fn is_reachable<M: probabilistic_models::ModelTypes<Distribution = SingleStateDistribution>>(
    model: &ProbabilisticModel<M>,
    objective_ap_index: usize,
)   {
    let open_list = Vec::new();
    for initial in model.states
}