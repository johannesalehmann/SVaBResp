use super::{Maximiser, Minimiser, StateData, ValueComparator, value_iteration_internal};
use crate::mecs;
use crate::sccs::{Scc, SccList, SccWithDependencies};
use probabilistic_models::{
    ActionCollection, ActionVector, AtomicPropositions, Distribution, DistributionVector,
    ProbabilisticModel, SinglePlayer, VectorPredecessors,
};

pub fn optimistic_value_iteration_maximise<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = SinglePlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    eps: f64,
) {
    optimistic_value_iteration(model, objective_ap_index, eps, Maximiser {})
}
pub fn optimistic_value_iteration_minimise<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = SinglePlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    eps: f64,
) {
    optimistic_value_iteration(model, objective_ap_index, eps, Minimiser {})
}

fn optimistic_value_iteration<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = SinglePlayer,
        >,
    C: ValueComparator<SinglePlayer>,
>(
    mut model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    mut eps: f64,
    comparator: C,
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
        crate::sccs::compute_sccs(&model, &crate::sccs::ExclusionList::new(&excluded[..]));
    let sccs = sccs.compute_dependencies(&model);
    let order = sccs.get_reverse_topological_order();

    let initial_eps = eps;

    loop {
        println!("eps={}:", eps);
        value_iteration_internal(&model, &mut data, eps, &sccs, &order[..], comparator);

        for i in 0..model.states.len() {
            upper_bound[i] = match data[i].value {
                0.0 => 0.0,
                v => (v + initial_eps).min(1.0),
            }
            // upper_bound[i] = match data[i].value {
            //     v => (v * (1.0 + initial_eps)).min(1.0),
            // }
        }

        let is_upper_bound = verify_optimistic(
            &mut model,
            eps,
            &mut data,
            &mut upper_bound,
            &sccs,
            comparator,
        );

        match is_upper_bound {
            OptimisticValueIterationResult::UpperBoundVerified => {
                println!("Upper bound candidate verified!");
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
            Owners = SinglePlayer,
        >,
    C: ValueComparator<SinglePlayer>,
>(
    model: &mut ProbabilisticModel<M>,
    eps: f64,
    data: &mut Vec<StateData>,
    upper_bound: &mut Vec<f64>,
    sccs: &SccList<SccWithDependencies>,
    value_comparator: C,
) -> OptimisticValueIterationResult {
    let start_time = std::time::Instant::now();
    let verification_steps = (1.0 / eps).max(1.0) as usize;
    let mut error: f64 = 0.0;
    for i in 0..verification_steps {
        let mut all_up = true;
        let mut all_down = true;
        error = 0.0;
        for scc in &sccs.sccs {
            for &state_index in scc.get_members() {
                let state = &model.states[state_index];
                let owner = &state.owner;

                let mut new_lower_value = value_comparator.initial_value(owner);
                let mut new_upper_value = value_comparator.initial_value(owner);

                for action in state.actions.iter() {
                    let distribution = &action.successors;
                    let mut lower_value = 0.0;
                    let mut upper_value = 0.0;
                    for successor in distribution.iter() {
                        lower_value += successor.probability * data[successor.index].value;
                        upper_value += successor.probability * upper_bound[successor.index];
                    }
                    if value_comparator.is_better(owner, new_lower_value, lower_value) {
                        new_lower_value = lower_value;
                    }
                    if value_comparator.is_better(owner, new_upper_value, upper_value) {
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
                    println!("Crossed in {} steps and {:?}", i, start_time.elapsed());
                    return OptimisticValueIterationResult::UpperBoundRefuted { error };
                }
            }
        }

        if all_down {
            println!("Verified in {} steps and {:?}", i, start_time.elapsed());
            return OptimisticValueIterationResult::UpperBoundVerified;
        } else if all_up {
            println!("Refuted in {} steps and {:?}", i, start_time.elapsed());
            return OptimisticValueIterationResult::UpperBoundRefuted { error };
        }
    }
    OptimisticValueIterationResult::UpperBoundRefuted { error }
}

enum OptimisticValueIterationResult {
    UpperBoundVerified,
    UpperBoundRefuted { error: f64 },
}

pub fn value_iteration_maximise<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = SinglePlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    eps: f64,
) {
    value_iteration(model, objective_ap_index, eps, Maximiser {})
}
pub fn value_iteration_minimise<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = SinglePlayer,
        >,
>(
    model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    eps: f64,
) {
    value_iteration(model, objective_ap_index, eps, Minimiser {})
}
fn value_iteration<
    M: probabilistic_models::ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = SinglePlayer,
        >,
    C: ValueComparator<SinglePlayer>,
>(
    mut model: ProbabilisticModel<M>,
    objective_ap_index: usize,
    eps: f64,
    value_comparator: C,
) {
    let start_time = std::time::Instant::now();

    let mecs = mecs::compute_mecs(&mut model);
    let winning_mecs = winning_mecs(&mecs, objective_ap_index, &model);
    mecs.collapse_mecs(&mut model);

    let mut data = vec![StateData::new(); model.states.len()];
    let excluded =
        handle_reachability_objective(&model, objective_ap_index, &mecs, &winning_mecs, &mut data);

    let sccs: SccList =
        crate::sccs::compute_sccs(&model, &crate::sccs::ExclusionList::new(&excluded[..]));
    let sccs = sccs.compute_dependencies(&model);
    let order = sccs.get_reverse_topological_order();

    value_iteration_internal(&model, &mut data, eps, &sccs, &order[..], value_comparator);

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
