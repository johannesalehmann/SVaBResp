mod objective;
pub use objective::{Buechi, Objective, Reachability, Safety};

mod value_objective;
pub use value_objective::{ReachabilityValue, ValueObjective};

mod solvable_game;
pub use solvable_game::{SolvableGame, SolvableValueGame};

mod winning_region;
pub use winning_region::WinningRegion;

use probabilistic_models::annotations::AtomicPropositions;
use probabilistic_models::base_model::TwoPlayerTurnBasedGame;
use probabilistic_models::initial_states::SingleInitialState;
use probabilistic_models::labels::Labels;
use probabilistic_models::predecessors::Predecessors;
use probabilistic_models::valuations::Valuations;

pub type StateIdx = probabilistic_models::StateIndex<u32>;
pub type ChoiceIdx = probabilistic_models::ChoiceIndex<u32>;
pub type BranchIdx = probabilistic_models::BranchIndex<u32>;
pub type PredecessorIdx = probabilistic_models::PredecessorIndex<usize>;
pub type ApIdx = probabilistic_models::AtomicPropositionIndex<usize>;
pub type ApEntryIdx = probabilistic_models::AnnotationEntryIndex<usize>;
pub type ActionIdx = probabilistic_models::ChoiceLabelIndex<usize>;
pub type ClassIdx = probabilistic_models::ValuationClassIndex<u16>;
pub type ClassEntryIdx = probabilistic_models::ValuationClassEntryIndex<u16>;
pub type ValuationIdx = probabilistic_models::ValuationIndex<usize>;

pub type GameValuations = Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx>;

pub type GameAtomicPropositions = AtomicPropositions<ApIdx, StateIdx, ApEntryIdx>;

pub type GameChoiceLabels = Labels<ChoiceIdx, ActionIdx, Option<String>>;

pub type GamePredecessors = Predecessors<StateIdx, ChoiceIdx, BranchIdx, PredecessorIdx>;

pub type BuiltModel = probabilistic_models::Model<
    probabilistic_models::base_model::Mdp<StateIdx, ChoiceIdx, BranchIdx>,
    SingleInitialState<StateIdx>,
    GameChoiceLabels,
    (),
    (),
    GameAtomicPropositions,
    (),
    (),
    GameValuations,
    (),
>;

pub type Game = probabilistic_models::Model<
    TwoPlayerTurnBasedGame<StateIdx, ChoiceIdx, BranchIdx>,
    SingleInitialState<StateIdx>,
    GameChoiceLabels,
    (),
    (),
    GameAtomicPropositions,
    (),
    (),
    GameValuations,
    GamePredecessors,
>;

// TODO: Use feature detection from tiny-pmc once it provides such a feature, instead of rolling
//  it on our own.
pub fn is_probabilistic<M: probabilistic_models::traits::ReadStateSpace>(model: &M) -> bool {
    model
        .choices()
        .into_iter()
        .any(|choice| model.branches_of_choice(choice).len() > 1)
}

pub fn single_valuation_class(valuations: &GameValuations) -> ClassIdx {
    let mut classes = valuations.classes().into_iter();
    let class = classes
        .next()
        .expect("the model does not have any valuation class");
    assert!(
        classes.next().is_none(),
        "grouping by variable values requires a model whose states all share one valuation class"
    );
    class
}

pub fn variable_index(valuations: &GameValuations, name: &str) -> Option<ClassEntryIdx> {
    valuations
        .class(single_valuation_class(valuations))
        .index_by_name(name)
}
