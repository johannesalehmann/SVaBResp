use crate::grouping::StateIdx;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::{BuildChoiceBranchGraph, BuildStates, ReadStateSpace};
use probabilistic_models::typed_index_collections::{Csr, To1, index};
use probabilistic_models::{BranchIndex, ChoiceIndex, DefaultIx, Index, Ix, StateIndex, Sub};
use std::ops::Range;

pub type SubIx = Sub<DefaultIx>;
pub type SubStateIdx = StateIndex<SubIx>;
pub type SubChoiceIdx = ChoiceIndex<SubIx>;
pub type SubBranchIdx = BranchIndex<SubIx>;

index!(ExitIndex<I: Ix> => I::Branch);
pub type ExitIdx = ExitIndex<SubIx>;

index!(ValuePositionIndex<I: Ix> => I::State);
pub type ValuePositionIdx = ValuePositionIndex<DefaultIx>;

/// The model restricted to a single SCC.
pub struct Subgame {
    first_position: ValuePositionIdx,
    mdp: Mdp<SubIx>,
    choice_exits: Csr<SubChoiceIdx, ExitIdx>,
    /// Destination and rescaled probability.
    exit_branches: To1<ExitIdx, (ValuePositionIdx, f64)>,
}

impl Subgame {
    /// Builds the subgame of `model_states`, which occupy `subgame_positions` of the value vector.
    /// Successors outside the subgame must already have their final value position.
    pub fn new<M: ReadStateSpace<Ix = DefaultIx>>(
        model: &M,
        model_states: impl Iterator<Item = StateIdx>,
        subgame_positions: Range<ValuePositionIdx>,
        value_position: &To1<StateIdx, ValuePositionIdx>,
    ) -> Self {
        let mut mdp = Mdp::<SubIx>::default();
        let mut choice_exits = Csr::new();
        let mut exit_branches: To1<ExitIdx, _> = To1::new();

        for model_state in model_states {
            let local_state = mdp.graph.add_state();
            let model_choices = model.choices_of_state(model_state);
            if model_choices.len() == 0 {
                mdp.graph.add_choice();
                choice_exits.add_entry_unchecked(ExitIdx::from_raw(exit_branches.len()));
            }

            for model_choice in model_choices {
                let local_choice = mdp.graph.add_choice();
                let first_exit = exit_branches.len();
                let mut self_loop_probability = 0.0;
                for branch in model.branches_of_choice(model_choice) {
                    let probability = model.branch_probability(branch);
                    let destination_position = value_position[model.branch_destination(branch)];
                    if !subgame_positions.contains(&destination_position) {
                        exit_branches.add((destination_position, probability));
                        continue;
                    }
                    let local_destination = SubStateIdx::from_raw(
                        destination_position.raw() - subgame_positions.start.raw(),
                    );
                    if local_destination == local_state {
                        self_loop_probability += probability;
                    } else {
                        mdp.graph.add_branch(probability, local_destination);
                    }
                }

                if self_loop_probability > 0.0 && self_loop_probability < 1.0 {
                    let scale = 1.0 / (1.0 - self_loop_probability);
                    for branch in mdp.graph.branches_of_choice(local_choice) {
                        mdp.graph.branch_probabilities[branch] *= scale;
                    }
                    for (_, probability) in &mut exit_branches.entries_mut()[first_exit..] {
                        *probability *= scale;
                    }
                }
                choice_exits.add_entry_unchecked(ExitIdx::from_raw(exit_branches.len()));
            }
        }

        Self {
            first_position: subgame_positions.start,
            mdp,
            choice_exits,
            exit_branches,
        }
    }

    pub fn value_position(&self, local_state: SubStateIdx) -> ValuePositionIdx {
        self.first_position + local_state.raw()
    }

    /// Solves the subgame in place. The states controlled by Eve maximise, the others minimise.
    ///
    /// The value vector is not reset before solving, so it must contain a valid lower bound of the
    /// solution.
    pub fn solve(
        &self,
        owned_by_eve: &To1<ValuePositionIdx, bool>,
        exit_value_buffer: &mut To1<SubChoiceIdx, f64>,
        values: &mut To1<ValuePositionIdx, f64>,
        eps: f64,
    ) {
        let graph = &self.mdp.graph;
        exit_value_buffer.clear();
        for choice in graph.choices() {
            let exit_value = (self.choice_exits.index(choice).into_iter())
                .map(|exit| {
                    let (destination_position, probability) = self.exit_branches[exit];
                    probability * values[destination_position]
                })
                .sum();
            exit_value_buffer.add(exit_value);
        }

        let subgame_positions = self.first_position..self.first_position + graph.states().len();
        let owned_by_eve = &owned_by_eve[subgame_positions.clone()];
        let values = &mut values[subgame_positions];
        let probabilities = &graph.branch_probabilities;
        let destinations = &graph.branch_destinations;
        loop {
            let mut converged = true;
            let mut choice = SubChoiceIdx::from_raw(0);
            let mut branch = SubBranchIdx::from_raw(0);
            for (state, &choice_end) in graph.state_to_choice.entries_raw().iter().enumerate() {
                let maximise = owned_by_eve[state];
                let mut best_value = if maximise { 0.0 } else { f64::INFINITY };
                while choice < choice_end {
                    let mut choice_value = exit_value_buffer[choice];
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

                let absolute_error = best_value - values[state];
                if absolute_error > 0.0 && absolute_error >= eps * best_value {
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
