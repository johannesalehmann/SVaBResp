use probabilistic_models::{
    ActionCollection, AtomicPropositions, Distribution, ProbabilisticModel,
};
use std::collections::HashSet;

pub fn value_iteration<M: probabilistic_models::ModelTypes>(
    model: &ProbabilisticModel<M>,
    objective_ap_index: usize,
) {
    let n = model.states.len();
    let mut data = vec![StateData::new(); n];
    for (i, state) in model.states.iter().enumerate() {
        if state.atomic_propositions.get_value(objective_ap_index) {
            data[i].value = 1.0;
        }
    }

    for i in 0..30_000 {
        if i % 5000 == 0 {
            println!("Iteration {}", i);
        }

        let mut largest_change: f64 = 0.0;
        for state_index in 0..n {
            let state = &model.states[state_index];
            if state.atomic_propositions.get_value(objective_ap_index) {
                continue;
            }

            let mut best_value = 0.0;
            let mut best_action = 0;
            for action in 0..state.actions.get_number_of_actions() {
                let distribution = &state.actions.get_action(action).successors;
                let mut value = 0.0;
                for successor in 0..distribution.number_of_successors() {
                    let successor = distribution.get_successor(successor);
                    value += successor.probability * data[successor.index].value;
                }
                if value >= best_value {
                    best_value = value;
                    best_action = action;
                }
            }

            largest_change = largest_change.max(best_value - data[state_index].value);
            data[state_index].value = best_value;
            data[state_index].action = best_action;
        }
    }
    // println!("Result of value iteration:");
    // for state in 0..n {
    //     println!("  {}: {}", state, data[state].value);
    // }
    println!("Result of value iteration: {}", data[0].value);
}

#[derive(Copy, Clone)]
struct StateData {
    value: f64,
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
