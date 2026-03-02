use crate::mdp::mecs;
use crate::mdp::sccs::{Scc, SccList, SccWithDependencies};
use probabilistic_models::{
    ActionCollection, ActionVector, AtomicPropositions, Distribution, DistributionVector,
    ProbabilisticModel, VectorPredecessors,
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

    let initial_eps = eps;

    loop {
        print!("eps={}:", eps);
        value_iteration_internal(&model, &mut data, eps, &sccs, &order[..]);

        let factor = 1.0 + 2.0 * initial_eps;
        for i in 0..model.states.len() {
            upper_bound[i] = match data[i].value {
                0.0 => 0.0,
                v => (v * factor).min(1.0),
            }
        }

        let is_upper_bound =
            verify_optimistic(&mut model, &mut eps, &mut data, &mut upper_bound, &sccs);

        match is_upper_bound {
            OptimisticValueIterationResult::UpperBoundVerified => {
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
            OptimisticValueIterationResult::UpperBoundRefuted { error } => {
                eps = error * 0.5;
            }
        }
    }

    println!(
        "Optimistic value iteration finished in {:?}: {}",
        start_time.elapsed(),
        data[0].value
    );
}

fn verify_optimistic<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
        >,
>(
    model: &mut ProbabilisticModel<M>,
    eps: &mut f64,
    mut data: &mut Vec<StateData>,
    upper_bound: &mut Vec<f64>,
    sccs: &SccList<SccWithDependencies>,
) -> OptimisticValueIterationResult {
    let verification_steps = (1.0 / *eps).max(1.0) as usize;
    println!("Verifying in {} steps", verification_steps);
    let mut error: f64 = 0.0;
    for _ in 0..verification_steps {
        //println!("  Step!");
        let mut all_up = true;
        let mut all_down = true;
        error = 0.0;
        for scc in &sccs.sccs {
            for &state_index in scc.get_members() {
                let mut new_lower_value = 0.0;
                let mut new_upper_value = 0.0;

                let state = &model.states[state_index];
                for action in state.actions.iter() {
                    let distribution = &action.successors;
                    let mut lower_value = 0.0;
                    let mut upper_value = 0.0;
                    for successor in distribution.iter() {
                        lower_value += successor.probability * data[successor.index].value;
                        upper_value += successor.probability * upper_bound[successor.index];
                    }
                    if lower_value >= new_lower_value {
                        new_lower_value = lower_value;
                    }
                    if upper_value >= new_upper_value {
                        new_upper_value = upper_value;
                    }
                }

                if new_lower_value > 0.0 {
                    error = error.max(new_lower_value - data[state_index].value);
                }
                data[state_index].value = new_lower_value;
                if new_upper_value < upper_bound[state_index] {
                    all_up = false;
                    upper_bound[state_index] = new_upper_value;
                } else if new_upper_value > upper_bound[state_index] {
                    all_down = false;
                }

                if new_upper_value < new_lower_value {
                    return OptimisticValueIterationResult::UpperBoundRefuted { error };
                }
            }
        }

        if all_down {
            return OptimisticValueIterationResult::UpperBoundVerified;
        } else if all_up {
            return OptimisticValueIterationResult::UpperBoundRefuted { error };
        }
    }
    OptimisticValueIterationResult::UpperBoundRefuted { error }
}

enum OptimisticValueIterationResult {
    UpperBoundVerified,
    UpperBoundRefuted { error: f64 },
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
            println!("\n\nState");
            let state = &model.states[state_index];

            let mut best_value = 0.0;

            for action in state.actions.iter() {
                print!("Action {}: ", action.action_name_index);
                let distribution = &action.successors;
                let mut value = 0.0;
                for successor in distribution.iter() {
                    print!(
                        "{} * {} ",
                        successor.probability, upper_bound[successor.index]
                    );
                    value += successor.probability * upper_bound[successor.index];
                }
                println!("= {}", value);
                if value >= best_value {
                    best_value = value;
                }
            }
            if best_value < upper_bound[state_index] {
                all_increasing = false;
            } else if best_value > upper_bound[state_index] {
                println!(
                    "Not all transitions are decreasing, {} > {}",
                    best_value, upper_bound[state_index]
                );
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
