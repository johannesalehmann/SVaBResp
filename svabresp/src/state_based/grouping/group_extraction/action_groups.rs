use crate::shapley::{ResponsibilityValues, SwitchingPairCollection};
use crate::state_based::game::{
    ActionIdx, ApIdx, Game, GameChoiceLabels, PredecessorIdx, StateIdx, single_valuation_class,
    variable_index,
};
use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use prism_model::{Expression, FullSpan, Identifier, Span, VariableInfo, VariableRange};
use probabilistic_models::base_model::TwoPlayerTurnBasedGame;
use probabilistic_models::labels::ReadLabels;
use probabilistic_models::traits::{ReadStateSpace, ReadValuations};
use probabilistic_models::typed_index_collections::Index;
use probabilistic_models::valuations::{BareStandaloneValuation, ValuationBitsMut};
use probabilistic_properties::Query;
use std::collections::HashMap;

// TODO: Instead of using constants for the variable names, store them in the extraction schemes.
//  That way, we can handle the case where a variable with the name is already present.

/// The PRISM variable that distinguishes the auxiliary states belonging to one original state.
const ACTION_INDEX_VARIABLE: &str = "action_index_internal_var";

/// The PRISM variable that marks the auxiliary states in which the coalition is offered the
/// corresponding action.
const QUESTIONMARK_VARIABLE: &str = "in_questionmark_state_internal_variable";

pub struct ActionGroupExtractionScheme {
    action_name_to_spans: HashMap<String, Vec<FullSpan>>,
}

impl ActionGroupExtractionScheme {
    pub fn new() -> Self {
        Self {
            action_name_to_spans: HashMap::new(),
        }
    }
}

impl super::GroupExtractionScheme for ActionGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(
        &mut self,
        prism_model: &mut PrismModel,
        property: &mut PrismProperty,
        character_to_line: &prism_parser::CharacterToLineMap,
    ) {
        let _ = property;
        // Add two variables to the PRISM code that will later be used during model construction to
        // assign unique values to additional auxiliary states. Adding the variables at this stage
        // is easier than adding them after the model builder has run
        prism_model
            .variable_manager
            .add_variable(
                VariableInfo::global_var(
                    Identifier::new(ACTION_INDEX_VARIABLE).unwrap(),
                    VariableRange::unbounded_int(),
                )
                .initial_value(Expression::int(0)),
            )
            .unwrap();

        prism_model
            .variable_manager
            .add_variable(VariableInfo::global_var(
                Identifier::new(QUESTIONMARK_VARIABLE).unwrap(),
                VariableRange::bool(),
            ))
            .unwrap();

        let mut last_line = None;
        let mut in_line_counter = 0;
        prism_model.name_unnamed_actions_with_custom_name(|span| {
            let line = span
                .range()
                .map(|range| character_to_line.get_line(range.start))
                .unwrap_or(0);
            if last_line == Some(line) {
                in_line_counter += 1;
            } else {
                in_line_counter = 0;
            }
            last_line = Some(line);
            let suffix = if in_line_counter == 0 {
                "".to_string()
            } else {
                format!("_{}", in_line_counter)
            };
            Identifier::new(format!("unnamed_action_line_{}{}", line, suffix)).unwrap()
        });

        for module in prism_model.modules.iter() {
            for command in &module.commands {
                let span = match &command.action {
                    None => command.action_span.clone(),
                    // We use action.span instead of command.action_span here so the enclosing
                    // square brackets are not highlighted. This indicates that all instances of the
                    // action share a single responsibility value
                    Some(action) => action.span.clone(),
                };
                let name = command
                    .action
                    .as_ref()
                    .map(|a| a.name.clone())
                    .unwrap_or("unnamed".to_string()); // TODO: This might break if the model builder changes the name assigned to unnamed actions. Find a more robust way to handle this.
                if let Some(spans) = self.action_name_to_spans.get_mut(&name) {
                    spans.push(span);
                } else {
                    self.action_name_to_spans.insert(name, vec![span]);
                }
            }
        }
    }

    fn create_groups(
        &mut self,
        game: Game,
        property: &Query<i64, f64, ApIdx>,
    ) -> (Game, GroupsAndAuxiliary<Self::GroupType>) {
        let _ = property;

        // TODO: This function is largely AI-written after a major refactor. At the time of writing,
        //  tiny-pmc does not expose a robust interface to modify existing models. Thus, it is not
        //  possible to refactor the existing code in a nice way, thus yielding the highly complex
        //  and hard-to-reason-about code that follows.
        //  Once an interface to modify existing models exists, this function should likely be
        //  rewritten from scratch (based off the old function from before the refactor). Until this
        //  is the case, keep in mind that what follows has not been vetted as thoroughly as the
        //  rest of the refactor.

        // Every original state `s` with actions `a_0, ..., a_{k-1}` is expanded into a chain in
        // which the coalition is offered one action at a time:
        //
        //   N_j --try--> Q_j --a_j--> (original successors of a_j)
        //    |            |
        //    | continue   | do_not_use
        //    v            v
        //   N_{j+1} (or A_s once every action has been offered)
        //
        // `N_j` belongs to the group of action `a_j`, `Q_j` is always controlled by the coalition
        // and `A_s`, which offers every action, is always controlled by the adversary. `N_0` keeps
        // the index of the original state so that transitions into `s` need not be rewritten; the
        // remaining states are appended after all original states.
        let old_state_count = game.states().len();

        let choice_count_of_state: Vec<usize> = game
            .states()
            .into_iter()
            .map(|state| game.choices_of_state(state).len())
            .collect();

        // The first index of the auxiliary states belonging to each original state. A state with
        // `k >= 1` actions contributes `Q_0, N_1, Q_1, ..., N_{k-1}, Q_{k-1}, A_s`, i.e. `2k`
        // states; a state without actions contributes only its (unreachable) `A_s`.
        let mut extras_start = Vec::with_capacity(old_state_count);
        let mut new_state_count = old_state_count;
        for &k in &choice_count_of_state {
            extras_start.push(new_state_count);
            new_state_count += if k == 0 { 1 } else { 2 * k };
        }

        let index = |i: usize| StateIdx::from_raw(i as u32);
        let normal = |s: usize, j: usize| {
            if j == 0 {
                index(s)
            } else {
                index(extras_start[s] + 2 * j - 1)
            }
        };
        let questionmark = |s: usize, j: usize| index(extras_start[s] + 2 * j);
        let adversary = |s: usize| {
            let k = choice_count_of_state[s];
            index(extras_start[s] + if k == 0 { 0 } else { 2 * k - 1 })
        };
        // Where `N_j` and `Q_j` go once action `a_j` has been declined.
        let after = |s: usize, j: usize| {
            if j + 1 < choice_count_of_state[s] {
                normal(s, j + 1)
            } else {
                adversary(s)
            }
        };

        // --- Choice labels ------------------------------------------------------------------
        // Actions are copied in order, so the original action indices stay valid.
        let original_action_count = game
            .choices()
            .into_iter()
            .map(|choice| game.choice_labels.action_index(choice).raw() + 1)
            .max()
            .unwrap_or(0);

        let mut labels = GameChoiceLabels::new();
        let mut group_names = Vec::with_capacity(original_action_count);
        for action in 0..original_action_count {
            let label = game
                .choice_labels
                .label_of_action(ActionIdx::from_raw(action))
                .clone();
            group_names.push(label.clone().unwrap_or_else(|| "unnamed".to_string()));
            labels.add_action(label);
        }
        let continue_action = labels.add_action(Some("continue_to_next_action".to_string()));
        let try_action = labels.add_action(Some("try_activate_action".to_string()));
        let back_action = labels.add_action(Some("do_not_use_action".to_string()));

        // --- Transition structure -----------------------------------------------------------
        let mut base = TwoPlayerTurnBasedGame::<StateIdx, _, _>::default();
        let mut state_groups: Vec<Vec<StateIdx>> = vec![Vec::new(); original_action_count];
        let mut helper_states = Vec::new();
        let mut adversary_states = Vec::new();
        // The original state each auxiliary state is derived from, in index order. Used below to
        // copy valuations and atomic propositions.
        let mut extra_base_states = Vec::with_capacity(new_state_count - old_state_count);
        // Whether each auxiliary state is a "questionmark" state, and which action it belongs to.
        let mut extra_action_index = Vec::with_capacity(new_state_count - old_state_count);
        let mut extra_is_questionmark = Vec::with_capacity(new_state_count - old_state_count);

        let action_of = |s: usize, j: usize| {
            game.choice_labels
                .action_index(game.choices_of_state(index(s)).index(j))
        };

        let add_choice = |base: &mut TwoPlayerTurnBasedGame<_, _, _>,
                          labels: &mut GameChoiceLabels,
                          action: ActionIdx| {
            let choice = base.add_choice();
            labels.label_entity(choice, action);
        };

        // Copy the branches of the `j`-th original choice of state `s`.
        let copy_branches =
            |base: &mut TwoPlayerTurnBasedGame<StateIdx, _, _>, s: usize, j: usize| {
                let choice = game.choices_of_state(index(s)).index(j);
                for branch in game.branches_of_choice(choice) {
                    base.add_branch(
                        game.branch_probability(branch),
                        game.branch_destination(branch),
                    );
                }
            };

        // Pass 1: the original states, which become `N_0`.
        for s in 0..old_state_count {
            let owner = game.base.owners[index(s)];
            base.add_state(index(s), owner);
            let k = choice_count_of_state[s];
            if k == 0 {
                adversary_states.push(index(s));
                continue;
            }
            state_groups[action_of(s, 0).raw()].push(index(s));

            add_choice(&mut base, &mut labels, continue_action);
            base.add_branch(1.0, after(s, 0));

            add_choice(&mut base, &mut labels, try_action);
            base.add_branch(1.0, questionmark(s, 0));
        }

        // Pass 2: the auxiliary states, in the index order fixed by `extras_start`.
        for s in 0..old_state_count {
            let owner = game.base.owners[index(s)];
            let k = choice_count_of_state[s];

            for j in 0..k {
                if j > 0 {
                    // `N_j`
                    base.add_state(normal(s, j), owner);
                    extra_base_states.push(index(s));
                    extra_action_index.push(j);
                    extra_is_questionmark.push(false);
                    state_groups[action_of(s, j).raw()].push(normal(s, j));

                    add_choice(&mut base, &mut labels, continue_action);
                    base.add_branch(1.0, after(s, j));

                    add_choice(&mut base, &mut labels, try_action);
                    base.add_branch(1.0, questionmark(s, j));
                }

                // `Q_j`
                base.add_state(questionmark(s, j), owner);
                extra_base_states.push(index(s));
                extra_action_index.push(j);
                extra_is_questionmark.push(true);
                helper_states.push(questionmark(s, j));

                add_choice(&mut base, &mut labels, back_action);
                base.add_branch(1.0, after(s, j));

                add_choice(&mut base, &mut labels, action_of(s, j));
                copy_branches(&mut base, s, j);
            }

            // `A_s`, which offers every original action to the adversary.
            base.add_state(adversary(s), owner);
            extra_base_states.push(index(s));
            extra_action_index.push(k);
            extra_is_questionmark.push(false);
            adversary_states.push(adversary(s));

            for j in 0..k {
                add_choice(&mut base, &mut labels, action_of(s, j));
                copy_branches(&mut base, s, j);
            }
        }

        // --- Valuations ---------------------------------------------------------------------
        // The auxiliary states inherit the valuation of the state they were derived from, with the
        // two bookkeeping variables set. Building the new valuations before appending them keeps
        // the immutable borrow of the old model separate from the mutable one.
        let valuation_class = single_valuation_class(&game.state_valuations);
        let action_index_variable = variable_index(&game.state_valuations, ACTION_INDEX_VARIABLE)
            .expect("The auxiliary action index variable is missing from the built model");
        let questionmark_variable = variable_index(&game.state_valuations, QUESTIONMARK_VARIABLE)
            .expect("The auxiliary questionmark variable is missing from the built model");
        let _ = valuation_class;

        let extra_valuations: Vec<BareStandaloneValuation<_, _>> = extra_base_states
            .iter()
            .zip(extra_action_index.iter())
            .zip(extra_is_questionmark.iter())
            .map(|((&base_state, &action_index), &is_questionmark)| {
                let entry = game.state_valuation(base_state);
                let mut valuation = entry.clone_into_standalone_valuation();
                valuation.set_int(action_index_variable, action_index as i64);
                if is_questionmark {
                    valuation.set_bool(questionmark_variable, true);
                }
                valuation.bare()
            })
            .collect();

        // --- Atomic propositions ------------------------------------------------------------
        let ap_indices: Vec<ApIdx> = game
            .atomic_propositions
            .internal_indices()
            .into_iter()
            .collect();
        let ap_values: Vec<Vec<bool>> = ap_indices
            .iter()
            .map(|&ap| {
                extra_base_states
                    .iter()
                    .map(|&base_state| game.atomic_propositions.entries()[ap][base_state])
                    .collect()
            })
            .collect();

        // --- Assemble the new game ----------------------------------------------------------
        let Game {
            initial,
            mut atomic_propositions,
            mut state_valuations,
            ..
        } = game;

        for (offset, valuation) in extra_valuations.iter().enumerate() {
            state_valuations.add_valuation(index(old_state_count + offset), valuation);
        }

        for (&ap, values) in ap_indices.iter().zip(ap_values) {
            let annotation = atomic_propositions
                .get_mut(ap)
                .expect("atomic proposition disappeared while rebuilding the model");
            for (offset, value) in values.into_iter().enumerate() {
                annotation.add_value(index(old_state_count + offset), value);
            }
        }

        let game = probabilistic_models::Model {
            base,
            initial,
            choice_labels: labels,
            branch_labels: (),
            observations: (),
            atomic_propositions,
            rewards: (),
            annotations: (),
            state_valuations,
            predecessors: (),
        }
        .compute_predecessors::<PredecessorIdx>();

        let mut builder = Self::GroupType::get_builder();
        for (group_name, states) in group_names.into_iter().zip(state_groups) {
            builder.create_group_from_vec(states, group_name);
        }

        (
            game,
            GroupsAndAuxiliary::with_auxiliary(builder.finish(), helper_states, adversary_states),
        )
    }

    fn get_syntax_elements<S: AsRef<str>>(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
        switching_pairs: &SwitchingPairCollection,
        player_names: &[S],
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting> {
        use crate::syntax_highlighting::*;
        let mut highlighting = SyntaxHighlighting::new();

        let colour_ramp_index = 1;

        let is_probabilistic = switching_pairs.contains_non_simple_pairs();

        let aggregated_switching_pairs = switching_pairs
            .clone()
            .aggregate_by_minimal_switching_pair();

        for (group_name, spans) in &self.action_name_to_spans {
            let (value, tooltip) = aggregated_switching_pairs.value_and_tool_tip_text(
                "Action",
                colour_ramp_index,
                group_name,
                &values,
                player_names,
                is_probabilistic,
                false,
            );

            for span in spans {
                if let Some(range) = span.range() {
                    highlighting.add_highlight(Highlight::new(
                        range.start,
                        range.end,
                        Colour::new(colour_ramp_index, value),
                        &tooltip,
                    ));
                }
            }
        }

        Some(highlighting)
    }
}
