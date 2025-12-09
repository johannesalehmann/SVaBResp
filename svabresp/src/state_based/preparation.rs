use crate::shapley::CooperativeGame;
use crate::{PrismModel, PrismProperty};

use super::grouping::{GroupExtractionScheme, VectorStateGroups};
use crate::state_based::game::StateBasedResponsibilityGame;
use probabilistic_models::{
    IterFunctions, IterProbabilisticModel, ProbabilisticModel, TwoPlayer,
    TwoPlayerStochasticGameType,
};

pub fn prepare_from_prism<G: GroupExtractionScheme>(
    mut prism_model: PrismModel,
    mut prism_property: PrismProperty,
    mut grouping_scheme: G,
) -> StateBasedResponsibilityGame<TwoPlayerStochasticGameType, G::GroupType> {
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

    let model =
        prism_model_builder::build_model(&prism_model, &atomic_propositions[..], &constants)
            .unwrap();

    let mut game = model
        .into_iter()
        .map_owners(|_| TwoPlayer::PlayerOne)
        .collect();

    let grouping = grouping_scheme.create_groups(&mut game, &mut property);

    super::game::StateBasedResponsibilityGame::new(game, grouping)
}
