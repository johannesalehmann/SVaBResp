use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use prism_model::{Expression, VariableRange, VariableReference};
use probabilistic_models::{
    Action, ActionCollection, AtomicProposition, AtomicPropositions, Builder, Distribution,
    DistributionBuilder, ModelTypes, Predecessors, PredecessorsBuilder, ProbabilisticModel, State,
    Successor, TwoPlayer, Valuation, VectorPredecessors,
};
use probabilistic_properties::Property;

pub struct ActionGroupExtractionScheme {
    action_index: Option<VariableReference>,
    in_questionmark_state: Option<VariableReference>,
}

impl ActionGroupExtractionScheme {
    pub fn new() -> Self {
        Self {
            action_index: None,
            in_questionmark_state: None,
        }
    }
}

impl super::GroupExtractionScheme for ActionGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(&mut self, prism_model: &mut PrismModel, property: &mut PrismProperty) {
        let _ = property;
        // Add two variables to the PRISM code that will later be used during model construction to
        // assign unique values to additional auxiliary states. Adding the variables at this stage
        // is easier than adding them after the model builder has run
        use prism_model::{Identifier, VariableInfo};
        let span = chumsky::span::SimpleSpan::new(0, 0);
        self.action_index = Some(
            prism_model
                .variable_manager
                .add_variable(VariableInfo::with_initial_value(
                    Identifier::new("action_index", span).unwrap(),
                    VariableRange::UnboundedInt { span },
                    false,
                    None,
                    Expression::Int(0, span),
                    span,
                ))
                .unwrap(),
        );
        self.in_questionmark_state = Some(
            prism_model
                .variable_manager
                .add_variable(VariableInfo::new(
                    Identifier::new("in_questionmark_state", span).unwrap(),
                    VariableRange::Boolean { span },
                    false,
                    None,
                    span,
                ))
                .unwrap(),
        );
    }

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Property<AtomicProposition, f64>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let _ = property;

        let mut state_groups = Vec::with_capacity(game.action_names.len());
        for action_name in &game.action_names {
            state_groups.push((action_name.clone(), Vec::new()));
        }
        let mut helper_state_group = Vec::new();
        let mut adversary_state_group = Vec::new();

        let action_index_variable = self.action_index.unwrap();
        let in_questionmark_state_variable = self.in_questionmark_state.unwrap();

        let continue_action_index = game.action_names.len();
        game.action_names
            .push("continue_to_next_action".to_string());

        let try_action_index = game.action_names.len();
        game.action_names.push("try_activate_action".to_string());

        let back_action_index = game.action_names.len();
        game.action_names.push("do_not_use_action".to_string());

        for state_index in 0..game.states.len() {
            let state = &game.states[state_index];
            let base_owner = state.owner;
            let base_atomic_propositions = M::AtomicPropositions::from_other(
                game.atomic_proposition_count,
                &state.atomic_propositions,
            );
            let base_valuation = state.valuation.clone();
            let mut targets = Vec::new();
            let mut action_name_indices = Vec::new();
            for action in state.actions.iter() {
                assert_eq!(
                    action.successors.number_of_successors(),
                    1,
                    "The action grouping construction does not support probabilistic choice"
                );
                targets.push(action.successors.get_successor(0).index);
                action_name_indices.push(action.action_name_index);
            }

            let action_count = state.actions.get_number_of_actions();
            if action_count == 0 {
                adversary_state_group.push(state_index);
            }

            for action_index in 0..action_count {
                let n = if action_index == 0 {
                    game.states.len()
                } else {
                    game.states.len() + 1
                };
                {
                    let mut normal_state_actions = <M::ActionCollection>::get_builder();
                    let mut next_normal = <M::Distribution>::get_builder();
                    next_normal.add_successor(Successor {
                        index: n + 1,
                        probability: 1.0,
                    });
                    normal_state_actions.add_action(Action {
                        successors: next_normal.finish(),
                        action_name_index: continue_action_index,
                    });

                    let mut questionmark = <M::Distribution>::get_builder();
                    questionmark.add_successor(Successor {
                        index: n,
                        probability: 1.0,
                    });
                    normal_state_actions.add_action(Action {
                        successors: questionmark.finish(),
                        action_name_index: try_action_index,
                    });

                    let mut valuation = base_valuation.clone();
                    valuation.set_unbounded_int(action_index_variable.index, action_index as i64);
                    let normal_state = State {
                        valuation,
                        actions: normal_state_actions.finish(),
                        atomic_propositions: M::AtomicPropositions::from_other(
                            game.atomic_proposition_count,
                            &base_atomic_propositions,
                        ),
                        owner: base_owner,
                        predecessors: <<M::Predecessors as Predecessors>::Builder>::create()
                            .finish(),
                    };

                    if action_index == 0 {
                        state_groups[action_name_indices[action_index]]
                            .1
                            .push(state_index);
                        game.states[state_index] = normal_state;
                    } else {
                        state_groups[action_name_indices[action_index]]
                            .1
                            .push(game.states.len());
                        game.states.push(normal_state);
                    }
                }

                {
                    let mut questionmark_actions = <M::ActionCollection>::get_builder();
                    let mut next_normal = <M::Distribution>::get_builder();
                    next_normal.add_successor(Successor {
                        index: n + 1,
                        probability: 1.0,
                    });
                    questionmark_actions.add_action(Action {
                        successors: next_normal.finish(),
                        action_name_index: back_action_index,
                    });

                    let mut follow_action = <M::Distribution>::get_builder();
                    follow_action.add_successor(Successor {
                        index: targets[action_index],
                        probability: 1.0,
                    });
                    questionmark_actions.add_action(Action {
                        successors: follow_action.finish(),
                        action_name_index: action_name_indices[action_index],
                    });

                    let mut valuation = base_valuation.clone();
                    valuation.set_unbounded_int(action_index_variable.index, action_index as i64);
                    valuation.set_bool(in_questionmark_state_variable.index, true);
                    let questionmark_state = State {
                        valuation,
                        actions: questionmark_actions.finish(),
                        atomic_propositions: M::AtomicPropositions::from_other(
                            game.atomic_proposition_count,
                            &base_atomic_propositions,
                        ),
                        owner: base_owner,
                        predecessors: <<M::Predecessors as Predecessors>::Builder>::create()
                            .finish(),
                    };
                    helper_state_group.push(game.states.len());
                    game.states.push(questionmark_state);
                }
            }

            {
                let mut adversarial_actions = <M::ActionCollection>::get_builder();
                for (&action_name_index, &target) in action_name_indices.iter().zip(targets.iter())
                {
                    let mut target_distribution = <M::Distribution>::get_builder();
                    target_distribution.add_successor(Successor {
                        index: target,
                        probability: 1.0,
                    });
                    adversarial_actions.add_action(Action {
                        successors: target_distribution.finish(),
                        action_name_index,
                    });
                }

                let mut valuation = base_valuation.clone();
                valuation.set_unbounded_int(action_index_variable.index, action_count as i64);
                let adversarial_state = State {
                    valuation,
                    actions: adversarial_actions.finish(),
                    atomic_propositions: M::AtomicPropositions::from_other(
                        game.atomic_proposition_count,
                        &base_atomic_propositions,
                    ),
                    owner: base_owner,
                    predecessors: <<M::Predecessors as Predecessors>::Builder>::create().finish(),
                };
                adversary_state_group.push(game.states.len());
                game.states.push(adversarial_state);
            }
        }

        game.rebuild_predecessors();

        let mut builder = Self::GroupType::get_builder();
        for (group_name, states) in state_groups {
            builder.create_group_from_vec(states, group_name);
        }

        GroupsAndAuxiliary::with_auxiliary(
            builder.finish(),
            helper_state_group,
            adversary_state_group,
        )
    }
}
