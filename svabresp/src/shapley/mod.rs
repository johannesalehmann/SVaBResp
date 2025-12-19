mod algorithms;
pub use algorithms::*;

mod auxiliary;

mod coop_game;
pub use coop_game::{
    CoalitionSpecifier, CooperativeGame, MinimalCoalitionCache, MonotoneCooperativeGame,
    PlayerDescriptions, SimpleCooperativeGame,
};

mod responsibility_values;

pub trait ShapleyAlgorithm {
    type Output<PD>;

    fn compute<G: CooperativeGame>(
        &mut self,
        game: G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType>;
    fn compute_simple<G: SimpleCooperativeGame>(
        &mut self,
        game: G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        self.compute(game)
    }
}
