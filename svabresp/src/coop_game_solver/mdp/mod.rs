use crate::grouping::{Grouping, StateIdx};
use prism_model_builder::To1;
use probabilistic_model_algorithms::sccs::{ExcludeStatesAndChoices, Scc, Sccs};
use probabilistic_models::base_model::{Mdp, TwoPlayer, TwoPlayerStochasticGame};
use probabilistic_models::traits::{BuildChoiceBranchGraph, ReadStateSpace};
use probabilistic_models::typed_index_collections::{Csr, index};
use probabilistic_models::{BranchIndex, ChoiceIndex, DefaultIx, Index, Ix, StateIndex, Sub};

pub type SubIx = Sub<DefaultIx>;

index!(ExternalTransitionIndex<I: Ix> => I::Branch);
index!(SubGameIndex<I: Ix> => I::State);
index!(GroupIndex<I: Ix> => I::Meta);
index!(GroupEntryIndex<I: Ix> => I::State);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SubGameState {
    sub_game: SubGameIndex<DefaultIx>,
    local_state: StateIndex<SubIx>,
}

struct Groups {
    group_to_entries: Csr<GroupIndex<DefaultIx>, GroupEntryIndex<DefaultIx>>,
    /// Sorted by sub game within each group.
    entry_states: To1<GroupEntryIndex<DefaultIx>, SubGameState>,
}

impl Groups {
    fn build(grouping: &Grouping, global_to_local: &GlobalToLocal) -> Self {
        let mut group_to_entries = Csr::with_capacity(grouping.groups.len());
        let mut entry_states = To1::new();
        for group in &grouping.groups {
            let group_start = entry_states.len();
            for &model_state in &group.states {
                if let Some(state) = global_to_local.global_to_sub_game_state(model_state) {
                    entry_states.add(state);
                }
            }
            entry_states.entries_mut()[group_start..].sort_unstable();
            group_to_entries.add_entry_unchecked(GroupEntryIndex::from_raw(entry_states.len()));
        }
        Self {
            group_to_entries,
            entry_states,
        }
    }

    /// For each group, the sub games that contain one of its states or exit into such a sub game.
    /// `successors` must only point to earlier sub games.
    fn compute_group_dirty_flags(
        &self,
        successors: &To1<SubGameIndex<DefaultIx>, Vec<SubGameIndex<DefaultIx>>>,
    ) -> To1<GroupIndex<DefaultIx>, To1<SubGameIndex<DefaultIx>, bool>> {
        let group_count = self.group_to_entries.keys().len();
        let dirty_per_group = (0..group_count).map(|group| {
            let mut dirty = To1::with_entries(vec![false; successors.len()]);
            for entry in self
                .group_to_entries
                .index(GroupIndex::from_raw(group as u32))
            {
                dirty[self.entry_states[entry].sub_game] = true;
            }
            for sub_game in (0..successors.len()).map(SubGameIndex::from_raw) {
                if successors[sub_game]
                    .iter()
                    .any(|&successor| dirty[successor])
                {
                    dirty[sub_game] = true;
                }
            }
            dirty
        });
        To1::with_entries(dirty_per_group.collect::<Vec<_>>())
    }
}

pub struct SubGameSolver {
    sub_games: To1<SubGameIndex<DefaultIx>, SubGame>,
    value_count: usize,
    initial_value_index: usize,
    groups: Groups,
    group_dirty_flags: To1<GroupIndex<DefaultIx>, To1<SubGameIndex<DefaultIx>, bool>>,
}

impl SubGameSolver {
    pub fn build(
        mdp: Mdp<DefaultIx>,
        initial: StateIdx,
        goal: &To1<StateIdx, bool>,
        grouping: &Grouping,
    ) -> Self {
        assert!(
            grouping.groups.len() <= u64::BITS as usize,
            "at most {} groups are supported",
            u64::BITS
        );
        let exclusion_criterion = Self::goal_exclusion_criterion(&mdp, goal);
        let sccs = Sccs::compute_tarjan(&mdp, &exclusion_criterion);

        let state_count = mdp.states().len();
        let mut global_to_local = GlobalToLocal {
            sub_game: To1::with_entries(vec![None; state_count]),
            local: To1::with_entries(vec![StateIndex::<SubIx>::from_raw(0); state_count]),
            value_vector_index: To1::with_entries(vec![0; state_count]),
        };
        let always_helping = To1::with_true_values(&grouping.always_helping, state_count);
        let mut sub_games: To1<_, SubGame> = To1::new();
        let mut successors_per_sub_game = To1::new();
        let mut next_value_index = 1;
        for scc in sccs.reverse_topological_ordering() {
            if scc.as_singleton().is_some_and(|state| goal[state]) {
                continue;
            }
            let sub_game_index = SubGameIndex::from_raw(sub_games.len());
            global_to_local.add_scc(scc, sub_game_index, next_value_index);
            let (sub_game, successors) =
                SubGame::build(&mdp, scc, &global_to_local, &always_helping);
            sub_games.add(sub_game);
            successors_per_sub_game.add(successors);
            next_value_index += scc.size();
        }

        let groups = Groups::build(grouping, &global_to_local);
        Self {
            sub_games,
            value_count: next_value_index,
            initial_value_index: global_to_local.global_to_value_vector_index(initial),
            group_dirty_flags: groups.compute_group_dirty_flags(&successors_per_sub_game),
            groups,
        }
    }

    fn goal_exclusion_criterion(
        mdp: &Mdp,
        goal: &To1<StateIdx, bool>,
    ) -> ExcludeStatesAndChoices<DefaultIx> {
        let state_count = mdp.states().len();
        let mut goal_choices = To1::with_entries(vec![false; mdp.choices().len()]);
        for goal_state in mdp.states().into_iter().filter(|&state| goal[state]) {
            for choice in mdp.choices_of_state(goal_state) {
                goal_choices[choice] = true;
            }
        }
        let exclusion_criterion =
            ExcludeStatesAndChoices::new(To1::with_entries(vec![false; state_count]), goal_choices);
        exclusion_criterion
    }
}

/// The mutable state of solving the sub games of a [`SubGameSolver`] for one coalition at a time.
#[derive(Clone)]
pub struct SolveWorker<'a> {
    solver: &'a SubGameSolver,
    values: Vec<f64>,
    owners: To1<SubGameIndex<DefaultIx>, To1<StateIndex<SubIx>, TwoPlayer>>,
    exit_values: To1<ChoiceIndex<SubIx>, f64>,
    values_before_adding_group: ValuesBeforeAddingGroup,
}

impl<'a> SolveWorker<'a> {
    pub fn new(solver: &'a SubGameSolver) -> Self {
        let mut values = vec![0.0; solver.value_count];
        values[0] = 1.0;
        Self {
            solver,
            values,
            owners: To1::with_entries(
                (solver.sub_games.iter())
                    .map(|sub_game| sub_game.game.players.state_to_owner.clone())
                    .collect::<Vec<_>>(),
            ),
            exit_values: To1::new(),
            values_before_adding_group: ValuesBeforeAddingGroup::default(),
        }
    }

    /// Solves the game for the empty coalition and returns the value of the initial state. Must be
    /// called before any group is added.
    pub fn solve_empty_coalition(&mut self, eps: f64) -> f64 {
        let all_sub_games = To1::with_entries(vec![true; self.solver.sub_games.len()]);
        self.solve_sub_games(&all_sub_games, eps)
    }

    /// Adds `group` to the coalition, solves it and returns the value of the initial state.
    pub fn add_group_and_solve(&mut self, group: usize, eps: f64) -> f64 {
        self.values_before_adding_group.save(group, &self.values);
        self.set_group_owner(group, TwoPlayer::Eve);
        let solver = self.solver;
        let dirty = &solver.group_dirty_flags[GroupIndex::from_raw(group as u32)];
        self.solve_sub_games(dirty, eps)
    }

    /// Undoes [`Self::add_group_and_solve`] without solving. Every group added after `group` must
    /// already be removed.
    pub fn remove_group(&mut self, group: usize) {
        self.set_group_owner(group, TwoPlayer::Adam);
        self.values
            .copy_from_slice(self.values_before_adding_group.get(group));
    }

    fn set_group_owner(&mut self, group: usize, owner: TwoPlayer) {
        let groups = &self.solver.groups;
        for entry in groups
            .group_to_entries
            .index(GroupIndex::from_raw(group as u32))
        {
            let state = groups.entry_states[entry];
            self.owners[state.sub_game][state.local_state] = owner;
        }
    }

    fn solve_sub_games(&mut self, dirty: &To1<SubGameIndex<DefaultIx>, bool>, eps: f64) -> f64 {
        let sub_games = self.solver.sub_games.entries();
        let mut first_value_index = 1;
        for ((sub_game, owners), &dirty) in
            (sub_games.iter().zip(self.owners.entries())).zip(dirty.entries())
        {
            let state_count = sub_game.game.states().len();
            if dirty {
                let (exit_values, own_values) = self.values.split_at_mut(first_value_index);
                sub_game.rebuild_exits(exit_values, &mut self.exit_values);
                sub_game.solve(
                    owners,
                    &self.exit_values,
                    &mut own_values[..state_count],
                    eps,
                );
            }
            first_value_index += state_count;
        }
        self.values[self.solver.initial_value_index]
    }
}

/// The value vector of a [`SolveWorker`] right before each group was added to its coalition.
#[derive(Clone, Default)]
struct ValuesBeforeAddingGroup {
    values_per_group: Vec<Vec<f64>>,
}

impl ValuesBeforeAddingGroup {
    fn save(&mut self, group: usize, values: &[f64]) {
        if self.values_per_group.len() <= group {
            self.values_per_group.resize(group + 1, Vec::new());
        }
        let saved_values = &mut self.values_per_group[group];
        saved_values.clear();
        saved_values.extend_from_slice(values);
    }

    fn get(&self, group: usize) -> &[f64] {
        &self.values_per_group[group]
    }
}

// Goal states have value vector index 0
struct GlobalToLocal {
    sub_game: To1<StateIdx, Option<SubGameIndex<DefaultIx>>>,
    local: To1<StateIdx, StateIndex<SubIx>>,
    value_vector_index: To1<StateIdx, usize>,
}

impl GlobalToLocal {
    fn add_scc(
        &mut self,
        scc: Scc<'_, DefaultIx>,
        sub_game: SubGameIndex<DefaultIx>,
        first_value_vector_index: usize,
    ) {
        for (local, model_state) in scc.states().enumerate() {
            self.sub_game[model_state] = Some(sub_game);
            self.local[model_state] = StateIndex::from_raw(local);
            self.value_vector_index[model_state] = first_value_vector_index + local;
        }
    }

    fn global_to_local(&self, state: StateIdx) -> StateIndex<SubIx> {
        self.local[state]
    }

    fn global_to_sub_game_state(&self, state: StateIdx) -> Option<SubGameState> {
        Some(SubGameState {
            sub_game: self.sub_game[state]?,
            local_state: self.local[state],
        })
    }

    fn global_to_value_vector_index(&self, state: StateIdx) -> usize {
        self.value_vector_index[state]
    }
}

struct ExternalTransition {
    target: usize,
    probability: f64,
}

pub struct SubGame {
    game: TwoPlayerStochasticGame<SubIx>,
    choice_to_external_transition: Csr<ChoiceIndex<SubIx>, ExternalTransitionIndex<SubIx>>,
    external_transitions: To1<ExternalTransitionIndex<SubIx>, ExternalTransition>,
}

impl SubGame {
    fn build(
        mdp: &Mdp<DefaultIx>,
        scc: Scc<'_, DefaultIx>,
        global_to_local: &GlobalToLocal,
        always_helping: &To1<StateIdx, bool>,
    ) -> (Self, Vec<SubGameIndex<DefaultIx>>) {
        let mut successors = Vec::new();
        let mut game = TwoPlayerStochasticGame::<SubIx>::default();
        let mut choice_to_external_transition = Csr::new();
        let mut external_transitions: To1<ExternalTransitionIndex<SubIx>, _> = To1::new();
        for model_state in scc.states() {
            let owner = if always_helping[model_state] {
                TwoPlayer::Eve
            } else {
                TwoPlayer::Adam
            };
            game.add_state(owner);
            for model_choice in mdp.choices_of_state(model_state) {
                game.add_choice();
                for branch in mdp.branches_of_choice(model_choice) {
                    let destination = mdp.branch_destination(branch);
                    let probability = mdp.branch_probability(branch);
                    if scc.contains(destination) {
                        game.add_branch(probability, global_to_local.global_to_local(destination));
                    } else {
                        successors.extend(global_to_local.sub_game[destination]);
                        external_transitions.add(ExternalTransition {
                            target: global_to_local.global_to_value_vector_index(destination),
                            probability,
                        });
                    }
                }
                choice_to_external_transition.add_entry_unchecked(
                    ExternalTransitionIndex::from_raw(external_transitions.len()),
                );
            }
        }
        successors.sort_unstable();
        successors.dedup();
        let sub_game = Self {
            game,
            choice_to_external_transition,
            external_transitions,
        };
        (sub_game, successors)
    }

    fn rebuild_exits(&self, values: &[f64], exit_values: &mut To1<ChoiceIndex<SubIx>, f64>) {
        exit_values.clear();
        for choice in self.game.choices() {
            let exit_value = (self.choice_to_external_transition.index(choice).into_iter())
                .map(|transition| {
                    let transition = &self.external_transitions[transition];
                    transition.probability * values[transition.target]
                })
                .sum();
            exit_values.add(exit_value);
        }
    }

    /// Solves the game using Gauss-Seidel value iteration. Eve maximises, Adam minimises.
    ///
    /// `values` must contain a lower bound when `solve()` is called, as the values are not reset.
    fn solve(
        &self,
        owners: &To1<StateIndex<SubIx>, TwoPlayer>,
        exit_values: &To1<ChoiceIndex<SubIx>, f64>,
        values: &mut [f64],
        eps: f64,
    ) {
        let graph = &self.game.graph;
        let choice_ends = graph.state_to_choice.entries_raw();
        let owners = owners.entries();
        debug_assert_eq!(values.len(), choice_ends.len());
        debug_assert_eq!(values.len(), owners.len());

        let probabilities = &graph.branch_probabilities;
        let destinations = &graph.branch_destinations;
        loop {
            let mut converged = true;
            let mut choice = ChoiceIndex::<SubIx>::from_raw(0);
            let mut branch = BranchIndex::<SubIx>::from_raw(0);
            for (state, (&choice_end, &owner)) in choice_ends.iter().zip(owners).enumerate() {
                let maximise = owner == TwoPlayer::Eve;
                let has_choices = choice < choice_end;
                let mut best_value = if maximise { 0.0 } else { f64::INFINITY };
                while choice < choice_end {
                    let mut choice_value = exit_values[choice];
                    let branch_end = graph.choice_to_branch.index_raw(choice);
                    while branch < branch_end {
                        choice_value += probabilities[branch] * values[destinations[branch].raw()];
                        branch += 1;
                    }
                    best_value = if maximise {
                        best_value.max(choice_value)
                    } else {
                        best_value.min(choice_value)
                    };
                    choice += 1;
                }
                if !has_choices {
                    best_value = 0.0;
                }

                if (best_value - values[state]).abs() > eps * best_value {
                    converged = false;
                }
                values[state] = best_value;
            }
            if converged {
                break;
            }
        }
    }
}
