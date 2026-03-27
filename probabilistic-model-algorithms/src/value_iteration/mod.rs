pub mod mdp;
pub mod stochastic_games;

use crate::sccs::{Scc, SccList};
use probabilistic_models::{
    ActionCollection, Distribution, Owners, ProbabilisticModel, SinglePlayer, TwoPlayer, Valuation,
};

trait ValueComparator<O: Owners>: Copy {
    fn initial_value(&self, state_owner: &O) -> f64;
    fn is_better(&self, state_owner: &O, before: f64, new: f64) -> bool;
}

#[derive(Copy, Clone)]
struct Maximiser {}

impl ValueComparator<SinglePlayer> for Maximiser {
    fn initial_value(&self, state_owner: &SinglePlayer) -> f64 {
        let _ = state_owner;
        0.0
    }

    fn is_better(&self, state_owner: &SinglePlayer, before: f64, new: f64) -> bool {
        let _ = state_owner;
        new >= before
    }
}

#[derive(Copy, Clone)]
struct Minimiser {}

impl ValueComparator<SinglePlayer> for Minimiser {
    fn initial_value(&self, state_owner: &SinglePlayer) -> f64 {
        let _ = state_owner;
        0.0
    }

    fn is_better(&self, state_owner: &SinglePlayer, before: f64, new: f64) -> bool {
        let _ = state_owner;
        new <= before
    }
}

#[derive(Copy, Clone)]
struct TwoPlayerMaxMin {}

impl ValueComparator<TwoPlayer> for TwoPlayerMaxMin {
    fn initial_value(&self, state_owner: &TwoPlayer) -> f64 {
        match state_owner {
            TwoPlayer::PlayerOne => 0.0,
            TwoPlayer::PlayerTwo => 1.0,
        }
    }

    fn is_better(&self, state_owner: &TwoPlayer, before: f64, new: f64) -> bool {
        match state_owner {
            TwoPlayer::PlayerOne => new >= before,
            TwoPlayer::PlayerTwo => new <= before,
        }
    }
}

fn value_iteration_internal<
    M: probabilistic_models::ModelTypes,
    SCC: Scc,
    C: ValueComparator<M::Owners>,
>(
    model: &ProbabilisticModel<M>,
    data: &mut Vec<StateData>,
    eps: f64,
    sccs: &SccList<SCC>,
    scc_order: &[usize],
    comparator: C,
) {
    let print_details = false;
    if print_details {
        println!("Value iteration");
    }
    for &scc in scc_order {
        if print_details {
            println!(
                "  Scc {} with states {:?}",
                scc,
                sccs.sccs[scc].get_members()
            );
        }
        loop {
            let mut largest_change: f64 = 0.0;
            for &state_index in sccs.sccs[scc].get_members() {
                if print_details {
                    println!(
                        "      State {}",
                        model.states[state_index]
                            .valuation
                            .displayable(&model.valuation_context),
                    );
                }
                let state = &model.states[state_index];
                let owner = &state.owner;

                let mut best_value = comparator.initial_value(owner);
                let mut best_action = 0;

                for (action_index, action) in state.actions.iter().enumerate() {
                    if print_details {
                        println!("        Action {}", action_index);
                    }
                    let distribution = &action.successors;
                    let mut value = 0.0;
                    for successor in distribution.iter() {
                        if print_details {
                            println!(
                                "          to {} with {}, has value {}",
                                model.states[successor.index]
                                    .valuation
                                    .displayable(&model.valuation_context),
                                successor.probability,
                                data[successor.index].value
                            );
                        }
                        value += successor.probability * data[successor.index].value;
                    }
                    if comparator.is_better(owner, best_value, value) {
                        best_value = value;
                        best_action = action_index;
                    }
                }

                let state_data = &mut data[state_index];
                let absolute_error = best_value - state_data.value;
                let relative_error = absolute_error / best_value;
                if relative_error > largest_change {
                    largest_change = relative_error;
                }
                if print_details {
                    println!("      {} -> {}", state_data.value, best_value);
                }
                *state_data = StateData {
                    value: best_value,
                    action: best_action,
                };
            }
            if largest_change < eps {
                break;
            }
        }
    }
}

#[derive(Copy, Clone)]
struct StateData {
    value: f64,
    #[allow(unused)]
    action: usize,
}

impl StateData {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            action: 0,
        }
    }
}
