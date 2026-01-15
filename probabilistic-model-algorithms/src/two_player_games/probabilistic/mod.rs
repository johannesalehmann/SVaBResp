use probabilistic_models::{ProbabilisticModel, TwoPlayer};

pub fn strategy_iteration<M: probabilistic_models::ModelTypes<Owners = TwoPlayer>>(
    model: &ProbabilisticModel<M>,
    objective_ap_index: usize,
) {
    let _ = (model, objective_ap_index);
    unimplemented!();
}
