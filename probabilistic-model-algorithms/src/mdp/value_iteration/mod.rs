use crate::mdp::mecs;
use crate::mdp::sccs::{Scc, SccList};
use probabilistic_models::{
    ActionCollection, ActionVector, AtomicPropositions, Distribution, DistributionVector,
    Predecessor, ProbabilisticModel, VectorPredecessors,
};

pub fn optimistic_value_iteration<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
        >,
>(
    mut model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    mut eps: f64,
) {
    let start_time = std::time::Instant::now();

    let mecs = mecs::compute_mecs(&mut model);
    let winning_mecs = winning_mecs(&mecs, objective_ap_index, &model);
    mecs.collapse_mecs(&mut model);

    let mut data = vec![StateData::new(); model.states.len()];
    let mut upper_bound = vec![0.0; model.states.len()];
    let excluded =
        handle_reachability_objective(&model, objective_ap_index, &mecs, &winning_mecs, &mut data);

    let sccs: SccList =
        crate::mdp::sccs::compute_sccs(&model, &super::sccs::ExclusionList::new(&excluded[..]));
    let sccs = sccs.compute_dependencies(&model);
    let order = sccs.get_reverse_topological_order();

    loop {
        print!("eps={}:", eps);
        value_iteration_internal(&model, &mut data, eps, &sccs, &order[..]);

        let factor = 1.0 + 2.0 * eps;
        for i in 0..model.states.len() {
            upper_bound[i] = match data[i].value {
                0.0 => 0.0,
                v => (v * factor).min(1.0),
            }
        }
        let is_upper_bound = is_upper_bound(&model, &upper_bound[..], &sccs);
        match is_upper_bound {
            BoundCheckResult::UpperBound => {
                println!(" Upper bound candidate verified!");
                for i in 0..model.states.len() {
                    if i == 0 {
                        println!(
                            "Lower bound: {}, upper bound: {}",
                            data[i].value, upper_bound[i]
                        );
                    }
                    data[i].value = 0.5 * (data[i].value + upper_bound[i]);
                }
                break;
            }
            BoundCheckResult::LowerBound => {
                println!(" Upper bound candidate is lower bound!");
                for i in 0..model.states.len() {
                    data[i].value = upper_bound[i];
                }
            }
            BoundCheckResult::Neither => {
                println!(" Could not verify upper bound!");
                eps = eps * 0.5;
            }
        }
    }

    println!(
        "Optimistic value iteration finished in {:?}: {}",
        start_time.elapsed(),
        data[0].value
    );
}

pub fn value_iteration<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
        >,
>(
    mut model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    eps: f64,
) {
    let start_time = std::time::Instant::now();

    let mecs = mecs::compute_mecs(&mut model);
    let winning_mecs = winning_mecs(&mecs, objective_ap_index, &model);
    mecs.collapse_mecs(&mut model);

    let mut data = vec![StateData::new(); model.states.len()];
    let excluded =
        handle_reachability_objective(&model, objective_ap_index, &mecs, &winning_mecs, &mut data);

    let sccs: SccList =
        crate::mdp::sccs::compute_sccs(&model, &super::sccs::ExclusionList::new(&excluded[..]));
    let sccs = sccs.compute_dependencies(&model);
    let order = sccs.get_reverse_topological_order();

    value_iteration_internal(&model, &mut data, eps, &sccs, &order[..]);

    println!(
        "Value iteration finished in {:?}: {}",
        start_time.elapsed(),
        data[0].value
    );
}

fn handle_reachability_objective<M: probabilistic_models::ModelTypes>(
    model: &ProbabilisticModel<M>,
    objective_ap_index: usize,
    mecs: &mecs::Mecs,
    winning_mecs: &[usize],
    data: &mut Vec<StateData>,
) -> Vec<usize> {
    let mut excluded = Vec::new();
    for (i, state) in model.states.iter().enumerate() {
        if state.atomic_propositions.get_value(objective_ap_index) {
            data[i].value = 1.0;
            excluded.push(i);
        }
    }

    for &winning_mec in winning_mecs {
        let state_index = mecs.identified_mec_state_index(winning_mec);
        if data[state_index].value != 1.0 {
            data[state_index].value = 1.0;
            excluded.push(state_index);
        }
    }
    excluded
}

fn winning_mecs<M: probabilistic_models::ModelTypes>(
    mecs: &mecs::Mecs,
    objective_ap_index: usize,
    model: &ProbabilisticModel<M>,
) -> Vec<usize> {
    let mut is_accepting = vec![false; mecs.len()];

    for (state_index, state) in model.states.iter().enumerate() {
        if let Some(mec_index) = mecs.mec_of_state(state_index) {
            if state.atomic_propositions.get_value(objective_ap_index) {
                is_accepting[mec_index] = true;
            }
        }
    }

    is_accepting
        .into_iter()
        .enumerate()
        .filter(|(_, acc)| *acc)
        .map(|(i, _)| i)
        .collect()
}

fn value_iteration_internal<M: probabilistic_models::ModelTypes, SCC: Scc>(
    model: &ProbabilisticModel<M>,
    data: &mut Vec<StateData>,
    eps: f64,
    sccs: &crate::mdp::sccs::SccList<SCC>,
    scc_order: &[usize],
) {
    for &scc in scc_order {
        loop {
            let mut largest_change: f64 = 0.0;
            for &state_index in sccs.sccs[scc].get_members() {
                let state = &model.states[state_index];

                let mut best_value = 0.0;
                let mut best_action = 0;

                for (action_index, action) in state.actions.iter().enumerate() {
                    let distribution = &action.successors;
                    let mut value = 0.0;
                    for successor in distribution.iter() {
                        value += successor.probability * data[successor.index].value;
                    }
                    if value >= best_value {
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

fn is_upper_bound<M: probabilistic_models::ModelTypes, S: Scc>(
    model: &ProbabilisticModel<M>,
    upper_bound: &[f64],
    sccs: &crate::mdp::sccs::SccList<S>,
) -> BoundCheckResult {
    let mut all_decreasing = true;
    let mut all_increasing = true;
    for scc in &sccs.sccs {
        for &state_index in scc.get_members() {
            let state = &model.states[state_index];

            let mut best_value = 0.0;

            for action in state.actions.iter() {
                let distribution = &action.successors;
                let mut value = 0.0;
                for successor in distribution.iter() {
                    value += successor.probability * upper_bound[successor.index];
                }
                if value >= best_value {
                    best_value = value;
                }
            }
            if best_value < upper_bound[state_index] {
                all_increasing = false;
            } else if best_value > upper_bound[state_index] {
                all_decreasing = false;
            }
        }
    }
    match (all_decreasing, all_increasing) {
        (true, true) => BoundCheckResult::UpperBound, // Only happens when the model is empty or when the bound is exactly the true value for all states
        (true, false) => BoundCheckResult::UpperBound,
        (false, true) => BoundCheckResult::LowerBound,
        (false, false) => BoundCheckResult::Neither,
    }
}

enum BoundCheckResult {
    UpperBound,
    LowerBound,
    Neither,
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
