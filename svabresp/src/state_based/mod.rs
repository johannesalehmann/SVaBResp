use probabilistic_models::{
    IterFunctions, IterProbabilisticModel, MdpType, SingleStateDistribution, TwoPlayer,
    VectorPredecessors,
};

mod game;
pub mod grouping;

use crate::shapley::{MinimalCoalitionCache, ShapleyAlgorithm};
use crate::{PrismModel, PrismProperty};

use grouping::GroupExtractionScheme;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::{
    AlgorithmCollection, GameAndSolverExternalOwners, ReachabilityAlgorithmCollection,
    SafetyAlgorithmCollection,
};

pub fn compute_for_prism<G: GroupExtractionScheme, S: ShapleyAlgorithm>(
    mut prism_model: PrismModel,
    mut prism_property: PrismProperty,
    mut grouping_scheme: G,
    shapley: &mut S,
) -> S::Output<String> {
    let constants = std::collections::HashMap::new();

    grouping_scheme.transform_prism(&mut prism_model, &mut prism_property);

    let mut atomic_propositions = Vec::new();
    let properties = tiny_pmc::building::prism_objectives_to_atomic_propositions(
        &mut atomic_propositions,
        vec![prism_property],
    );
    let properties =
        prism_model_builder::build_properties(&prism_model, properties.into_iter(), &constants)
            .unwrap();

    assert_eq!(properties.len(), 1);
    let mut property = properties.into_iter().nth(0).unwrap();

    let model = prism_model_builder::build_model::<_, MdpType<VectorPredecessors>>(
        &prism_model,
        &atomic_propositions[..],
        &constants,
    )
    .unwrap();

    let mut game: probabilistic_models::TwoPlayerNonstochasticGame<VectorPredecessors> = model
        .into_iter()
        .map_owners(|_| TwoPlayer::PlayerTwo)
        .collect();

    let grouping = grouping_scheme.create_groups(&mut game, &mut property);

    if let Some(solver) = ReachabilityAlgorithmCollection::create_if_compatible(&property) {
        let solvable_game = GameAndSolverExternalOwners::new(game, solver);
        let coop_game = game::StateBasedResponsibilityGame::new(solvable_game, grouping);
        let mut cached_coop_game = MinimalCoalitionCache::create(coop_game);

        shapley.compute_simple(cached_coop_game)
    } else if let Some(solver) = SafetyAlgorithmCollection::create_if_compatible(&property) {
        let solvable_game = GameAndSolverExternalOwners::new(game, solver);
        let coop_game = game::StateBasedResponsibilityGame::new(solvable_game, grouping);
        let mut cached_coop_game = MinimalCoalitionCache::create(coop_game);

        shapley.compute_simple(cached_coop_game)
    } else {
        panic!("Unsupported property type");
    }
}
