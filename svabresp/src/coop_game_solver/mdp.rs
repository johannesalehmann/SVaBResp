use crate::grouping::{Grouping, StateIdx};
use probabilistic_model_algorithms::sccs::{ExcludeStatesAndChoices, Sccs};
use probabilistic_models::base_model::{BuildOwners, TwoPlayer, TwoPlayerStochasticGame};
use probabilistic_models::traits::{BuildChoiceBranchGraph, BuildStates, ReadStateSpace};
use probabilistic_models::typed_index_collections::{Csr, To1, index};
use probabilistic_models::{ChoiceIndex, DefaultIx, Index, Ix, StateIndex, Sub};

pub type SubIx = Sub<DefaultIx>;
pub type SubStateIdx = StateIndex<SubIx>;
pub type SubChoiceIdx = ChoiceIndex<SubIx>;

index!(SubgameIndex<I: Ix> => I::State);
index!(ExitIndex<I: Ix> => I::Branch);
index!(GroupIndex<I: Ix> => I::Meta);
index!(GroupEntryIndex<I: Ix> => I::State);

pub type SubgameIdx = SubgameIndex<DefaultIx>;
pub type ExitIdx = ExitIndex<SubIx>;
pub type GroupIdx = GroupIndex<DefaultIx>;
pub type GroupEntryIdx = GroupEntryIndex<DefaultIx>;

/// The location of a state of the original model within the subgames.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubgameState {
    pub subgame: SubgameIdx,
    pub state: SubStateIdx,
}

/// The game restricted to a single SCC. Eve is the helping player and maximises, Adam is the
/// adversarial player and minimises.
pub struct Subgame {
    pub game: TwoPlayerStochasticGame<SubIx>,
    /// For each choice, the value contributed by the branches leaving the subgame. Initially 0.
    pub exit_values: To1<SubChoiceIdx, f64>,
    /// For each choice, the branches leaving the subgame.
    pub exits: Csr<SubChoiceIdx, ExitIdx>,
    pub exit_branches: To1<ExitIdx, (StateIdx, f64)>,
    pub to_global: To1<SubStateIdx, StateIdx>,
}

/// Computes the cooperative game of an MDP by splitting it into one subgame per SCC.
pub struct MdpSolver {
    /// Subgames in reverse topological order, i.e. every subgame only exits into earlier ones.
    pub subgames: To1<SubgameIdx, Subgame>,
    pub values: To1<StateIdx, f64>,
    pub groups: Csr<GroupIdx, GroupEntryIdx>,
    /// The states of each group, sorted by subgame.
    pub group_states: To1<GroupEntryIdx, SubgameState>,
}

impl MdpSolver {
    /// Goal states are not part of any subgame and have value 1. All states start out owned by
    /// Adam, except the always helping ones, which are owned by Eve.
    pub fn new<M: ReadStateSpace<Ix = DefaultIx>>(
        model: &M,
        goal: &To1<StateIdx, bool>,
        grouping: &Grouping,
    ) -> Self {
        let exclusion = ExcludeStatesAndChoices::new(
            goal.clone(),
            To1::with_entries(vec![false; model.choices().len()]),
        );
        let sccs = Sccs::compute_tarjan(model, &exclusion);

        let mut location = To1::with_entries(vec![None; model.states().len()]);
        for (subgame, scc) in sccs.reverse_topological_ordering().enumerate() {
            for (state, global) in scc.states().enumerate() {
                location[global] = Some(SubgameState {
                    subgame: SubgameIdx::from_raw(subgame),
                    state: SubStateIdx::from_raw(state),
                });
            }
        }

        let mut subgames = To1::with_entries(
            sccs.reverse_topological_ordering()
                .enumerate()
                .map(|(subgame, scc)| {
                    Subgame::new(
                        model,
                        scc.states(),
                        SubgameIdx::from_raw(subgame),
                        &location,
                    )
                })
                .collect::<Vec<_>>(),
        );
        for &state in &grouping.always_helping {
            if let Some(loc) = location[state] {
                subgames[loc.subgame].game.players.state_to_owner[loc.state] = TwoPlayer::Eve;
            }
        }

        let mut groups = Csr::with_capacity(grouping.groups.len());
        let mut group_states = To1::new();
        for group in &grouping.groups {
            let start = group_states.len();
            for &state in &group.states {
                if let Some(loc) = location[state] {
                    group_states.add(loc);
                }
            }
            group_states.entries_mut()[start..].sort_unstable();
            groups.add_entry_unchecked(GroupEntryIdx::from_raw(group_states.len()));
        }

        Self {
            subgames,
            values: goal.map(|&g| if g { 1.0 } else { 0.0 }),
            groups,
            group_states,
        }
    }
}

impl Subgame {
    /// Builds the subgame of the given states, removing self-loops by rescaling the remaining
    /// branches (except for choices that are a pure self-loop).
    fn new<M: ReadStateSpace<Ix = DefaultIx>>(
        model: &M,
        states: impl Iterator<Item = StateIdx>,
        subgame: SubgameIdx,
        location: &To1<StateIdx, Option<SubgameState>>,
    ) -> Self {
        let mut game = TwoPlayerStochasticGame::<SubIx>::default();
        let mut exit_values = To1::new();
        let mut exits = Csr::new();
        let mut exit_branches: To1<ExitIdx, _> = To1::new();
        let mut to_global = To1::new();

        for global in states {
            let state = game.graph.add_state();
            game.players.add_state_owner(state, TwoPlayer::Adam);
            to_global.add(global);

            for choice in model.choices_of_state(global) {
                let sub_choice = game.graph.add_choice();
                let first_exit = exit_branches.len();
                let mut to_self = 0.0;
                for branch in model.branches_of_choice(choice) {
                    let p = model.branch_probability(branch);
                    let destination = model.branch_destination(branch);
                    match location[destination] {
                        Some(loc) if loc.subgame == subgame && loc.state == state => to_self += p,
                        Some(loc) if loc.subgame == subgame => {
                            game.graph.add_branch(p, loc.state);
                        }
                        _ => {
                            exit_branches.add((destination, p));
                        }
                    }
                }

                if to_self == 1.0 {
                    game.graph.add_branch(1.0, state);
                } else if to_self > 0.0 {
                    let scale = 1.0 / (1.0 - to_self);
                    for branch in game.graph.branches_of_choice(sub_choice) {
                        game.graph.branch_probabilities[branch] *= scale;
                    }
                    for (_, p) in &mut exit_branches.entries_mut()[first_exit..] {
                        *p *= scale;
                    }
                }
                exits.add_entry_unchecked(ExitIdx::from_raw(exit_branches.len()));
                exit_values.add(0.0);
            }
        }

        Self {
            game,
            exit_values,
            exits,
            exit_branches,
            to_global,
        }
    }
}
