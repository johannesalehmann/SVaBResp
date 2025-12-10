use crate::shapley::{CoalitionSpecifier, CooperativeGame, SimpleCooperativeGame};
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::AlgorithmCollection;
use probabilistic_models::TwoPlayer::PlayerTwo;
use probabilistic_models::{
    AtomicProposition, IterFunctions, ModelTypes, ProbabilisticModel, SingleStateDistribution,
    TwoPlayer, VectorPredecessors,
};
use probabilistic_properties::Property;
use std::marker::PhantomData;

pub struct StateBasedResponsibilityGame<
    M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>,
    G: StateGroups,
    A: AlgorithmCollection,
> {
    game: ProbabilisticModel<M>,
    grouping: G,
    algorithm_collection: A,
}

impl<
    'a,
    M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>,
    G: StateGroups,
    A: AlgorithmCollection,
> StateBasedResponsibilityGame<M, G, A>
{
    pub fn new(game: ProbabilisticModel<M>, grouping: G, algorithm_collection: A) -> Self {
        Self {
            game,
            grouping,
            algorithm_collection,
        }
    }
}

impl<
    M: ModelTypes<
            Distribution = SingleStateDistribution,
            Owners = TwoPlayer,
            Predecessors = VectorPredecessors,
        >,
    G: StateGroups,
    A: AlgorithmCollection,
> SimpleCooperativeGame for StateBasedResponsibilityGame<M, G, A>
{
    fn get_player_count(&self) -> usize {
        self.grouping.get_count()
    }

    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool {
        for i in 0..self.grouping.get_count() {
            if coalition.is_in_coalition(i) {
                for state in self.grouping.get_states(i) {
                    self.game.states[state].owner = TwoPlayer::PlayerOne;
                }
            }
        }

        let winning = self.algorithm_collection.compute_winning_player(&self.game);

        for i in 0..self.grouping.get_count() {
            if coalition.is_in_coalition(i) {
                for state in self.grouping.get_states(i) {
                    self.game.states[state].owner = TwoPlayer::PlayerTwo;
                }
            }
        }

        winning == TwoPlayer::PlayerOne
    }
}
