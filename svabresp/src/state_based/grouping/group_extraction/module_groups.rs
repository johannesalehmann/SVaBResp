use crate::shapley::{CoalitionSpecifier, ResponsibilityValues, SwitchingPairCollection};
use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use chumsky::span::SimpleSpan;
use prism_model::{
    Assignment, Command, Expression, Identifier, Module, Update, VariableInfo, VariableRange,
    VariableReference,
};
use probabilistic_models::{
    AtomicProposition, Context, ModelTypes, ProbabilisticModel, TwoPlayer, Valuation,
    VectorPredecessors,
};
use probabilistic_properties::Query;
use std::collections::HashMap;

pub struct ModuleGroupInfo {
    name: String,
    spans: Vec<SimpleSpan>,
}

impl ModuleGroupInfo {
    pub fn new<S: Into<String>>(name: S, spans: Vec<SimpleSpan>) -> Self {
        Self {
            name: name.into(),
            spans,
        }
    }

    pub fn with_single_span<S: Into<String>>(name: S, span: SimpleSpan) -> Self {
        Self {
            name: name.into(),
            spans: vec![span],
        }
    }
}

pub struct ModuleGroupExtractionScheme {
    group_count: Option<usize>, // The number of groups includes the scheduler group, one group per module and one per synchronising action
    selected_module_variable: Option<VariableReference>,
    group_info: Vec<ModuleGroupInfo>,
}

impl ModuleGroupExtractionScheme {
    pub fn new() -> Self {
        Self {
            group_count: None,
            selected_module_variable: None,
            group_info: Vec::new(),
        }
    }
}

impl super::GroupExtractionScheme for ModuleGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(
        &mut self,
        prism_model: &mut PrismModel,
        property: &mut PrismProperty,

        atomic_propositions: &mut Vec<prism_model::Expression<VariableReference, SimpleSpan>>,
    ) {
        let _ = (property, atomic_propositions);

        let span = SimpleSpan::new(0, 0);
        let selected_module_variable = prism_model
            .variable_manager
            .add_variable(VariableInfo::with_initial_value(
                Identifier::new("_active_module", span).unwrap(),
                VariableRange::UnboundedInt { span },
                false,
                Some(prism_model.modules.modules.len()),
                Expression::Int(0, span),
                span,
            ))
            .unwrap();

        let mut action_infos: HashMap<String, ActionInfo> = HashMap::new();
        for module in &prism_model.modules.modules {
            let mut module_action_guards: HashMap<String, Expression<_, _>> = HashMap::new();
            let mut module_action_spans: HashMap<String, Vec<SimpleSpan>> = HashMap::new();
            for command in &module.commands {
                if let Some(action) = &command.action {
                    if module_action_guards.contains_key(&action.name) {
                        let current_guard = module_action_guards.get_mut(&action.name).unwrap();
                        *current_guard = Expression::Disjunction(
                            Box::new(command.guard.clone()),
                            Box::new(current_guard.clone()),
                            span,
                        );
                        let current_spans = module_action_spans.get_mut(&action.name).unwrap();
                        current_spans.push(action.span);
                    } else {
                        module_action_guards.insert(action.name.clone(), command.guard.clone());
                        module_action_spans.insert(action.name.clone(), vec![action.span.clone()]);
                    }
                }
            }
            for (name, guard) in module_action_guards {
                let spans = module_action_spans.get_mut(&name).unwrap();
                if let Some(action_info) = action_infos.get_mut(&name) {
                    action_info.module_guards.push(guard);
                    action_info.spans.append(spans)
                } else {
                    action_infos.insert(
                        name,
                        ActionInfo {
                            module_guards: vec![guard],
                            spans: spans.clone(),
                        },
                    );
                }
            }
        }

        let mut scheduler = Module::new(Identifier::new("scheduler", span).unwrap(), span);
        self.group_info.push(ModuleGroupInfo::with_single_span(
            "scheduler",
            prism_model.model_type.get_span().clone(),
        ));

        for (module_index, module) in prism_model.modules.modules.iter_mut().enumerate() {
            self.group_info.push(ModuleGroupInfo::with_single_span(
                module.name.name.clone(),
                module.name.span.clone(),
            ));
            let execute_action = format!("execute_module_{}", module_index);
            let mut guard = Expression::Bool(false, span);
            for command in &mut module.commands {
                if command.action.is_none()
                    || !action_infos[&command.action.as_ref().unwrap().name].is_synchronising()
                {
                    guard = Expression::Disjunction(
                        Box::new(guard),
                        Box::new(command.guard.clone()),
                        span,
                    );
                    command.action = Some(Identifier::new(execute_action.clone(), span).unwrap());
                }
            }

            let guard = Expression::Conjunction(
                Box::new(guard),
                Box::new(Expression::Equals(
                    Box::new(Expression::VarOrConst(selected_module_variable, span)),
                    Box::new(Expression::Int(0, span)),
                    span,
                )),
                span,
            );

            let mut select_command = Command::new(None, guard, span);
            select_command.updates.push(Update::with_assignments(
                Expression::Int(1, span),
                vec![Assignment::new(
                    selected_module_variable,
                    Expression::Int(module_index as i64 + 1, span),
                    span,
                    span,
                )],
                span,
            ));
            scheduler.commands.push(select_command);

            let mut activate_command = Command::new(
                Some(Identifier::new(execute_action.clone(), span).unwrap()),
                Expression::Equals(
                    Box::new(Expression::VarOrConst(selected_module_variable, span)),
                    Box::new(Expression::Int(module_index as i64 + 1, span)),
                    span,
                ),
                span,
            );
            activate_command.updates.push(Update::with_assignments(
                Expression::Int(1, span),
                vec![Assignment::new(
                    selected_module_variable,
                    Expression::Int(0, span),
                    span,
                    span,
                )],
                span,
            ));
            scheduler.commands.push(activate_command);
        }

        let mut index = 1 + prism_model.modules.modules.len();
        for (action, action_info) in action_infos {
            if action_info.is_synchronising() {
                self.group_info
                    .push(ModuleGroupInfo::new(&action, action_info.spans.clone()));

                let guard = Expression::Conjunction(
                    Box::new(action_info.get_guard()),
                    Box::new(Expression::Equals(
                        Box::new(Expression::VarOrConst(selected_module_variable, span)),
                        Box::new(Expression::Int(0, span)),
                        span,
                    )),
                    span,
                );
                let mut select_command = Command::new(None, guard, span);
                select_command.updates.push(Update::with_assignments(
                    Expression::Int(1, span),
                    vec![Assignment::new(
                        selected_module_variable,
                        Expression::Int(index as i64, span),
                        span,
                        span,
                    )],
                    span,
                ));
                scheduler.commands.push(select_command);

                let mut activate_command = Command::new(
                    Some(Identifier::new(action, span).unwrap()),
                    Expression::Equals(
                        Box::new(Expression::VarOrConst(selected_module_variable, span)),
                        Box::new(Expression::Int(index as i64, span)),
                        span,
                    ),
                    span,
                );
                activate_command.updates.push(Update::with_assignments(
                    Expression::Int(1, span),
                    vec![Assignment::new(
                        selected_module_variable,
                        Expression::Int(0, span),
                        span,
                        span,
                    )],
                    span,
                ));
                scheduler.commands.push(activate_command);

                index += 1;
            }
        }

        prism_model.modules.add(scheduler).unwrap();

        self.group_count = Some(index);
        self.selected_module_variable = Some(selected_module_variable);
    }

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Query<i64, f64, AtomicProposition>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let _ = property;

        let group_count = self.group_count.unwrap();

        let mut groups = Vec::with_capacity(group_count);
        for _ in 0..group_count {
            groups.push(Vec::new());
        }

        for (index, state) in game.states.iter().enumerate() {
            // TODO: Using get_variable_count() - 1 as index is a hack that assumes that last-added
            // variable is also the last variable in the valuation. We cannot use
            // self.selected_module_variable.index, as the variable manager index does not
            // correspond to the index in the valuation (the former includes constants, the latter
            // does not)
            let value = state
                .valuation
                .evaluate_unbounded_int(game.valuation_context.get_variable_count() - 1)
                as usize;
            groups[value].push(index);
        }

        let mut builder = Self::GroupType::get_builder();
        for (group_name, group) in self
            .group_info
            .iter()
            .map(|g| g.name.clone())
            .zip(groups.into_iter())
        {
            builder.create_group_from_vec(group, group_name);
        }

        GroupsAndAuxiliary::new(builder.finish())
    }

    fn get_syntax_elements<S: AsRef<str>>(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
        switching_pairs: &SwitchingPairCollection,
        player_names: &[S],
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting> {
        use crate::syntax_highlighting::*;
        let mut highlighting = SyntaxHighlighting::new();

        let aggregated_switching_pairs = switching_pairs
            .clone()
            .aggregate_by_minimal_switching_pair();

        for group in &self.group_info {
            let (value, tooltip_start) = if let Some(responsibility) = values.get(&group.name) {
                (
                    responsibility.value,
                    format!("Responsibility: {}", round_float(responsibility.value)),
                )
            } else {
                (0.0, "No responsibility".to_string())
            };

            let mut tooltip_text = vec![tooltip_start];

            let player_index = values.get_index(&group.name);
            if let Some(player_index) = player_index {
                let switching_pairs = aggregated_switching_pairs.switching_pairs(player_index);
                if switching_pairs.len() > 0 {
                    tooltip_text.push("\n\n**Switching pairs**".to_string());
                }

                for switching_pair in switching_pairs {
                    tooltip_text.push("\n\n- ".to_string());
                    tooltip_text.push(CoalitionSpecifier::to_string(
                        &switching_pair.coalition,
                        player_names,
                    ));
                    tooltip_text.push(format!(
                        "\n\n    Contribution: {}",
                        round_float(switching_pair.direct_contribution)
                    ));
                    tooltip_text.push(format!(
                        "\n\n    Value: {} - {} = {}",
                        round_float(switching_pair.value_with),
                        round_float(switching_pair.value_without),
                        round_float(switching_pair.value()),
                    ));
                    if switching_pair.indirect_contribution > 0.0 {
                        let superset_pair_text = if switching_pair.aggregated_pair_count == 1 {
                            "superset pair is"
                        } else {
                            "superset pairs are"
                        };
                        tooltip_text.push(format!(
                            "\n\n    {} {} hidden (contribution: {})",
                            switching_pair.aggregated_pair_count,
                            superset_pair_text,
                            round_float(switching_pair.indirect_contribution)
                        ));
                    }
                }
            }

            let tooltip = tooltip_text.join("");

            for span in &group.spans {
                highlighting.add_highlight(Highlight::new(
                    span.start,
                    span.end,
                    Colour::new(0, value),
                    &tooltip,
                ));
            }
        }

        Some(highlighting)
    }
}

fn round_float(value: f64) -> f64 {
    (value * 1000.0).round() * 0.001
}

struct ActionInfo {
    pub module_guards: Vec<Expression<VariableReference, SimpleSpan>>,
    pub spans: Vec<SimpleSpan>,
}

impl ActionInfo {
    fn is_synchronising(&self) -> bool {
        self.module_guards.len() > 0
    }

    fn get_guard(self) -> Expression<VariableReference, SimpleSpan> {
        let mut guard = Expression::Bool(true, SimpleSpan::new(0, 0));
        for module_guard in self.module_guards {
            guard = Expression::Conjunction(
                Box::new(guard),
                Box::new(module_guard),
                SimpleSpan::new(0, 0),
            );
        }
        guard
    }
}
