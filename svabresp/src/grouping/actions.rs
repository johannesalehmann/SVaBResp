use super::{Grouping, GroupingScheme, StateGroup, StateIdx};
use prism_model::{Expression, Identifier, VariableInfo, VariableRange};
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::base_model::{BaseModelEnum, BaseModelType, Mdp};
use probabilistic_models::labels::{ChoiceLabels, ReadLabels};
use probabilistic_models::traits::{BuildChoiceBranchGraph, BuildStates, ReadStateSpace};
use probabilistic_models::valuations::ValuationBitsMut;
use probabilistic_models::{ActionIndex, DefaultIx, Index, RawIndex};
use tiny_pmc::{API, OptionalModel, PrismModel, PrismQuery};

// TODO: Instead of using constants for the variable names, store them in the grouping scheme.
//  That way, we can handle the case where a variable with the name is already present.

/// The PRISM variable that distinguishes the auxiliary states belonging to one original state.
const ACTION_INDEX_VARIABLE: &str = "action_index_internal_var";

/// The PRISM variable that marks the auxiliary states in which the coalition is offered the
/// corresponding action.
const QUESTIONMARK_VARIABLE: &str = "in_questionmark_state_internal_variable";

type Labels = ChoiceLabels<DefaultIx, Option<String>>;
type ActionIdx = ActionIndex<DefaultIx>;

/// Groups states by action: The group of an action decides whether that action is available.
///
/// Every original state `s` with actions `a_0, ..., a_{k-1}` is expanded into a chain in which the
/// coalition is offered one action at a time:
///
/// ```text
///   N_j --try--> Q_j --a_j--> (original successors of a_j)
///    |            |
///    | continue   | do_not_use
///    v            v
///   N_{j+1} (or A_s once every action has been offered)
/// ```
///
/// `N_j` belongs to the group of action `a_j`, `Q_j` is always controlled by the coalition and
/// `A_s`, which offers every action, is always controlled by the adversary. `N_0` keeps the index of
/// the original state so that transitions into `s` need not be rewritten; the remaining states are
/// appended after all original states.
pub struct ActionGrouping;

impl GroupingScheme for ActionGrouping {
    fn requires_prism(&self) -> bool {
        true
    }

    fn transform_prism(&mut self, prism_model: &mut PrismModel, _query: &mut PrismQuery) {
        // Add two variables to the PRISM code that are used to assign unique valuations to the
        // auxiliary states. Adding the variables at this stage is easier than adding them after
        // the model builder has run.
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
            .add_variable(
                VariableInfo::global_var(
                    Identifier::new(QUESTIONMARK_VARIABLE).unwrap(),
                    VariableRange::bool(),
                )
                .initial_value(Expression::bool(false)),
            )
            .unwrap();

        // Every action becomes a group, so unnamed actions need a name. Since unnamed actions never
        // synchronise, giving each one a unique name does not change the semantics.
        prism_model.name_unnamed_actions();
    }

    fn create_groups(
        &mut self,
        model: &mut OptionalModel,
        _query: &ProcessedQuery<API>,
    ) -> Grouping {
        if matches!(
            model.base.model_type(),
            BaseModelType::NonstochasticGame | BaseModelType::StochasticGame
        ) {
            panic!("Grouping by actions is not supported for games");
        }

        let labels = model
            .choice_labels
            .as_ref()
            .expect("Grouping by actions requires a model with choice labels");
        let expansion =
            probabilistic_models::for_concrete_base!(model.as_ref(), |m| expand(&m, labels));
        let old_state_count = expansion.old_state_count;

        // The auxiliary states inherit the valuation of the state they were derived from, with the
        // two bookkeeping variables set. Building the new valuations before appending them keeps
        // the immutable borrow of the old valuations separate from the mutable one.
        if let Some(valuations) = model.state_valuations.as_mut() {
            let (action_index_variable, _) = super::variable(valuations, ACTION_INDEX_VARIABLE)
                .expect("The auxiliary action index variable is missing from the built model");
            let (questionmark_variable, _) = super::variable(valuations, QUESTIONMARK_VARIABLE)
                .expect("The auxiliary questionmark variable is missing from the built model");

            let extra_valuations = expansion
                .extra_states
                .iter()
                .map(|extra| {
                    let entry = valuations.entry(extra.original_state);
                    let mut valuation = entry.clone_into_standalone_valuation();
                    valuation.set_int(action_index_variable, extra.action_index as i64);
                    valuation.set_bool(questionmark_variable, extra.is_questionmark);
                    valuation.bare()
                })
                .collect::<Vec<_>>();

            for (offset, valuation) in extra_valuations.iter().enumerate() {
                valuations.add_valuation(state_index(old_state_count + offset), valuation);
            }
        }

        // The auxiliary states also inherit the atomic propositions of their original state.
        if let Some(atomic_propositions) = model.atomic_propositions.as_mut() {
            for ap in atomic_propositions.internal_indices() {
                let annotation = &mut atomic_propositions[ap];
                for (offset, extra) in expansion.extra_states.iter().enumerate() {
                    let value = annotation[extra.original_state];
                    annotation.add_value(state_index(old_state_count + offset), value);
                }
            }
        }

        if model
            .rewards
            .as_ref()
            .is_some_and(|rewards| rewards.len() > 0)
        {
            log::warn!("Grouping by actions discards the reward structures of the model");
        }

        model.base = BaseModelEnum::Mdp(expansion.base);
        model.choice_labels = Some(expansion.labels);
        model.branch_labels = None;
        model.rewards = None;
        model.predecessors = None;

        Grouping {
            groups: expansion.groups,
            always_helping: expansion.always_helping,
            always_adversarial: expansion.always_adversarial,
        }
    }
}

fn state_index(index: usize) -> StateIdx {
    StateIdx::from_raw(index)
}

/// An auxiliary state created by the expansion.
struct ExtraState {
    /// The original state this auxiliary state is derived from.
    original_state: StateIdx,
    /// `j` for `N_j` and `Q_j`, and the number of actions for `A_s`.
    action_index: usize,
    is_questionmark: bool,
}

struct Expansion {
    old_state_count: usize,
    base: Mdp,
    labels: Labels,
    groups: Vec<StateGroup>,
    always_helping: Vec<StateIdx>,
    always_adversarial: Vec<StateIdx>,
    /// The auxiliary states, in index order starting at `old_state_count`.
    extra_states: Vec<ExtraState>,
}

fn expand<M: ReadStateSpace<Ix = DefaultIx>>(model: &M, old_labels: &Labels) -> Expansion {
    let old_state_count = model.states().len();

    let choice_count_of_state: Vec<usize> = model
        .states()
        .into_iter()
        .map(|state| model.choices_of_state(state).len())
        .collect();

    // The first index of the auxiliary states belonging to each original state. A state with
    // `k >= 1` actions contributes `Q_0, N_1, Q_1, ..., N_{k-1}, Q_{k-1}, A_s`, i.e. `2k` states; a
    // state without actions contributes only its (unreachable) `A_s`.
    let mut extras_start = Vec::with_capacity(old_state_count);
    let mut new_state_count = old_state_count;
    for &k in &choice_count_of_state {
        extras_start.push(new_state_count);
        new_state_count += if k == 0 { 1 } else { 2 * k };
    }

    let index = state_index;
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
    let choice_of = |s: usize, j: usize| model.choices_of_state(index(s)).index(j);
    let action_of = |s: usize, j: usize| old_labels.action_index(choice_of(s, j));

    // --- Choice labels ------------------------------------------------------------------------
    // Actions are copied in order, so the original action indices stay valid.
    let original_action_count = model
        .choices()
        .into_iter()
        .map(|choice| old_labels.action_index(choice).raw().as_usize() + 1)
        .max()
        .unwrap_or(0);

    let mut labels = Labels::new();
    let mut groups = Vec::with_capacity(original_action_count);
    for action in 0..original_action_count {
        let label = old_labels
            .label_of_action(ActionIdx::from_raw(RawIndex::from_usize(action)))
            .clone();
        groups.push(StateGroup {
            name: label.clone().unwrap_or_else(|| "unnamed".to_string()),
            states: Vec::new(),
        });
        labels.add_action(label);
    }
    let continue_action = labels.add_action(Some("continue_to_next_action".to_string()));
    let try_action = labels.add_action(Some("try_activate_action".to_string()));
    let back_action = labels.add_action(Some("do_not_use_action".to_string()));

    // --- Transition structure -----------------------------------------------------------------
    let mut base = Mdp::new();
    let mut always_helping = Vec::new();
    let mut always_adversarial = Vec::new();
    let mut extra_states = Vec::with_capacity(new_state_count - old_state_count);

    let add_state = |base: &mut Mdp, expected: StateIdx| {
        let state = BuildStates::add_state(base);
        assert_eq!(
            state, expected,
            "Auxiliary states were created out of order"
        );
    };
    let add_choice = |base: &mut Mdp, labels: &mut Labels, action: ActionIdx| {
        let choice = base.add_choice();
        labels.label_entity(choice, action);
    };
    // Copies the `j`-th original choice of state `s`, including its label.
    let copy_choice = |base: &mut Mdp, labels: &mut Labels, s: usize, j: usize| {
        add_choice(base, labels, action_of(s, j));
        for branch in model.branches_of_choice(choice_of(s, j)) {
            base.add_branch(
                model.branch_probability(branch),
                model.branch_destination(branch),
            );
        }
    };

    // Pass 1: the original states, which become `N_0`.
    for s in 0..old_state_count {
        add_state(&mut base, index(s));
        if choice_count_of_state[s] == 0 {
            always_adversarial.push(index(s));
            continue;
        }
        groups[action_of(s, 0).raw().as_usize()]
            .states
            .push(index(s));

        add_choice(&mut base, &mut labels, continue_action);
        base.add_branch(1.0, after(s, 0));

        add_choice(&mut base, &mut labels, try_action);
        base.add_branch(1.0, questionmark(s, 0));
    }

    // Pass 2: the auxiliary states, in the index order fixed by `extras_start`.
    for s in 0..old_state_count {
        let k = choice_count_of_state[s];

        for j in 0..k {
            if j > 0 {
                // `N_j`
                add_state(&mut base, normal(s, j));
                extra_states.push(ExtraState {
                    original_state: index(s),
                    action_index: j,
                    is_questionmark: false,
                });
                groups[action_of(s, j).raw().as_usize()]
                    .states
                    .push(normal(s, j));

                add_choice(&mut base, &mut labels, continue_action);
                base.add_branch(1.0, after(s, j));

                add_choice(&mut base, &mut labels, try_action);
                base.add_branch(1.0, questionmark(s, j));
            }

            // `Q_j`
            add_state(&mut base, questionmark(s, j));
            extra_states.push(ExtraState {
                original_state: index(s),
                action_index: j,
                is_questionmark: true,
            });
            always_helping.push(questionmark(s, j));

            add_choice(&mut base, &mut labels, back_action);
            base.add_branch(1.0, after(s, j));

            copy_choice(&mut base, &mut labels, s, j);
        }

        // `A_s`, which offers every original action to the adversary.
        add_state(&mut base, adversary(s));
        extra_states.push(ExtraState {
            original_state: index(s),
            action_index: k,
            is_questionmark: false,
        });
        always_adversarial.push(adversary(s));

        for j in 0..k {
            copy_choice(&mut base, &mut labels, s, j);
        }
    }

    Expansion {
        old_state_count,
        base,
        labels,
        groups,
        always_helping,
        always_adversarial,
        extra_states,
    }
}
