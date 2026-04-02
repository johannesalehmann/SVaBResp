use log::{info, trace};
use probabilistic_model_algorithms::traits::{StochasticGameAlgorithm, StochasticGameAndSolver};
use probabilistic_models::{
    IterFunctions, IterProbabilisticModel, MdpType, TwoPlayer, Valuation, VectorPredecessors,
};

mod nonstochastic_game;
pub use nonstochastic_game::StateBasedResponsibilityNonstochasticGame;

pub mod grouping;

mod group_names;
pub use group_names::GroupNames;

pub mod refinement;
mod stochastic_game;

use crate::shapley::{MinimalCoalitionCache, ShapleyAlgorithm, SwitchingPairCollector};
use crate::state_based::grouping::StateGroups;
use crate::state_based::refinement::GroupBlockingProvider;
use crate::{PrismModel, PrismProperty};
use grouping::GroupExtractionScheme;
use prism_model_builder::UserProvidedConstValue;
use probabilistic_model_algorithms::deterministic_games::{
    BuechiAlgorithmCollection, NonstochasticGameAlgorithm,
    NonstochasticGameAndSolverExternalOwners, ReachabilityAlgorithmCollection,
    SafetyAlgorithmCollection,
};
use probabilistic_model_algorithms::value_iteration::stochastic_games::StochasticGameValueIterationAlgorithm;

pub fn compute_for_prism<
    G: GroupExtractionScheme,
    S: ShapleyAlgorithm,
    B: GroupBlockingProvider,
    SPC: SwitchingPairCollector,
>(
    mut prism_model: PrismModel,
    mut prism_property: PrismProperty,
    grouping_scheme: &mut G,
    group_blocking_provider: B,
    shapley: &mut S,
    constants: std::collections::HashMap<String, UserProvidedConstValue>,
    switching_pair_collector: &mut SPC,
) -> S::Output<String> {
    let mut atomic_propositions = Vec::new();
    grouping_scheme.transform_prism(
        &mut prism_model,
        &mut prism_property,
        &mut atomic_propositions,
    );
    let properties = tiny_pmc::building::prism_objectives_to_atomic_propositions(
        &mut atomic_propositions,
        vec![prism_property],
    );
    trace!("Building model");
    let builder_results = prism_model_builder::build_model::<_, MdpType<VectorPredecessors>, _>(
        &mut prism_model,
        &atomic_propositions[..],
        properties.into_iter(),
        &constants,
    )
    .unwrap();

    let properties = builder_results.properties;
    assert_eq!(properties.len(), 1);
    let property = properties.into_iter().nth(0).unwrap();

    let model = builder_results.model;
    let features = model.get_model_features();
    if features.probabilism {
        info!("Model exhibits probabilistic behaviour");

        let mut game: probabilistic_models::TwoPlayerStochasticGame<VectorPredecessors> = model
            .into_iter()
            .map_owners(|_| TwoPlayer::PlayerTwo)
            .collect();

        let grouping = grouping_scheme.create_groups(&mut game, &property);

        if let Some(solver) = StochasticGameValueIterationAlgorithm::create_if_compatible(&property)
        {
            let solvable_game = StochasticGameAndSolver::new(game, solver);

            let coop_game = stochastic_game::StateBasedResponsibilityStochasticGame::new(
                solvable_game,
                grouping.groups,
                grouping.always_helping,
                grouping.always_adversarial,
            );

            // TODO: Support blocking?
            // let blocking = group_blocking_provider.compute_blocks(&mut coop_game);
            // let coop_game = coop_game.map_grouping(|g| blocking.apply_to_grouping(g));

            shapley.compute_with_switching_pairs(coop_game, switching_pair_collector)
        } else {
            panic!("Unsupported property type");
        }
    } else {
        trace!("Transforming transition system into game");
        let mut game: probabilistic_models::TwoPlayerNonstochasticGame<VectorPredecessors> = model
            .into_iter()
            .map_owners(|_| TwoPlayer::PlayerTwo)
            .collect();

        trace!("Computing state groups");
        let grouping = grouping_scheme.create_groups(&mut game, &property);
        info!("There are {} state groups", grouping.groups.get_count());
        let print_groups = false;
        if print_groups {
            println!("Group membership:");
            for group in 0..grouping.groups.get_count() {
                println!("  Group {}", group);
                for state in grouping.groups.get_states(group) {
                    println!(
                        "    {}",
                        game.states[state]
                            .valuation
                            .displayable(&game.valuation_context)
                    );
                }
            }
        }

        if let Some(solver) = ReachabilityAlgorithmCollection::create_if_compatible(&property) {
            let solvable_game = NonstochasticGameAndSolverExternalOwners::new(game, solver);
            let mut coop_game = nonstochastic_game::StateBasedResponsibilityNonstochasticGame::new(
                solvable_game,
                grouping.groups,
                grouping.always_helping,
                grouping.always_adversarial,
            );

            let blocking = group_blocking_provider.compute_blocks(&mut coop_game);

            let coop_game = coop_game.map_grouping(|g| blocking.apply_to_grouping(g));

            let cached_coop_game = MinimalCoalitionCache::create(coop_game);

            shapley.compute_simple_with_switching_pairs(cached_coop_game, switching_pair_collector)
        } else if let Some(solver) = SafetyAlgorithmCollection::create_if_compatible(&property) {
            let solvable_game = NonstochasticGameAndSolverExternalOwners::new(game, solver);
            let mut coop_game = nonstochastic_game::StateBasedResponsibilityNonstochasticGame::new(
                solvable_game,
                grouping.groups,
                grouping.always_helping,
                grouping.always_adversarial,
            );

            let blocking = group_blocking_provider.compute_blocks(&mut coop_game);
            let coop_game = coop_game.map_grouping(|g| blocking.apply_to_grouping(g));

            let cached_coop_game = MinimalCoalitionCache::create(coop_game);

            shapley.compute_simple_with_switching_pairs(cached_coop_game, switching_pair_collector)
        } else if let Some(solver) = BuechiAlgorithmCollection::create_if_compatible(&property) {
            let solvable_game = NonstochasticGameAndSolverExternalOwners::new(game, solver);
            let mut coop_game = nonstochastic_game::StateBasedResponsibilityNonstochasticGame::new(
                solvable_game,
                grouping.groups,
                grouping.always_helping,
                grouping.always_adversarial,
            );

            let blocking = group_blocking_provider.compute_blocks(&mut coop_game);
            let coop_game = coop_game.map_grouping(|g| blocking.apply_to_grouping(g));

            let cached_coop_game = MinimalCoalitionCache::create(coop_game);

            shapley.compute_simple_with_switching_pairs(cached_coop_game, switching_pair_collector)
        } else {
            panic!("Unsupported property type");
        }
    }
}
