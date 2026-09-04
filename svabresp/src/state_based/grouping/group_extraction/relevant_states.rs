use crate::state_based::game::{
    ApIdx, Buechi, Game, Objective, Reachability, Safety, StateIdx, WinningRegion, is_probabilistic,
};
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::typed_index_collections::To1;
use probabilistic_properties::Query;

pub struct RelevantStates {
    relevant_states: To1<StateIdx, bool>,
    dummy_states: Vec<StateIdx>,
}

impl RelevantStates {
    pub fn compute(game: &mut Game, property: &Query<i64, f64, ApIdx>) -> Self {
        if is_probabilistic(game) {
            let mut relevant_states = To1::with_capacity(game.states().len());
            let mut dummy_states = Vec::new();
            for state in game.states() {
                let relevant = game.choices_of_state(state).len() > 1;
                relevant_states.add_checked(state, relevant);
                if !relevant {
                    dummy_states.push(state);
                }
            }
            return Self {
                relevant_states,
                dummy_states,
            };
        }

        let (max_winning, min_winning) =
            if let Some(objective) = Reachability::try_from_query(property) {
                smallest_and_largest_winning_region(game, &objective)
            } else if let Some(objective) = Safety::try_from_query(property) {
                smallest_and_largest_winning_region(game, &objective)
            } else if let Some(objective) = Buechi::try_from_query(property) {
                smallest_and_largest_winning_region(game, &objective)
            } else {
                panic!("Unsupported property type")
            };

        let mut relevant_states = To1::with_capacity(game.states().len());
        let mut dummy_states = Vec::new();
        for state in game.states() {
            let relevant = game.choices_of_state(state).len() > 1
                && max_winning.contains(state)
                && !min_winning.contains(state);
            relevant_states.add_checked(state, relevant);
            if !relevant {
                dummy_states.push(state);
            }
        }

        Self {
            relevant_states,
            dummy_states,
        }
    }

    pub fn is_relevant(&self, state: StateIdx) -> bool {
        self.relevant_states[state]
    }

    pub fn into_dummy_states(self) -> Vec<StateIdx> {
        self.dummy_states
    }
}

fn smallest_and_largest_winning_region<O: Objective>(
    game: &Game,
    objective: &O,
) -> (WinningRegion, WinningRegion) {
    let mut context = objective.create_context(game);

    for state in game.states() {
        objective.set_owner(&mut context, state, TwoPlayer::Eve);
    }
    objective.reset(&mut context);
    let max_winning = objective.winning_region(game, &mut context);

    for state in game.states() {
        objective.set_owner(&mut context, state, TwoPlayer::Adam);
    }
    objective.reset(&mut context);
    let min_winning = objective.winning_region(game, &mut context);

    (max_winning, min_winning)
}
