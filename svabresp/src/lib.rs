pub use num_bigint;
pub use num_rational;
pub use num_traits;

pub mod shapley;
pub mod state_based;

pub use prism_model_builder::ConstValue;

use crate::shapley::CooperativeGame;
use chumsky::span::SimpleSpan;
use prism_model::{Expression, Identifier, VariableReference};
use probabilistic_models::{
    MdpType, ModelTypes, TwoPlayerStochasticGame, TwoPlayerStochasticGameType,
};

type PrismModel = prism_model::Model<(), Identifier<SimpleSpan>, VariableReference, SimpleSpan>;
type PrismProperty = probabilistic_properties::Property<
    Expression<VariableReference, SimpleSpan>,
    Expression<VariableReference, SimpleSpan>,
>;
