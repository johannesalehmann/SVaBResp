use log::trace;
use probabilistic_models::{
    IterFunctions, IterProbabilisticModel, MdpType, TwoPlayer, VectorPredecessors,
};

mod game;
pub use game::StateBasedResponsibilityGame;

pub mod grouping;

pub mod refinement;

use crate::shapley::{MinimalCoalitionCache, ShapleyAlgorithm};
use crate::state_based::refinement::GroupBlockingProvider;
use crate::{PrismModel, PrismProperty};
use grouping::GroupExtractionScheme;
use prism_model_builder::ConstValue;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::{
    AlgorithmCollection, GameAndSolverExternalOwners, ReachabilityAlgorithmCollection,
    SafetyAlgorithmCollection,
};

pub fn compute_for_prism<
    G: GroupExtractionScheme,
    S: ShapleyAlgorithm,
    B: GroupBlockingProvider,
>(
    mut prism_model: PrismModel,
    mut prism_property: PrismProperty,
    mut grouping_scheme: G,
    group_blocking_provider: B,
    shapley: &mut S,
    constants: std::collections::HashMap<String, ConstValue>,
) -> S::Output<String> {
    trace!("Applying grouping scheme to PRISM model");
    grouping_scheme.transform_prism(&mut prism_model, &mut prism_property);

    trace!("Building atomic proposition list");
    let mut atomic_propositions = Vec::new();
    for label in &prism_model.labels.labels {
        atomic_propositions.push(label.condition.clone());
    }
    let properties = tiny_pmc::building::prism_objectives_to_atomic_propositions(
        &mut atomic_propositions,
        vec![prism_property],
    );

    trace!("Building properties");
    let properties =
        prism_model_builder::build_properties(&prism_model, properties.into_iter(), &constants)
            .unwrap();

    assert_eq!(properties.len(), 1);
    let property = properties.into_iter().nth(0).unwrap();

    trace!("Building model");
    let model = prism_model_builder::build_model::<_, MdpType<VectorPredecessors>>(
        &prism_model,
        &atomic_propositions[..],
        &constants,
    )
    .unwrap();

    trace!("Transforming transition system into game");
    let mut game: probabilistic_models::TwoPlayerNonstochasticGame<VectorPredecessors> = model
        .into_iter()
        .map_owners(|_| TwoPlayer::PlayerTwo)
        .collect();

    trace!("Computing state groups");
    let grouping = grouping_scheme.create_groups(&mut game, &property);

    if let Some(solver) = ReachabilityAlgorithmCollection::create_if_compatible(&property) {
        let solvable_game = GameAndSolverExternalOwners::new(game, solver);
        let mut coop_game = game::StateBasedResponsibilityGame::new(
            solvable_game,
            grouping.groups,
            grouping.always_helping,
            grouping.always_adversarial,
        );

        let blocking = group_blocking_provider.compute_blocks(&mut coop_game);

        let coop_game = coop_game.map_grouping(|g| blocking.apply_to_grouping(g));

        let cached_coop_game = MinimalCoalitionCache::create(coop_game);

        shapley.compute_simple(cached_coop_game)
    } else if let Some(solver) = SafetyAlgorithmCollection::create_if_compatible(&property) {
        let solvable_game = GameAndSolverExternalOwners::new(game, solver);
        let mut coop_game = game::StateBasedResponsibilityGame::new(
            solvable_game,
            grouping.groups,
            grouping.always_helping,
            grouping.always_adversarial,
        );

        let blocking = group_blocking_provider.compute_blocks(&mut coop_game);
        let coop_game = coop_game.map_grouping(|g| blocking.apply_to_grouping(g));

        let cached_coop_game = MinimalCoalitionCache::create(coop_game);

        shapley.compute_simple(cached_coop_game)
    } else {
        panic!("Unsupported property type");
    }
}
