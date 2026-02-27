mod exclusion_criterion;
pub use exclusion_criterion::*;

mod sccs_datastructure;
pub use sccs_datastructure::*;

use probabilistic_models::{ActionCollection, Distribution, ProbabilisticModel};

pub fn compute_sccs<
    M: probabilistic_models::ModelTypes,
    Ex: ExclusionCriterion,
    BS: BuildableScc,
>(
    model: &ProbabilisticModel<M>,
    excluded: &Ex,
) -> BS {
    let mut visited = vec![false; model.states.len()];
    let mut l = Vec::with_capacity(model.states.len());

    for excluded in excluded.iter_states() {
        visited[excluded] = true;
    }

    for i in 0..model.states.len() {
        if !visited[i] {
            visit(model, &mut visited, &mut l, i, excluded);
        }
    }

    let mut reverse_edges = vec![Vec::new(); model.states.len()];
    for (i, state) in model.states.iter().enumerate() {
        for (j, action) in state.actions.iter().enumerate() {
            if !excluded.is_action_excluded(i, j) {
                for successor in action.successors.iter() {
                    reverse_edges[successor.index].push(i);
                }
            }
        }
    }

    for v in &mut visited {
        *v = false;
    }
    for excluded in excluded.iter_states() {
        visited[excluded] = true;
    }

    let mut scc_builder = BS::builder(model);

    for &v in l.iter().rev() {
        if !visited[v] {
            let scc_index = scc_builder.add_scc();
            visited[v] = true;
            visit_reversed(&mut visited, &reverse_edges, v, &mut scc_builder, scc_index);
        }
    }

    scc_builder.finish()
}

fn visit<M: probabilistic_models::ModelTypes, Ex: ExclusionCriterion>(
    model: &ProbabilisticModel<M>,
    visited: &mut Vec<bool>,
    l: &mut Vec<usize>,
    i: usize,
    excluded: &Ex,
) {
    let mut stack = Vec::new();
    stack.push((i, false));
    visited[i] = true;

    while let Some(top) = stack.pop() {
        match top {
            (i, false) => {
                stack.push((i, true));
                for (j, action) in model.states[i].actions.iter().enumerate() {
                    if !excluded.is_action_excluded(i, j) {
                        for successor in action.successors.iter() {
                            if !visited[successor.index] {
                                visited[successor.index] = true;
                                stack.push((successor.index, false));
                            }
                        }
                    }
                }
            }
            (i, true) => {
                l.push(i);
            }
        }
    }
}

fn visit_reversed<BS, B: SccBuilder<BS>>(
    visited: &mut Vec<bool>,
    reverse_edges: &Vec<Vec<usize>>,
    i: usize,
    scc_builder: &mut B,
    scc_index: usize,
) {
    let mut stack = Vec::new();
    stack.push(i);
    while let Some(state) = stack.pop() {
        scc_builder.add_to_scc(state, scc_index);
        for &edge in &reverse_edges[state] {
            if !visited[edge] {
                scc_builder.mark_non_trivial(scc_index);
                visited[edge] = true;
                stack.push(edge);
            } else {
                // If an SCC has a single state with the self loop, it is not trivial, but the
                // above check does not count this edge. Therefore, we handle this separately:
                if edge == i {
                    scc_builder.mark_non_trivial(scc_index);
                }
            }
        }
    }
}
