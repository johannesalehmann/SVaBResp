use crate::shapley::CooperativeGame;
use crate::state_based::grouping::StateGroups;
use probabilistic_models::{
    IterFunctions, ModelTypes, ProbabilisticModel, SingleStateDistribution, TwoPlayer,
};

pub struct StateBasedResponsibilityGame<M: ModelTypes<Owners = TwoPlayer>, G: StateGroups> {
    game: ProbabilisticModel<M>,
    grouping: G,
}

impl<M: ModelTypes<Owners = TwoPlayer>, G: StateGroups> StateBasedResponsibilityGame<M, G> {
    pub fn new(game: ProbabilisticModel<M>, grouping: G) -> Self {
        Self { game, grouping }
    }
}

impl<M: ModelTypes<Distribution = SingleStateDistribution, Owners = TwoPlayer>, G: StateGroups>
    CooperativeGame for StateBasedResponsibilityGame<M, G>
{
    fn get_player_count(&self) -> usize {
        self.grouping.get_count()
    }

    fn get_value<C: crate::shapley::CoalitionSpecifier>(&mut self, coalition: C) -> f64 {
        todo!()
    }
}
