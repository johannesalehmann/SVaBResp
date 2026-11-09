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
}

pub struct SubGameSolver {
    sub_games: To1<SubGameIndex<DefaultIx>, SubGame>,
    values: Vec<f64>,
    groups: Groups,
    /// Bit `i` is set iff group `i` is part of the coalition.
    coalition: u64,
}

impl SubGameSolver {
    pub fn build(mdp: Mdp<DefaultIx>, goal: &To1<StateIdx, bool>, grouping: &Grouping) -> Self {
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
        let mut next_value_index = 1;
        for scc in sccs.reverse_topological_ordering() {
            if scc.as_singleton().is_some_and(|state| goal[state]) {
                continue;
            }
            let sub_game_index = SubGameIndex::from_raw(sub_games.len());
            global_to_local.add_scc(scc, sub_game_index, next_value_index);
            let (sub_game, successors) =
                SubGame::build(&mdp, scc, &global_to_local, &always_helping);
            for successor in successors {
                sub_games[successor].dependents.push(sub_game_index);
            }
            sub_games.add(sub_game);
            next_value_index += scc.size();
        }

        let mut values = vec![0.0; next_value_index];
        values[0] = 1.0;
        Self {
            sub_games,
            values,
            groups: Groups::build(grouping, &global_to_local),
            coalition: 0,
        }
    }

    pub fn solve(&mut self, coalition: u64) {
        self.set_owners(coalition);
    }

    fn set_owners(&mut self, coalition: u64) {
        let mut changed_groups = self.coalition ^ coalition;
        let mut newly_dirty = Vec::new();
        while changed_groups != 0 {
            let group = changed_groups.trailing_zeros();
            changed_groups &= changed_groups - 1;
            let owner = if coalition & (1 << group) != 0 {
                TwoPlayer::Eve
            } else {
                TwoPlayer::Adam
            };
            let group = GroupIndex::from_raw(group);
            for entry in self.groups.group_to_entries.index(group) {
                let state = self.groups.entry_states[entry];
                let sub_game = &mut self.sub_games[state.sub_game];
                sub_game.game.players.state_to_owner[state.local_state] = owner;
                if !sub_game.dirty {
                    sub_game.dirty = true;
                    newly_dirty.push(state.sub_game);
                }
            }
        }
        self.coalition = coalition;
        self.mark_dependents_dirty(newly_dirty);
    }

    fn mark_dependents_dirty(&mut self, mut stack: Vec<SubGameIndex<DefaultIx>>) {
        while let Some(sub_game) = stack.pop() {
            for i in 0..self.sub_games[sub_game].dependents.len() {
                let dependent = self.sub_games[sub_game].dependents[i];
                if !self.sub_games[dependent].dirty {
                    self.sub_games[dependent].dirty = true;
                    stack.push(dependent);
                }
            }
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
    exit_values: To1<ChoiceIndex<SubIx>, f64>,
    choice_to_external_transition: Csr<ChoiceIndex<SubIx>, ExternalTransitionIndex<SubIx>>,
    external_transitions: To1<ExternalTransitionIndex<SubIx>, ExternalTransition>,
    dirty: bool,
    dependents: Vec<SubGameIndex<DefaultIx>>, // Subgames that depend on this subgame
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
        let exit_values = To1::with_entries(vec![0.0; game.choices().len()]);
        successors.sort_unstable();
        successors.dedup();
        let sub_game = Self {
            game,
            exit_values,
            choice_to_external_transition,
            external_transitions,
            dirty: true,
            dependents: Vec::new(),
        };
        (sub_game, successors)
    }

    pub fn rebuild_exits(&mut self, values: &[f64]) {
        self.exit_values.fill(0.0);
        for (choice, transition) in &self.choice_to_external_transition {
            let transition = &self.external_transitions[transition];
            self.exit_values[choice] += transition.probability * values[transition.target];
        }
    }

    /// Solves the game using Gauss-Seidel value iteration. Eve maximises, Adam minimises.
    ///
    /// `values` must contain a lower bound when `solve()` is called, as the values are not reset.
    pub fn solve(&self, values: &mut [f64], eps: f64) {
        let graph = &self.game.graph;
        let choice_ends = graph.state_to_choice.entries_raw();
        let owners = self.game.players.state_to_owner.entries();
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
                    let mut choice_value = self.exit_values[choice];
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
