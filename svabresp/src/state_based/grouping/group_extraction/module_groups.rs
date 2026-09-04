use crate::shapley::{ResponsibilityValues, SwitchingPairCollection};
use crate::state_based::game::{ApIdx, Game, variable_index};
use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use prism_model::{
    Assignment, Command, Expression, FullSpan, Identifier, Module, Span, Update, VariableInfo,
    VariableRange, VariableReference,
};
use probabilistic_models::traits::{ReadStateSpace, ReadValuations};
use probabilistic_models::valuations::ValuationBits;
use probabilistic_properties::Query;
use std::collections::HashMap;

// TODO: Store this in the extraction scheme instead of as a constant. This allows adapting to cases
//  where the variable name already exists.
/// The name of the auxiliary PRISM variable that records which module the scheduler activated.
const ACTIVE_MODULE_VARIABLE: &str = "_active_module";

/// Builds the expression `_active_module = value`.
fn active_module_is(
    variable: VariableReference,
    value: i64,
) -> Expression<VariableReference, FullSpan> {
    Expression::var_or_const(variable).equals_to(Expression::int(value))
}

/// Builds the update `1: (_active_module' = value)`.
fn set_active_module(
    variable: VariableReference,
    value: i64,
) -> Update<VariableReference, FullSpan> {
    Update::with_assignments(
        Expression::int(1),
        vec![Assignment::new(variable, Expression::int(value))],
    )
}

pub struct ModuleGroupInfo {
    name: String,
    spans: Vec<FullSpan>,
}

impl ModuleGroupInfo {
    pub fn new<S: Into<String>>(name: S, spans: Vec<FullSpan>) -> Self {
        Self {
            name: name.into(),
            spans,
        }
    }

    pub fn with_single_span<S: Into<String>>(name: S, span: FullSpan) -> Self {
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
        character_to_line: &prism_parser::CharacterToLineMap,
    ) {
        let _ = (property, character_to_line);

        let selected_module_variable = prism_model
            .variable_manager
            .add_variable(
                VariableInfo::local_var(
                    Identifier::new(ACTIVE_MODULE_VARIABLE).unwrap(),
                    VariableRange::unbounded_int(),
                    prism_model.modules.len(),
                )
                .initial_value(Expression::int(0)),
            )
            .unwrap();

        let mut action_infos: HashMap<String, ActionInfo> = HashMap::new();
        for module in prism_model.modules.iter() {
            let mut module_action_guards: HashMap<String, Expression<_, _>> = HashMap::new();
            let mut module_action_spans: HashMap<String, Vec<FullSpan>> = HashMap::new();
            for command in &module.commands {
                if let Some(action) = &command.action {
                    if module_action_guards.contains_key(&action.name) {
                        let current_guard = module_action_guards.get_mut(&action.name).unwrap();
                        *current_guard = command.guard.clone().or(current_guard.clone());
                        let current_spans = module_action_spans.get_mut(&action.name).unwrap();
                        current_spans.push(action.span.clone());
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

        let mut scheduler = Module::new(Identifier::new("scheduler").unwrap());
        self.group_info.push(ModuleGroupInfo::with_single_span(
            "scheduler",
            prism_model.model_type.span().clone(),
        ));

        for (module_index, module) in prism_model.modules.iter_mut().enumerate() {
            self.group_info.push(ModuleGroupInfo::with_single_span(
                module.name.name.clone(),
                module.name.span.clone(),
            ));
            let execute_action = format!("execute_module_{}", module_index);
            let mut guard = Expression::bool(false);
            for command in &mut module.commands {
                if command.action.is_none()
                    || !action_infos[&command.action.as_ref().unwrap().name].is_synchronising()
                {
                    guard = guard.or(command.guard.clone());
                    command.action = Some(Identifier::new(execute_action.clone()).unwrap());
                }
            }

            let guard = guard.and(active_module_is(selected_module_variable, 0));

            let mut select_command = Command::new(None, guard);
            select_command.add_update(set_active_module(
                selected_module_variable,
                module_index as i64 + 1,
            ));
            scheduler.commands.push(select_command);

            let mut activate_command = Command::new(
                Some(Identifier::new(execute_action.clone()).unwrap()),
                active_module_is(selected_module_variable, module_index as i64 + 1),
            );
            activate_command.add_update(set_active_module(selected_module_variable, 0));
            scheduler.commands.push(activate_command);
        }

        let mut index = prism_model.modules.len() + 1;
        for (action, action_info) in action_infos {
            if action_info.is_synchronising() {
                self.group_info
                    .push(ModuleGroupInfo::new(&action, action_info.spans.clone()));

                let guard = action_info
                    .get_guard()
                    .and(active_module_is(selected_module_variable, 0));
                let mut select_command = Command::new(None, guard);
                select_command
                    .add_update(set_active_module(selected_module_variable, index as i64));
                scheduler.commands.push(select_command);

                let mut activate_command = Command::new(
                    Some(Identifier::new(action).unwrap()),
                    active_module_is(selected_module_variable, index as i64),
                );
                activate_command.add_update(set_active_module(selected_module_variable, 0));
                scheduler.commands.push(activate_command);

                index += 1;
            }
        }

        prism_model.modules.add(scheduler).unwrap();

        self.group_count = Some(index);
        self.selected_module_variable = Some(selected_module_variable);
    }

    fn create_groups(
        &mut self,
        game: Game,
        property: &Query<i64, f64, ApIdx>,
    ) -> (Game, GroupsAndAuxiliary<Self::GroupType>) {
        let _ = property;

        let group_count = self.group_count.unwrap();

        let mut groups = Vec::with_capacity(group_count);
        for _ in 0..group_count {
            groups.push(Vec::new());
        }

        let active_module = variable_index(&game.state_valuations, ACTIVE_MODULE_VARIABLE)
            .expect("The scheduler variable is missing from the model");
        for state in game.states() {
            let value = game.state_valuation(state).evaluate_int(active_module) as usize;
            groups[value].push(state);
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

        (game, GroupsAndAuxiliary::new(builder.finish()))
    }

    fn get_syntax_elements<S: AsRef<str>>(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
        switching_pairs: &SwitchingPairCollection,
        player_names: &[S],
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting> {
        use crate::syntax_highlighting::*;
        let mut highlighting = SyntaxHighlighting::new();
        let colour_ramp_index = 0;

        let is_probabilistic = switching_pairs.contains_non_simple_pairs();

        let aggregated_switching_pairs = switching_pairs
            .clone()
            .aggregate_by_minimal_switching_pair();

        for group in &self.group_info {
            let (value, tooltip) = aggregated_switching_pairs.value_and_tool_tip_text(
                "Module",
                colour_ramp_index,
                &group.name,
                &values,
                player_names,
                is_probabilistic,
                false,
            );

            for span in &group.spans {
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

struct ActionInfo {
    pub module_guards: Vec<Expression<VariableReference, FullSpan>>,
    pub spans: Vec<FullSpan>,
}

impl ActionInfo {
    fn is_synchronising(&self) -> bool {
        self.module_guards.len() >= 2
    }

    fn get_guard(self) -> Expression<VariableReference, FullSpan> {
        let mut guard = Expression::bool(true);
        for module_guard in self.module_guards {
            guard = guard.and(module_guard);
        }
        guard
    }
}
