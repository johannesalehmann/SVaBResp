use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use chumsky::span::SimpleSpan;
use prism_model::{
    Assignment, Command, Expression, Identifier, Module, Update, VariableInfo, VariableRange,
    VariableReference,
};
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, Valuation, VectorPredecessors,
};
use probabilistic_properties::Property;
use std::collections::HashMap;

pub struct ModuleExtractionScheme {
    group_count: Option<usize>, // The number of groups includes the scheduler group, one group per module and one per synchronising action
    selected_module_variable: Option<VariableReference>,
    group_names: Vec<String>,
}

impl ModuleExtractionScheme {
    pub fn new() -> Self {
        Self {
            group_count: None,
            selected_module_variable: None,
            group_names: Vec::new(),
        }
    }
}

impl super::GroupExtractionScheme for ModuleExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(&mut self, prism_model: &mut PrismModel, property: &mut PrismProperty) {
        let _ = property;

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
            let mut module_action_infos = HashMap::new();
            for command in &module.commands {
                if let Some(action) = &command.action {
                    if module_action_infos.contains_key(&action.name) {
                        let current_guard: &mut Expression<_, _> =
                            module_action_infos.get_mut(&action.name).unwrap();
                        *current_guard = Expression::Disjunction(
                            Box::new(command.guard.clone()),
                            Box::new(current_guard.clone()),
                            span,
                        );
                    } else {
                        module_action_infos.insert(action.name.clone(), command.guard.clone());
                    }
                }
            }
            for (name, guard) in module_action_infos {
                if action_infos.contains_key(&name) {
                    action_infos
                        .get_mut(&name)
                        .unwrap()
                        .module_guards
                        .push(guard);
                } else {
                    action_infos.insert(
                        name,
                        ActionInfo {
                            module_guards: vec![guard],
                        },
                    );
                }
            }
        }

        let mut scheduler = Module::new(Identifier::new("scheduler", span).unwrap(), span);
        self.group_names.push("scheduler".to_string());

        for (module_index, module) in prism_model.modules.modules.iter_mut().enumerate() {
            self.group_names.push(module.name.name.clone());
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
                self.group_names.push(action.clone());

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
        property: &Property<AtomicProposition, f64>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let _ = property;

        let group_count = self.group_count.unwrap();
        let selected_module_variable = self.selected_module_variable.unwrap();

        let mut groups = Vec::with_capacity(group_count);
        for _ in 0..group_count {
            groups.push(Vec::new());
        }

        for (index, state) in game.states.iter().enumerate() {
            let value = state
                .valuation
                .evaluate_unbounded_int(selected_module_variable.index)
                as usize;
            groups[value].push(index);
        }

        let mut builder = Self::GroupType::get_builder();
        for (group_name, group) in self.group_names.drain(..).zip(groups.into_iter()) {
            builder.create_group_from_vec(group, group_name);
        }

        GroupsAndAuxiliary::new(builder.finish())
    }
}

struct ActionInfo {
    pub module_guards: Vec<Expression<VariableReference, SimpleSpan>>,
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
