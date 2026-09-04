use log::{info, trace};

pub mod game;

mod nonstochastic_game;
pub use nonstochastic_game::StateBasedResponsibilityNonstochasticGame;

pub mod grouping;

mod group_names;
pub use group_names::GroupNames;

pub mod refinement;

use crate::shapley::{MinimalCoalitionCache, ShapleyAlgorithm, SwitchingPairCollector};
use crate::state_based::game::{
    ApIdx, Buechi, Game, GamePredecessors, Objective, PredecessorIdx, Reachability, Safety,
    SolvableGame, StateIdx, is_probabilistic,
};
use crate::state_based::grouping::{GroupsAndAuxiliary, StateGroups, VectorStateGroups};
use crate::state_based::refinement::GroupBlockingProvider;
use crate::{PrismModel, PrismProperty};
use grouping::GroupExtractionScheme;
use prism_model_builder::UserProvidedConstValue;
use prism_parser::CharacterToLineMap;
use probabilistic_models::base_model::TwoPlayerTurnBasedGame;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::typed_index_collections::To1;
use probabilistic_properties::Query;

pub struct StateBasedOutput<O, G: StateGroups> {
    pub shapley_output: O,
    pub grouping: G,
}

pub fn compute_for_prism<
    G: GroupExtractionScheme,
    S: ShapleyAlgorithm,
    B: GroupBlockingProvider,
    SPC: SwitchingPairCollector,
>(
    mut prism_model: PrismModel,
    character_to_line_map: &CharacterToLineMap,
    mut prism_property: PrismProperty,
    grouping_scheme: &mut G,
    group_blocking_provider: B,
    shapley: &mut S,
    constants: std::collections::HashMap<String, UserProvidedConstValue>,
    switching_pair_collector: &mut SPC,
) -> StateBasedOutput<S::Output<String>, VectorStateGroups> {
    grouping_scheme.transform_prism(&mut prism_model, &mut prism_property, character_to_line_map);

    trace!("Building model");
    // TODO: In the following, we build all labels. This is required for some grouping schemes.
    //  However, it might be more efficient to first ask the scheme which labels need to be built
    //  and then pass this ifnromation on to the grouping scheme.
    let built = prism_model_builder::ModelBuilder::new_mdp_builder(&mut prism_model)
        .with_all_labels()
        .with_query(prism_property)
        .with_constants(constants)
        .build();

    let property = built.query;
    let game = into_game(built.model);

    trace!("Computing state groups");
    let (game, grouping) = grouping_scheme.create_groups(game, &property);
    info!("There are {} state groups", grouping.groups.get_count());

    if is_probabilistic(&game) {
        info!("Model exhibits probabilistic behaviour");
        todo!("responsibility computation for probabilistic models has not been migrated yet")
    }

    trace!("Transforming transition system into game");
    if let Some(objective) = Reachability::try_from_query(&property) {
        compute_responsibility(
            game,
            objective,
            grouping,
            group_blocking_provider,
            shapley,
            switching_pair_collector,
        )
    } else if let Some(objective) = Safety::try_from_query(&property) {
        compute_responsibility(
            game,
            objective,
            grouping,
            group_blocking_provider,
            shapley,
            switching_pair_collector,
        )
    } else if let Some(objective) = Buechi::try_from_query(&property) {
        compute_responsibility(
            game,
            objective,
            grouping,
            group_blocking_provider,
            shapley,
            switching_pair_collector,
        )
    } else {
        panic!("Unsupported property type")
    }
}

fn compute_responsibility<
    O: Objective,
    G: StateGroups,
    S: ShapleyAlgorithm,
    B: GroupBlockingProvider,
    SPC: SwitchingPairCollector,
>(
    game: Game,
    objective: O,
    grouping: GroupsAndAuxiliary<G>,
    group_blocking_provider: B,
    shapley: &mut S,
    switching_pair_collector: &mut SPC,
) -> StateBasedOutput<S::Output<String>, VectorStateGroups> {
    let solvable_game = SolvableGame::new(game, objective);
    let mut coop_game = StateBasedResponsibilityNonstochasticGame::new(
        solvable_game,
        grouping.groups,
        grouping.always_helping,
        grouping.always_adversarial,
    );

    let blocking = group_blocking_provider.compute_blocks(&mut coop_game);
    let mut coop_game = coop_game.map_grouping(|g| blocking.apply_to_grouping(g));

    let mut cached_coop_game = MinimalCoalitionCache::create(&mut coop_game);

    let shapley_output = shapley
        .compute_simple_with_switching_pairs(&mut cached_coop_game, switching_pair_collector);

    StateBasedOutput {
        shapley_output,
        grouping: coop_game.grouping.to_vector_state_groups(),
    }
}

// TODO: We add dummy owners here, but the Shapley algorithm never uses or modifies them (instead,
//  they are modified directly in the context of the solver algorithm). Perhaps it would be nicer
//  to keep an ownerless model throughout. The only thing missing for this is the ability to create
//  a game-solving context using a model that does not implement ReadOwners. Adding such a method to
//  the game-solving context should be possible (e.g. `.new_uninitialised(...)`)
fn into_game(model: game::BuiltModel) -> Game {
    let state_count = model.states().len();
    let base = TwoPlayerTurnBasedGame {
        base_mdp: model.base,
        owners: To1::with_entries(vec![TwoPlayer::Adam; state_count]),
    };

    probabilistic_models::Model {
        base,
        initial: model.initial,
        choice_labels: model.choice_labels,
        branch_labels: (),
        observations: (),
        atomic_propositions: model.atomic_propositions,
        rewards: (),
        annotations: (),
        state_valuations: model.state_valuations,
        predecessors: (),
    }
    .compute_predecessors::<PredecessorIdx>()
}
