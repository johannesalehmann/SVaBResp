use crate::shapley::{CoalitionSpecifier, MonotoneCooperativeGame, SimpleCooperativeGame};
use crate::state_based::grouping::StateGroups;
use probabilistic_model_algorithms::two_player_games::non_probabilistic::{
    AlgorithmCollection, ChangeableOwners, SolvableGame,
};
use probabilistic_models::{
    ModelTypes, ProbabilisticModel, SingleStateDistribution, TwoPlayer, VectorPredecessors,
};

pub struct StateBasedResponsibilityGame<G: StateGroups, A: SolvableGame> {
    solvable: A,
    grouping: G,
}

impl<'a, G: StateGroups, A: SolvableGame> StateBasedResponsibilityGame<G, A> {
    pub fn new(solvable: A, grouping: G) -> Self {
        Self { solvable, grouping }
    }
}

impl<G: StateGroups, A: SolvableGame> SimpleCooperativeGame for StateBasedResponsibilityGame<G, A> {
    fn get_player_count(&self) -> usize {
        self.grouping.get_count()
    }

    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool {
        for i in 0..self.grouping.get_count() {
            if coalition.is_in_coalition(i) {
                for state in self.grouping.get_states(i) {
                    self.solvable.set_owner(state, TwoPlayer::PlayerOne);
                }
            } else {
                for state in self.grouping.get_states(i) {
                    self.solvable.set_owner(state, TwoPlayer::PlayerTwo);
                }
            }
        }

        self.solvable.get_winner() == TwoPlayer::PlayerOne
    }
}

impl<G: StateGroups, A: SolvableGame> MonotoneCooperativeGame
    for StateBasedResponsibilityGame<G, A>
{
}
