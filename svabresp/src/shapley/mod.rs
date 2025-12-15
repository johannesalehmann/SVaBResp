mod algorithms;
pub use algorithms::*;

mod auxiliary;

mod coop_game;
pub use coop_game::{
    CoalitionSpecifier, CooperativeGame, MinimalCoalitionCache, MonotoneCooperativeGame,
    SimpleCooperativeGame,
};

mod responsibility_values;

pub trait ShapleyAlgorithm {
    type Output;

    fn compute<G: CooperativeGame>(&mut self, game: &mut G) -> Self::Output;
    fn compute_simple<G: SimpleCooperativeGame>(&mut self, game: &mut G) -> Self::Output {
        self.compute(game)
    }
}
