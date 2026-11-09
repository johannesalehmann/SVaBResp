mod subgame;

use crate::coop_game::{Coalition, CooperativeGame};
use crate::grouping::{Grouping, StateIdx};
use probabilistic_model_algorithms::sccs::{ExcludeStatesAndChoices, Sccs};
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::typed_index_collections::{Csr, To1, index};
use probabilistic_models::{DefaultIx, Index, Ix};
use std::ops::Range;
use subgame::{SubChoiceIdx, SubStateIdx, Subgame, ValuePositionIdx};

index!(SubgameIndex<I: Ix> => I::State);
index!(GroupIndex<I: Ix> => I::Meta);
index!(GroupEntryIndex<I: Ix> => I::State);

pub type SubgameIdx = SubgameIndex<DefaultIx>;
pub type GroupIdx = GroupIndex<DefaultIx>;
pub type GroupEntryIdx = GroupEntryIndex<DefaultIx>;

type ValueVector = To1<ValuePositionIdx, f64>;

fn goal_position() -> ValuePositionIdx {
    ValuePositionIdx::from_raw(0)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SubgameState {
    subgame: SubgameIdx,
    local_state: SubStateIdx,
}

/// Computes the cooperative game of an MDP. States in the coalition are owned by Eve, who
/// maximises, all other states by Adam, who minimises.
pub struct MdpSolver {
    /// Every subgame only exits into earlier subgames.
    subgames: To1<SubgameIdx, Subgame>,
    value_count: usize,
    initial_position: ValuePositionIdx,
    always_helping: Vec<SubgameState>,
    groups: Csr<GroupIdx, GroupEntryIdx>,
    /// Sorted by subgame.
    group_states: To1<GroupEntryIdx, SubgameState>,
}

impl MdpSolver {
    pub fn new<M: ReadStateSpace<Ix = DefaultIx>>(
        model: &M,
        initial: StateIdx,
        goal: &To1<StateIdx, bool>,
        grouping: &Grouping,
    ) -> Self {
        let exclusion_criterion = ExcludeStatesAndChoices::new(
            goal.clone(),
            To1::with_entries(vec![false; model.choices().len()]),
        );
        let sccs = Sccs::compute_tarjan(model, &exclusion_criterion);

        let state_count = model.states().len();
        let mut value_positions = To1::with_entries(vec![goal_position(); state_count]);
        let mut subgame_state = To1::with_entries(vec![None; state_count]);
        let mut subgames = To1::new();
        let mut next_position = goal_position() + 1;
        for (subgame, scc) in sccs.reverse_topological_ordering().enumerate() {
            let subgame_positions = next_position..next_position + scc.size();
            for (local_state, model_state) in scc.states().enumerate() {
                value_positions[model_state] = next_position + local_state;
                subgame_state[model_state] = Some(SubgameState {
                    subgame: SubgameIdx::from_raw(subgame),
                    local_state: SubStateIdx::from_raw(local_state),
                });
            }
            next_position = subgame_positions.end;
            subgames.add(Subgame::new(
                model,
                scc.states(),
                subgame_positions,
                &value_positions,
            ));
        }

        let mut groups = Csr::with_capacity(grouping.groups.len());
        let mut group_states = To1::new();
        for group in &grouping.groups {
            let group_start = group_states.len();
            for &model_state in &group.states {
                if let Some(state) = subgame_state[model_state] {
                    group_states.add(state);
                }
            }
            group_states.entries_mut()[group_start..].sort_unstable();
            groups.add_entry_unchecked(GroupEntryIdx::from_raw(group_states.len()));
        }

        Self {
            subgames,
            value_count: next_position.raw(),
            initial_position: value_positions[initial],
            always_helping: (grouping.always_helping.iter())
                .filter_map(|&model_state| subgame_state[model_state])
                .collect(),
            groups,
            group_states,
        }
    }

    /// Computes the value of every coalition. The coalitions of the highest `split_groups` groups
    /// are solved sequentially, after which one thread per such coalition solves its extensions
    /// by the remaining groups. The result does not depend on `split_groups`.
    pub fn solve(&self, split_groups: usize, eps: f64) -> CooperativeGame {
        let group_count = self.groups.keys().len();
        let thread_groups = group_count - split_groups.min(group_count);

        let mut root_values = To1::with_entries(vec![0.0; self.value_count]);
        root_values[goal_position()] = 1.0;
        let mut worker = Worker::new(self, 0, eps);
        worker.solve_subgames_from(SubgameIdx::from_raw(0), &mut root_values);
        let mut split_values = vec![To1::new(); 1 << (group_count - thread_groups)];
        let split_groups = thread_groups..group_count;
        worker.walk(0, split_groups, root_values, |coalition, values| {
            split_values[(coalition >> thread_groups) as usize] = values.clone();
        });

        let mut coalition_values = vec![0.0; 1 << group_count];
        let extension_mask: Coalition = (1 << thread_groups) - 1;
        std::thread::scope(|scope| {
            let thread_chunks = coalition_values.chunks_mut(1 << thread_groups);
            for ((split_coalition, values), chunk) in
                (split_values.into_iter().enumerate()).zip(thread_chunks)
            {
                scope.spawn(move || {
                    let root = (split_coalition as Coalition) << thread_groups;
                    let mut worker = Worker::new(self, root, eps);
                    worker.walk(root, 0..thread_groups, values, |coalition, values| {
                        chunk[(coalition & extension_mask) as usize] =
                            values[self.initial_position];
                    });
                });
            }
        });
        CooperativeGame::new(group_count, coalition_values)
    }

    fn value_position(&self, state: SubgameState) -> ValuePositionIdx {
        self.subgames[state.subgame].value_position(state.local_state)
    }

    fn first_affected_subgame(&self, group: usize) -> SubgameIdx {
        let entries = self.groups.index(GroupIdx::from_raw(group as u32));
        match entries.len() {
            0 => SubgameIdx::from_raw(self.subgames.len()),
            _ => self.group_states[entries.start].subgame,
        }
    }
}

struct Worker<'a> {
    solver: &'a MdpSolver,
    eps: f64,
    owned_by_eve: To1<ValuePositionIdx, bool>,
    exit_value_buffer: To1<SubChoiceIdx, f64>,
}

impl<'a> Worker<'a> {
    fn new(solver: &'a MdpSolver, coalition: Coalition, eps: f64) -> Self {
        let mut worker = Self {
            solver,
            eps,
            owned_by_eve: To1::with_entries(vec![false; solver.value_count]),
            exit_value_buffer: To1::new(),
        };
        for &state in &solver.always_helping {
            worker.owned_by_eve[solver.value_position(state)] = true;
        }
        let group_count = solver.groups.keys().len();
        for group in (0..group_count).filter(|&group| coalition >> group & 1 == 1) {
            worker.set_group_owned_by_eve(group, true);
        }
        worker
    }

    fn set_group_owned_by_eve(&mut self, group: usize, owned_by_eve: bool) {
        for entry in self.solver.groups.index(GroupIdx::from_raw(group as u32)) {
            let position = self.solver.value_position(self.solver.group_states[entry]);
            self.owned_by_eve[position] = owned_by_eve;
        }
    }

    fn solve_subgames_from(&mut self, first_subgame: SubgameIdx, values: &mut ValueVector) {
        for subgame in &self.solver.subgames.entries()[first_subgame.raw()..] {
            subgame.solve(
                &self.owned_by_eve,
                &mut self.exit_value_buffer,
                values,
                self.eps,
            );
        }
    }

    /// Visits `root` and all coalitions that extend it by `groups`, in depth-first order. Groups
    /// are added from highest to lowest, i.e. the parent of a coalition lacks its lowest group.
    fn walk(
        &mut self,
        root: Coalition,
        groups: Range<usize>,
        root_values: ValueVector,
        mut visit: impl FnMut(Coalition, &ValueVector),
    ) {
        visit(root, &root_values);
        let mut values_per_depth = vec![root_values; groups.len() + 1];
        self.visit_extensions(root, groups, &mut values_per_depth, &mut visit);
    }

    /// `values_per_depth[0]` holds the values of `coalition`; the deeper entries are overwritten.
    fn visit_extensions(
        &mut self,
        coalition: Coalition,
        groups: Range<usize>,
        values_per_depth: &mut [ValueVector],
        visit: &mut impl FnMut(Coalition, &ValueVector),
    ) {
        let (parent_values, deeper_values) = values_per_depth.split_first_mut().unwrap();
        let lowest_group = groups.start;
        for group in groups.rev() {
            let extended = coalition | 1 << group;
            let values = &mut deeper_values[0];
            values.copy_from_other(parent_values);
            self.set_group_owned_by_eve(group, true);
            self.solve_subgames_from(self.solver.first_affected_subgame(group), values);
            visit(extended, values);
            self.visit_extensions(extended, lowest_group..group, deeper_values, visit);
            self.set_group_owned_by_eve(group, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grouping::StateGroup;
    use probabilistic_models::{Model, mdp};

    #[test]
    fn solve() {
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
        let model = Model::new(mdp);
        let goal = To1::with_entries(vec![false, false, true, false, false]);
        let group = |s: usize| StateGroup {
            name: format!("s{s}"),
            states: vec![StateIdx::from_raw(s)],
        };
        let grouping = Grouping::new(vec![group(0), group(1), group(4)]);
        let solver = MdpSolver::new(&model, StateIdx::from_raw(0), &goal, &grouping);

        let expected = [0.0, 0.5, 0.5, 1.0, 0.0, 1.0, 1.0, 1.0];
        let sequential = solver.solve(0, 1e-9);
        for (value, expected) in sequential.values.iter().zip(expected) {
            assert!((value - expected).abs() < 1e-6, "{value} != {expected}");
        }
        for split_groups in 1..=4 {
            assert_eq!(solver.solve(split_groups, 1e-9).values, sequential.values);
        }
    }
}
