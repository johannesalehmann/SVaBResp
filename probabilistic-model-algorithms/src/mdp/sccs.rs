use probabilistic_models::{ActionCollection, Distribution, ProbabilisticModel};

pub fn compute_sccs<M: probabilistic_models::ModelTypes>(
    model: &ProbabilisticModel<M>,
    exclude_states: &[usize],
) -> Sccs {
    let mut visited = vec![false; model.states.len()];
    let mut l = Vec::with_capacity(model.states.len());

    for &excluded in exclude_states {
        visited[excluded] = true;
    }

    for i in 0..model.states.len() {
        if !visited[i] {
            visit(model, &mut visited, &mut l, i);
        }
    }

    let mut reverse_edges = vec![Vec::new(); model.states.len()];
    for (i, state) in model.states.iter().enumerate() {
        for action in state.actions.iter() {
            for successor in action.successors.iter() {
                reverse_edges[successor.index].push(i);
            }
        }
    }

    for v in &mut visited {
        *v = false;
    }
    for &excluded in exclude_states {
        visited[excluded] = true;
    }

    let mut sccs = Vec::new();
    let mut state_to_scc = vec![0; model.states.len()];

    for &v in l.iter().rev() {
        if !visited[v] {
            let mut scc = Vec::new();
            visited[v] = true;
            visit_reversed(
                &mut visited,
                &reverse_edges,
                &mut scc,
                &mut state_to_scc,
                v,
                sccs.len(),
            );
            let scc = Scc {
                members: scc,
                depends_on: Vec::new(),
            };
            sccs.push(scc);
        }
    }

    for (scc_index, scc) in sccs.iter_mut().enumerate() {
        for &state in &scc.members {
            for action in model.states[state].actions.iter() {
                for successor in action.successors.iter() {
                    let successor_scc = state_to_scc[successor.index];
                    if successor_scc != scc_index && !scc.depends_on.contains(&successor_scc) {
                        scc.depends_on.push(successor_scc);
                    }
                }
            }
        }
    }
    Sccs { sccs }
}

fn visit<M: probabilistic_models::ModelTypes>(
    model: &ProbabilisticModel<M>,
    visited: &mut Vec<bool>,
    l: &mut Vec<usize>,
    i: usize,
) {
    let mut stack = Vec::new();
    stack.push((i, false));
    visited[i] = true;

    while let Some(top) = stack.pop() {
        match top {
            (i, false) => {
                stack.push((i, true));
                for action in model.states[i].actions.iter() {
                    for successor in action.successors.iter() {
                        if !visited[successor.index] {
                            visited[successor.index] = true;
                            stack.push((successor.index, false));
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

fn visit_reversed(
    visited: &mut Vec<bool>,
    reverse_edges: &Vec<Vec<usize>>,
    scc: &mut Vec<usize>,
    state_to_scc: &mut Vec<usize>,
    i: usize,
    scc_index: usize,
) {
    let mut stack = Vec::new();
    stack.push(i);
    while let Some(state) = stack.pop() {
        scc.push(state);
        state_to_scc[state] = scc_index;
        for &edge in &reverse_edges[state] {
            if !visited[edge] {
                visited[edge] = true;
                stack.push(edge);
            }
        }
    }
}

pub struct Sccs {
    pub sccs: Vec<Scc>,
}

impl Sccs {
    pub fn get_reverse_topological_order(&self) -> Vec<usize> {
        let mut visited = vec![false; self.sccs.len()];
        let mut order = Vec::new();

        for scc in 0..self.sccs.len() {
            if !visited[scc] {
                visited[scc] = true;
                self.dfs(scc, &mut visited, &mut order);
            }
        }

        order
    }

    fn dfs(&self, index: usize, visited: &mut Vec<bool>, order: &mut Vec<usize>) {
        for &scc in &self.sccs[index].depends_on {
            if !visited[scc] {
                visited[scc] = true;
                self.dfs(scc, visited, order);
            }
        }
        order.push(index);
    }
}

pub struct Scc {
    pub members: Vec<usize>,
    pub depends_on: Vec<usize>,
}
