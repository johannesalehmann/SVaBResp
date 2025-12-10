mod coop_game;
pub use coop_game::{CoalitionSpecifier, CooperativeGame, SimpleCooperativeGame};

pub trait ShapleyAlgorithm {
    type Output;

    fn compute<G: CooperativeGame>(&mut self, game: &G) -> Self::Output;
    fn compute_simple<G: SimpleCooperativeGame>(&mut self, game: &G) -> Self::Output {
        self.compute(game)
    }
}
