use crate::state_based::grouping::StateGroups;
use probabilistic_models::{
    IterFunctions, IterProbabilisticModel, ModelTypes, ProbabilisticModel, TwoPlayer,
};

mod game;
pub mod grouping;
mod preparation;

pub use preparation::prepare_from_prism;
