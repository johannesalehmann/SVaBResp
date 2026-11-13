use crate::coop_game::CooperativeGame;
use crate::coop_game_solver::mdp::{SolveWorker, SubGameSolver};
use crate::grouping::{Grouping, StateIdx};
use crate::scheduler::PartialScheduler;
use prism_model_builder::To1;
use probabilistic_models::DefaultIx;
use probabilistic_models::base_model::Mdp;
use std::thread::Scope;

pub mod mdp;

/// Computes the value of every coalition of groups. The extensions of each coalition of the
/// highest `parallel_groups` groups are solved in their own thread. The result does not depend on
/// `parallel_groups`.
pub fn build_cooperative_game(
    mdp: Mdp<DefaultIx>,
    initial: StateIdx,
    goal: &To1<StateIdx, bool>,
    grouping: &Grouping,
    scheduler: Option<&PartialScheduler>,
    parallel_groups: usize,
    eps: f64,
) -> CooperativeGame {
    let group_count = grouping.groups.len();
    let solver = SubGameSolver::build(mdp, initial, goal, grouping, scheduler);
    let mut worker = SolveWorker::new(&solver);
    let mut coalition_values = vec![0.0; 1 << group_count];
    coalition_values[0] = worker.solve_empty_coalition(eps);
    std::thread::scope(|scope| {
        solve_extensions_in_parallel(
            scope,
            worker,
            group_count,
            parallel_groups,
            &mut coalition_values,
            eps,
        );
    });
    CooperativeGame::new(group_count, coalition_values)
}

/// Like [`solve_extensions`], but solves the extensions by each subset of the highest
/// `parallel_groups` remaining groups in their own thread.
fn solve_extensions_in_parallel<'scope, 'env>(
    scope: &'scope Scope<'scope, 'env>,
    mut worker: SolveWorker<'env>,
    remaining_groups: usize,
    parallel_groups: usize,
    coalition_values: &'scope mut [f64],
    eps: f64,
) {
    if parallel_groups == 0 || remaining_groups == 0 {
        solve_extensions(&mut worker, remaining_groups, coalition_values, eps);
        return;
    }
    let group = remaining_groups - 1;
    let (without_group, with_group) = coalition_values.split_at_mut(1 << group);
    let mut extended_worker = worker.clone();
    scope.spawn(move || {
        with_group[0] = extended_worker.add_group_and_solve(group, eps);
        solve_extensions_in_parallel(
            scope,
            extended_worker,
            group,
            parallel_groups - 1,
            with_group,
            eps,
        );
    });
    solve_extensions_in_parallel(
        scope,
        worker,
        group,
        parallel_groups - 1,
        without_group,
        eps,
    );
}

/// Writes the values of the worker's coalition extended by every subset of the lowest
/// `remaining_groups` groups, indexed by that subset. `coalition_values[0]`, the value of the
/// coalition itself, must already be set. The worker is back in its coalition on return.
fn solve_extensions(
    worker: &mut SolveWorker,
    remaining_groups: usize,
    coalition_values: &mut [f64],
    eps: f64,
) {
    let Some(group) = remaining_groups.checked_sub(1) else {
        return;
    };
    let (without_group, with_group) = coalition_values.split_at_mut(1 << group);
    solve_extensions(worker, group, without_group, eps);
    with_group[0] = worker.add_group_and_solve(group, eps);
    solve_extensions(worker, group, with_group, eps);
    worker.remove_group(group);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grouping::StateGroup;
    use probabilistic_models::{Index, mdp};

    #[test]
    fn build_cooperative_game() {
        // s0, s1 and s4 form groups 0, 1 and 2. s2 is the goal and s3 a deadlock.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 0.5: s2 & 0.5: s4,
            s1 -> 0.5: s2 & 0.5: s0,
            s1 -> 1.0: s3,
            s2 -> 1.0: s2,
            s3 -> deadlock,
            s4 -> 1.0: s2,
            s4 -> 1.0: s3
        });
        let goal = To1::with_entries(vec![false, false, true, false, false]);
        let group = |s: usize| StateGroup {
            name: format!("s{s}"),
            states: vec![StateIdx::from_raw(s)],
        };
        let grouping = Grouping::new(vec![group(0), group(1), group(4)]);
        let build = |parallel_groups| {
            super::build_cooperative_game(
                mdp.clone(),
                StateIdx::from_raw(0),
                &goal,
                &grouping,
                None,
                parallel_groups,
                1e-9,
            )
        };

        let expected = [0.0, 0.5, 0.5, 1.0, 0.0, 1.0, 1.0, 1.0];
        let sequential = build(0);
        for (value, expected) in sequential.values.iter().zip(expected) {
            assert!((value - expected).abs() < 1e-6, "{value} != {expected}");
        }
        for parallel_groups in 1..=4 {
            assert_eq!(build(parallel_groups).values, sequential.values);
        }
    }
}
