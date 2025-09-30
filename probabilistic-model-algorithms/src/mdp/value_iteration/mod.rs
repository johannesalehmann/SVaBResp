use probabilistic_models::{
    ActionCollection, AtomicPropositions, Distribution, ProbabilisticModel,
};
use std::collections::HashSet;

pub fn value_iteration<M: probabilistic_models::ModelTypes>(
    model: &ProbabilisticModel<M>,
    objective_ap_index: usize,
) {
    let start_time = std::time::Instant::now();

    let n = model.states.len();
    let mut data = vec![StateData::new(); n];
    for (i, state) in model.states.iter().enumerate() {
        if state.atomic_propositions.get_value(objective_ap_index) {
            data[i].value = 1.0;
        }
    }
    println!("Finding SCCs");
    let sccs = crate::mdp::sccs::compute_sccs(model);
    println!("SCCs: {}", sccs.sccs.len());
    let order = sccs.get_reverse_topological_order();

    for scc in order {
        for i in 0..200_000 {
            let mut largest_change: f64 = 0.0;
            for &state_index in &sccs.sccs[scc].members {
                let state = &model.states[state_index];
                if state.atomic_propositions.get_value(objective_ap_index) {
                    continue;
                }

                let mut best_value = 0.0; // data[state_index].value; // 0.0;
                let mut best_action = 0; //data[state_index].action; // 0;

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
            if largest_change < 0.000_000_000_1 {
                // if i > 1 {
                //     println!(
                //         "Aborting after {} iterations (size: {})",
                //         i + 1,
                //         sccs.sccs[scc].members.len()
                //     );
                // }
                break;
            }
        }
    }
    // println!("Result of value iteration:");
    // for state in 0..n {
    //     println!("  {}: {}", state, data[state].value);
    // }
    println!(
        "Value iteration finished in {:?}: {}",
        start_time.elapsed(),
        data[0].value
    );
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
