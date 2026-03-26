mod markov_chains;
pub use markov_chains::check_markov_chain;

mod stochastic_games;
pub use stochastic_games::check_stochastic_game;

mod transition_systems;
pub use transition_systems::check_transition_system;

mod markov_decision_processes;
pub use markov_decision_processes::check_mdp;

mod nonstochastic_games;
pub use nonstochastic_games::check_nonstochastic_game;

use crate::CheckerError;
use probabilistic_models::{AtomicProposition, ModelTypes, ProbabilisticModel};

pub fn check<M: ModelTypes>(
    model: ProbabilisticModel<M>,
    query: probabilistic_properties::Query<i64, f64, AtomicProposition>,
) -> Result<f64, CheckerError> {
    let features = model.get_model_features();

    if features.representable_as_transition_system() {
        // TODO
    }

    let _ = query;
    Err(CheckerError::NoSuitableAlgorithm)
}
