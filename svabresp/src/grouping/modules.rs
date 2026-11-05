use super::{Grouping, GroupingScheme, StateGroup};
use prism_model::{
    Assignment, Command, Expression, FullSpan, Identifier, Module, Update, VariableInfo,
    VariableRange, VariableReference,
};
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::valuations::ValuationBits;
use std::collections::BTreeMap;
use tiny_pmc::{API, OptionalModel, PrismModel, PrismQuery};

// TODO: Store this in the grouping scheme instead of as a constant. This allows adapting to cases
//  where the variable name already exists.
/// The name of the auxiliary PRISM variable that records which module the scheduler activated.
const ACTIVE_MODULE_VARIABLE: &str = "_active_module";

/// Groups states by the module (or synchronising action) that is executed next.
///
/// The PRISM model is extended by a scheduler module. In a state with `_active_module = 0`, the
/// scheduler selects a module (or synchronising action) by setting `_active_module` to its index.
/// In the following state, only that module (or action) can execute, after which
/// `_active_module` is reset to `0`. The states with `_active_module = 0` form the scheduler
/// group, while the states with `_active_module = i` form the group of the `i`-th module or action.
pub struct ModuleGrouping {
    /// One name per group: the scheduler, every module and every synchronising action.
    group_names: Vec<String>,
}

impl ModuleGrouping {
    pub fn new() -> Self {
        Self {
            group_names: Vec::new(),
        }
    }
}

impl GroupingScheme for ModuleGrouping {
    fn requires_prism(&self) -> bool {
        true
    }

    fn transform_prism(&mut self, prism_model: &mut PrismModel, _query: &mut PrismQuery) {
        let active_module = prism_model
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

        // Collect the guard of every action in every module. A BTreeMap ensures that the groups
        // are created in a deterministic order.
        let mut action_infos: BTreeMap<String, ActionInfo> = BTreeMap::new();
        for module in prism_model.modules.iter() {
            let mut module_action_guards: BTreeMap<String, Expression<_, _>> = BTreeMap::new();
            for command in &module.commands {
                if let Some(action) = &command.action {
                    let guard = match module_action_guards.remove(&action.name) {
                        Some(current_guard) => command.guard.clone().or(current_guard),
                        None => command.guard.clone(),
                    };
                    module_action_guards.insert(action.name.clone(), guard);
                }
            }
            for (name, guard) in module_action_guards {
                action_infos
                    .entry(name)
                    .or_insert_with(|| ActionInfo {
                        module_guards: Vec::new(),
                    })
                    .module_guards
                    .push(guard);
            }
        }

        let mut scheduler = Module::new(Identifier::new("scheduler").unwrap());
        self.group_names.push("scheduler".to_string());

        for (module_index, module) in prism_model.modules.iter_mut().enumerate() {
            self.group_names.push(module.name.name.clone());
            let execute_action = format!("execute_module_{}", module_index);
            let mut guard = Expression::bool(false);
            for command in &mut module.commands {
                let is_synchronising = command
                    .action
                    .as_ref()
                    .is_some_and(|action| action_infos[&action.name].is_synchronising());
                if !is_synchronising {
                    guard = guard.or(command.guard.clone());
                    command.action = Some(Identifier::new(execute_action.clone()).unwrap());
                }
            }

            add_scheduler_commands(
                &mut scheduler,
                active_module,
                guard,
                module_index + 1,
                Identifier::new(execute_action).unwrap(),
            );
        }

        for (action, action_info) in action_infos {
            if action_info.is_synchronising() {
                let index = self.group_names.len();
                self.group_names.push(action.clone());
                add_scheduler_commands(
                    &mut scheduler,
                    active_module,
                    action_info.into_guard(),
                    index,
                    Identifier::new(action).unwrap(),
                );
            }
        }

        prism_model.modules.add(scheduler).unwrap();
    }

    fn create_groups(
        &mut self,
        model: &mut OptionalModel,
        _query: &ProcessedQuery<API>,
    ) -> Grouping {
        let valuations = model
            .state_valuations
            .as_ref()
            .expect("Grouping by modules requires a model with state valuations");
        let (active_module, _) = super::variable(valuations, ACTIVE_MODULE_VARIABLE)
            .expect("The scheduler variable is missing from the model");

        let mut groups = self
            .group_names
            .iter()
            .map(|name| StateGroup {
                name: name.clone(),
                states: Vec::new(),
            })
            .collect::<Vec<_>>();

        let states = probabilistic_models::for_concrete_base!(model.as_ref(), |m| m.states());
        for state in states {
            let value = valuations.entry(state).evaluate_int(active_module) as usize;
            groups[value].states.push(state);
        }

        Grouping::new(groups)
    }
}

/// Adds the two scheduler commands for the group with index `index`:
/// - `[] guard & _active_module=0 -> (_active_module'=index)` selects the group and
/// - `[action] _active_module=index -> (_active_module'=0)` synchronises with the group's commands.
fn add_scheduler_commands(
    scheduler: &mut Module,
    active_module: VariableReference,
    guard: Expression<VariableReference, FullSpan>,
    index: usize,
    action: Identifier<FullSpan>,
) {
    let mut select_command = Command::new(None, guard.and(active_module_is(active_module, 0)));
    select_command.add_update(set_active_module(active_module, index as i64));
    scheduler.commands.push(select_command);

    let mut activate_command =
        Command::new(Some(action), active_module_is(active_module, index as i64));
    activate_command.add_update(set_active_module(active_module, 0));
    scheduler.commands.push(activate_command);
}

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

struct ActionInfo {
    /// For every module that uses the action, the disjunction of the guards of its commands with
    /// this action.
    module_guards: Vec<Expression<VariableReference, FullSpan>>,
}

impl ActionInfo {
    fn is_synchronising(&self) -> bool {
        self.module_guards.len() >= 2
    }

    fn into_guard(self) -> Expression<VariableReference, FullSpan> {
        self.module_guards
            .into_iter()
            .fold(Expression::bool(true), Expression::and)
    }
}
