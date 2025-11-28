use probabilistic_models::{
    AtomicPropositions, Distribution, InitialStates, ProbabilisticModel, SingleStateDistribution,
};
use std::collections::{HashMap, HashSet};

pub fn is_reachable<M: probabilistic_models::ModelTypes<Distribution = SingleStateDistribution>>(
    model: &ProbabilisticModel<M>,
    objective_ap_index: usize,
) -> ReachableResult {
    let mut open_list = Vec::new();
    let mut predecessors = HashMap::new();
    for &initial in model.initial_states.iter() {
        if model.states[initial]
            .atomic_propositions
            .get_value(objective_ap_index)
        {
            return ReachableResult::Reachable(get_path(predecessors, initial));
        }

        open_list.push(initial);
        predecessors.insert(initial, Predecessor::None);
    }

    while let Some(state) = open_list.pop() {
        for successor in model.states.get(state).unwrap().get_all_successors() {
            let index = successor.target_index;
            if !predecessors.contains_key(&index) {
                if model.states[index]
                    .atomic_propositions
                    .get_value(objective_ap_index)
                {
                    return ReachableResult::Reachable(get_path(predecessors, index));
                }

                open_list.push(index);
                predecessors.insert(
                    index,
                    Predecessor::Some {
                        index: state,
                        action_index: successor.action_index,
                    },
                );
            }
        }
    }

    ReachableResult::Unreachable
}

fn get_path(predecessors: HashMap<usize, Predecessor>, target: usize) -> Path {
    let mut states = Vec::new();
    let mut actions = Vec::new();
    states.push(target);
    let mut current = target;
    while let Predecessor::Some {
        index,
        action_index,
    } = predecessors[&current]
    {
        actions.push(action_index);
        states.push(index);
        current = index;
    }

    states.reverse();
    actions.reverse();

    Path { states, actions }
}

pub enum ReachableResult {
    Reachable(Path),
    Unreachable,
}

pub struct Path {
    states: Vec<usize>,
    actions: Vec<usize>,
}

enum Predecessor {
    None,
    Some { index: usize, action_index: usize },
}
