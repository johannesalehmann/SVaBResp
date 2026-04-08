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
use probabilistic_models::{
    AtomicProposition, IterFunctions, IterProbabilisticModel, Mdp, ModelTypes, ProbabilisticModel,
    VectorPredecessors,
};

pub fn check<M: ModelTypes>(
    model: ProbabilisticModel<M>,
    query: probabilistic_properties::Query<i64, f64, AtomicProposition>,
) -> Result<f64, CheckerError> {
    let features = model.get_model_features();

    // if features.representable_as_transition_system() {
    //     let ts: TransitionSystem<
    //         M::Predecessors,
    //         M::Valuation,
    //         M::AtomicPropositions,
    //         M::InitialStates,
    //     > = model.into_iter().map_owners(|_| ()).collect();
    //     let ts: TransitionSystem<
    //         VectorPredecessors,
    //         M::Valuation,
    //         M::AtomicPropositions,
    //         M::InitialStates,
    //     > = ts.rebuild_and_transform_predecessors();
    //
    //     let result = check_transition_system(ts, &query);
    //
    //     let model = match result {
    //         Ok(result) => return Ok(result),
    //         Err((CheckerError::NoSuitableAlgorithm)) => (),
    //     };
    // };
    if features.representable_as_markov_decision_process() {
        let mdp: Mdp<M::Predecessors, M::Valuation, M::AtomicPropositions, M::InitialStates> =
            model.into_iter().map_owners(|_| ()).collect();
        let mdp: Mdp<VectorPredecessors, M::Valuation, M::AtomicPropositions, M::InitialStates> =
            mdp.rebuild_and_transform_predecessors();

        let result = check_mdp(mdp, query);

        match result {
            Ok(result) => return Ok(result),
            Err(CheckerError::NoSuitableAlgorithm) => (),
        };
    }
    Err(CheckerError::NoSuitableAlgorithm)
}
