mod expressions;
mod variables;

use crate::expressions::{Evaluator, TreeWalkingEvaluator, ValuationSource, VariableType};
use crate::variables::{ConstAndVarValuationSource, ModelVariableInfo};
use log::info;
use prism_model::{
    Expression, Identifier, Model, Update, VariableManager, VariableRange, VariableReference,
};
use probabilistic_models::probabilistic_properties::{ProbabilityOperator, Property};
use probabilistic_models::{
    Action, ActionCollection, AtomicProposition, AtomicPropositions, Builder, Distribution,
    InitialStates, InitialStatesBuilder, MdpType, ModelTypes, Predecessors, PredecessorsBuilder,
    ProbabilisticModel, State, Successor, Valuation, ValuationBuilder,
};
use probabilistic_models::{DistributionBuilder, Predecessor};
use std::collections::HashMap;

pub fn build_model<S: Clone, M: ModelTypes>(
    model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    atomic_propositions: &[Expression<VariableReference, S>],
    const_values: &HashMap<String, UserProvidedConstValue>,
) -> Result<ProbabilisticModel<M>, ModelBuildingError> {
    ExplicitModelBuilder::<M>::run(model, atomic_propositions, const_values)
}
pub fn build_properties<
    S: Clone,
    I: Iterator<Item = Property<AtomicProposition, Expression<VariableReference, S>>>,
>(
    model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    properties: I,
    const_values: &HashMap<String, UserProvidedConstValue>,
) -> Result<Vec<Property<AtomicProposition, f64>>, ModelBuildingError> {
    ExplicitModelBuilder::<MdpType>::build_property(model, properties, const_values)
}

pub enum UserProvidedConstValue {
    Int(i64),
    Bool(bool),
    Float(f64),
}

pub struct StateInProgress<M: ModelTypes> {
    pub valuation: M::Valuation,
    pub actions: <M::ActionCollection as ActionCollection<M::Distribution>>::Builder,
    pub atomic_propositions: M::AtomicPropositions,
    pub predecessors: <M::Predecessors as Predecessors>::Builder,
}
pub struct ExplicitModelBuilder<M: ModelTypes> {
    states: Vec<StateInProgress<M>>,
    valuation_to_state: HashMap<M::Valuation, usize>,
    open_states: Vec<usize>,
    variable_info: variables::ModelVariableInfo<M::Valuation>,
    action_names: Vec<String>,
    action_name_indices: HashMap<String, usize>,
}

impl<M: ModelTypes> ExplicitModelBuilder<M> {
    pub fn build_property<
        S: Clone,
        I: Iterator<Item = Property<AtomicProposition, Expression<VariableReference, S>>>,
    >(
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        properties: I,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
    ) -> Result<Vec<Property<AtomicProposition, f64>>, ModelBuildingError> {
        let variable_info: variables::ModelVariableInfo<M::Valuation> =
            variables::ModelVariableInfo::new(model, user_provided_consts)?;

        let const_valuation_source = variable_info.get_const_only_valuation_source();

        let mut result = Vec::new();
        for property in properties {
            let constraint = property
                .operator
                .constraint
                .map_probability_specifier_with_result(|p| {
                    Ok(TreeWalkingEvaluator::new().evaluate_as_float(&p, &const_valuation_source))
                })?;

            result.push(Property {
                operator: ProbabilityOperator {
                    kind: property.operator.kind,
                    constraint,
                },
                path: property.path,
            })
        }
        Ok(result)
    }

    pub fn run<S: Clone>(
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        atomic_propositions: &[Expression<VariableReference, S>],
        const_values: &HashMap<String, UserProvidedConstValue>,
    ) -> Result<ProbabilisticModel<M>, ModelBuildingError> {
        let start_time = std::time::Instant::now();
        let variable_info = variables::ModelVariableInfo::new(model, const_values)?;

        let synchronised_actions = SynchronisedActions::from_prism(model);

        let mut builder = Self {
            states: Vec::new(),
            valuation_to_state: HashMap::new(),
            open_states: Vec::new(),
            variable_info,
            action_names: Vec::new(),
            action_name_indices: HashMap::new(),
        };

        let initial_states = builder.create_initial_states(model, atomic_propositions.len())?;

        while let Some(state) = builder.open_states.pop() {
            builder.process_state(state, &model, atomic_propositions, &synchronised_actions)?;
        }

        let mut initial_states_builder = M::InitialStates::get_builder();
        for initial_state in initial_states {
            initial_states_builder.add_by_index(initial_state)
        }
        let initial_states = initial_states_builder.finish();

        let mut result = ProbabilisticModel::new(
            initial_states,
            builder.variable_info.valuation_context,
            atomic_propositions.len(),
        );
        result.action_names = builder.action_names;
        for state_in_progress in builder.states.into_iter() {
            let state = State {
                valuation: state_in_progress.valuation,
                actions: state_in_progress.actions.finish(),
                atomic_propositions: state_in_progress.atomic_propositions,
                owner: <M::Owners as probabilistic_models::Owners>::default_owner(),
                predecessors: state_in_progress.predecessors.finish(),
            };
            result.states.push(state);
        }

        info!(
            "Model built in {:?} ({} states)",
            start_time.elapsed(),
            result.states.len()
        );
        Ok(result)
    }

    fn get_unnamed_action_name_index(&mut self) -> usize {
        self.get_action_name_index("unnamed")
    }

    fn get_action_name_index(&mut self, name: &str) -> usize {
        if let Some(&index) = self.action_name_indices.get(name) {
            index
        } else {
            let index = self.action_names.len();
            self.action_names.push(name.to_string());
            self.action_name_indices.insert(name.to_string(), index);
            index
        }
    }

    fn get_or_add_state(
        &mut self,
        valuation: M::Valuation,
        atomic_proposition_len: usize,
    ) -> usize {
        let index = self.valuation_to_state.get(&valuation);
        match index {
            Some(&index) => index,
            None => {
                let index = self.states.len();
                let action_builder: <M::ActionCollection as ActionCollection<M::Distribution>>::Builder =
                    M::ActionCollection::get_builder();
                let atomic_propositions =
                    <M::AtomicPropositions>::get_empty(atomic_proposition_len);
                let predecessors = <M::Predecessors as Predecessors>::Builder::create();
                self.valuation_to_state.insert(valuation.clone(), index);
                self.states.push(StateInProgress {
                    valuation,
                    actions: action_builder,
                    atomic_propositions,
                    predecessors,
                });
                self.open_states.push(index);
                index
            }
        }
    }

    fn process_state<S: Clone>(
        &mut self,
        state: usize,
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        atomic_propositions: &[Expression<VariableReference, S>],
        synchronised_actions: &SynchronisedActions,
    ) -> Result<(), ModelBuildingError> {
        self.evaluate_atomic_propositions(state, atomic_propositions);

        let mut action_index = 0;

        for module_index in 0..model.modules.modules.len() {
            let module = &model.modules.modules[module_index];
            for command_index in 0..module.commands.len() {
                let command = &module.commands[command_index];
                if command.action.is_some() {
                    continue; // Synchronising actions are handled separately
                }
                let valuation = &self.states[state].valuation;
                let val_source = self.variable_info.get_valuation_source(valuation);
                let guard =
                    TreeWalkingEvaluator::new().evaluate_as_bool(&command.guard, &val_source);
                if guard {
                    let mut distribution = <M::Distribution as Distribution>::get_builder();

                    for update_index in 0..command.updates.len() {
                        let valuation = &self.states[state].valuation;
                        let val_source = self.variable_info.get_valuation_source(valuation);

                        let update = &command.updates[update_index];
                        let probability = TreeWalkingEvaluator::new()
                            .evaluate_as_float(&update.probability, &val_source);
                        let new_valuation = self.apply_assignments(
                            &model.variable_manager,
                            valuation,
                            &val_source,
                            &[&update],
                        );

                        let index = self.get_or_add_state(new_valuation, atomic_propositions.len());
                        distribution.add_successor(Successor { probability, index });

                        self.states[index].predecessors.add(Predecessor {
                            from: state,
                            action_index,
                            probability,
                        });
                    }

                    let action_name_index = self.get_unnamed_action_name_index();
                    let successors = distribution.finish();
                    if successors.number_of_successors() == 0 {
                        println!(
                            "State {} a local action with zero successors",
                            self.states[state]
                                .valuation
                                .displayable(&self.variable_info.valuation_context)
                        )
                    }
                    self.states[state].actions.add_action(Action {
                        successors,
                        action_name_index,
                    });
                    action_index += 1;
                }
            }
        }

        for synchronised_action in &synchronised_actions.actions {
            let action_name_index = self.get_action_name_index(&synchronised_action.name);

            let valuation = &self.states[state].valuation;
            let val_source = self.variable_info.get_valuation_source(valuation);

            let mut satisfied_guards_indices = Vec::new();
            let mut all_satisfied = true;
            for action_module in &synchronised_action.participating_modules {
                let module = &model.modules.modules[action_module.module_index];
                let mut module_info = Vec::new();
                for &command_index in &action_module.command_indices {
                    let command = &module.commands[command_index];
                    let guard =
                        TreeWalkingEvaluator::new().evaluate_as_bool(&command.guard, &val_source);
                    if guard {
                        module_info.push(command_index);
                    }
                }
                if module_info.is_empty() {
                    all_satisfied = false;
                }
                satisfied_guards_indices.push(module_info);
            }

            let n = satisfied_guards_indices.len();

            if n == 0 {
                panic!(
                    "Synchronised actions with zero associated modules are not yet supported (but they should be impossible to create anyways)"
                );
            }

            if all_satisfied {
                let modules = &synchronised_action.participating_modules;
                let mut indices = vec![0; n];
                while indices[0] < satisfied_guards_indices[0].len() {
                    let mut command_indices = Vec::with_capacity(n);
                    for i in 0..n {
                        command_indices.push(satisfied_guards_indices[i][indices[i]]);
                    }

                    let mut update_indices = vec![0; n];

                    let mut distribution = <M::Distribution as Distribution>::get_builder();

                    while update_indices[0]
                        < model.modules.modules[modules[0].module_index].commands
                            [command_indices[0]]
                            .updates
                            .len()
                            .max(1)
                    // max(1) is required because a synchronising action may have an empty update ("true")
                    {
                        let valuation = &self.states[state].valuation;
                        let val_source = self.variable_info.get_valuation_source(valuation);
                        let mut updates = Vec::new();
                        for i in 0..n {
                            let command = &model.modules.modules[modules[i].module_index].commands
                                [command_indices[i]];
                            if command.updates.len() > 0 {
                                updates.push(&command.updates[update_indices[i]]);
                            }
                        }
                        let new_valuation = self.apply_assignments(
                            &model.variable_manager,
                            valuation,
                            &val_source,
                            &updates[..],
                        );

                        let mut probability = 1.0;

                        for i in 0..n {
                            let command = &model.modules.modules[modules[i].module_index].commands
                                [command_indices[i]];
                            if command.updates.len() > 0 {
                                let ith_expression =
                                    &command.updates[update_indices[i]].probability;
                                let ith_probability = TreeWalkingEvaluator::new()
                                    .evaluate_as_float(ith_expression, &val_source);
                                probability *= ith_probability;
                            }
                        }

                        let index = self.get_or_add_state(new_valuation, atomic_propositions.len());
                        distribution.add_successor(Successor { probability, index });
                        self.states[index].predecessors.add(Predecessor {
                            from: state,
                            action_index,
                            probability,
                        });

                        for i in (0..n).rev() {
                            if update_indices[i] + 1
                                < model.modules.modules[modules[i].module_index].commands
                                    [command_indices[i]]
                                    .updates
                                    .len()
                            {
                                update_indices[i] += 1;
                                for j in i + 1..n {
                                    update_indices[j] = 0;
                                }
                                break;
                            } else {
                                if i == 0 {
                                    update_indices[0] += 1;
                                }
                            }
                        }
                    }

                    self.states[state].actions.add_action(Action {
                        successors: distribution.finish(),
                        action_name_index,
                    });
                    action_index += 1;

                    for i in (0..n).rev() {
                        if indices[i] + 1 < satisfied_guards_indices[i].len() {
                            indices[i] += 1;
                            for j in i + 1..n {
                                indices[j] = 0;
                            }
                            break;
                        } else {
                            if i == 0 {
                                indices[0] += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn evaluate_atomic_propositions<S: Clone>(
        &mut self,
        state_index: usize,
        atomic_propositions: &[Expression<VariableReference, S>],
    ) {
        let state = &mut self.states[state_index];
        let val_source = self.variable_info.get_valuation_source(&state.valuation);
        for (i, atomic_proposition) in atomic_propositions.iter().enumerate() {
            let is_true =
                TreeWalkingEvaluator::new().evaluate_as_bool(&atomic_proposition, &val_source);
            state.atomic_propositions.set_value(i, is_true);
        }
    }

    fn apply_assignments<S: Clone>(
        &self,
        variable_manager: &VariableManager<Expression<VariableReference, S>, S>,
        valuation: &<M as ModelTypes>::Valuation,
        val_source: &ConstAndVarValuationSource<<M as ModelTypes>::Valuation>,
        updates: &[&Update<Expression<VariableReference, S>, VariableReference, S>],
    ) -> <M as ModelTypes>::Valuation {
        let mut new_valuation = valuation.clone();
        for update in updates {
            for assignment in &update.assignments {
                let target = variable_manager.get(&assignment.target).unwrap();
                let target_index = self
                    .variable_info
                    .valuation_map
                    .map_to_variable(assignment.target.index)
                    .expect("Cannot assign to constant");
                match target.range {
                    VariableRange::BoundedInt { .. } => {
                        let value = TreeWalkingEvaluator::new()
                            .evaluate_as_int(&assignment.value, &val_source);
                        let (min, max) = self.variable_info.details[target_index].bounds.unwrap();
                        if value < min || value > max {
                            panic!(
                                "Value exceeds variable bounds, bounds are ({}, {}), value is {}",
                                min, max, value
                            );
                        } else {
                            new_valuation.set_bounded_int(target_index, value);
                        }
                    }
                    VariableRange::UnboundedInt { .. } => {
                        let value = TreeWalkingEvaluator::new()
                            .evaluate_as_int(&assignment.value, &val_source);
                        new_valuation.set_unbounded_int(target_index, value);
                    }
                    VariableRange::Boolean { .. } => {
                        let value = TreeWalkingEvaluator::new()
                            .evaluate_as_bool(&assignment.value, &val_source);
                        new_valuation.set_bool(target_index, value);
                    }
                    VariableRange::Float { .. } => {
                        let value = TreeWalkingEvaluator::new()
                            .evaluate_as_float(&assignment.value, &val_source);
                        new_valuation.set_float(target_index, value);
                    }
                }
            }
        }
        new_valuation
    }

    fn create_initial_states<S: Clone>(
        &mut self,
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        atomic_proposition_len: usize,
    ) -> Result<Vec<usize>, ModelBuildingError> {
        if model.init_constraint.is_some() {
            panic!("Init constraints are not yet supported by the model builder");
        }
        let const_value_source = self.variable_info.get_const_only_valuation_source();

        let mut valuation_builder =
            M::Valuation::get_builder(&self.variable_info.valuation_context);

        for (i, variable) in model.variable_manager.variables.iter().enumerate() {
            if let Some(index) = &self.variable_info.valuation_map.map_to_variable(i) {
                match variable.range {
                    VariableRange::BoundedInt { .. } => match &variable.initial_value {
                        None => {
                            if let Some((min, _)) = self.variable_info.details[*index].bounds {
                                valuation_builder.add_bounded_int(min);
                            } else {
                                panic!("Variable bounds list is inconsistent");
                            }
                        }
                        Some(initial) => {
                            let value = TreeWalkingEvaluator::new()
                                .evaluate_as_int(initial, &const_value_source);
                            valuation_builder.add_bounded_int(value);
                        }
                    },
                    VariableRange::UnboundedInt { .. } => match &variable.initial_value {
                        None => panic!("Unbounded int must have init expression"),
                        Some(initial) => {
                            let value = TreeWalkingEvaluator::new()
                                .evaluate_as_int(initial, &const_value_source);
                            valuation_builder.add_int(value);
                        }
                    },
                    VariableRange::Boolean { .. } => match &variable.initial_value {
                        None => {
                            valuation_builder.add_bool(false);
                        }
                        Some(initial) => {
                            let value = TreeWalkingEvaluator::new()
                                .evaluate_as_bool(initial, &const_value_source);
                            valuation_builder.add_bool(value);
                        }
                    },
                    VariableRange::Float { .. } => match &variable.initial_value {
                        None => {
                            panic!(
                                "Floats must have init expressions (I'm not sure whether this is PRISM-spec-compliant)"
                            )
                        }
                        Some(initial) => {
                            let value = TreeWalkingEvaluator::new()
                                .evaluate_as_float(initial, &const_value_source);
                            valuation_builder.add_float(value);
                        }
                    },
                }
            }
        }

        let valuation = valuation_builder.finish();

        let index = self.get_or_add_state(valuation, atomic_proposition_len);

        Ok(vec![index])
    }

    #[allow(unused)]
    fn print_valuation<S: Clone>(
        valuation: &M::Valuation,
        variable_info: &ModelVariableInfo<M::Valuation>,
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    ) {
        print!("(");
        let mut first = true;
        for (i, var) in model.variable_manager.variables.iter().enumerate() {
            if let Some(index) = variable_info.valuation_map.map_to_variable(i) {
                if !first {
                    print!(", ");
                }
                first = false;
                print!("{} = ", var.name);
                match var.range {
                    VariableRange::BoundedInt { .. } => {
                        print!("{}", valuation.evaluate_bounded_int(index))
                    }
                    VariableRange::UnboundedInt { .. } => {
                        print!("{}", valuation.evaluate_bounded_int(index))
                    }
                    VariableRange::Boolean { .. } => {
                        print!("{}", valuation.evaluate_bool(index))
                    }
                    VariableRange::Float { .. } => {
                        print!("{}", valuation.evaluate_float(index))
                    }
                }
            }
        }
        print!(")");
    }
}

#[derive(Debug)]
pub enum ModelBuildingError {}

pub struct SynchronisedActions {
    actions: Vec<SynchronisedAction>,
}

pub struct SynchronisedAction {
    participating_modules: Vec<SynchronisedActionModule>,
    name: String,
}

pub struct SynchronisedActionModule {
    module_index: usize,
    command_indices: Vec<usize>,
}

impl SynchronisedActions {
    pub fn from_prism<S: Clone>(
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    ) -> Self {
        let mut actions: HashMap<String, SynchronisedAction> = HashMap::new();

        for (module_index, module) in model.modules.modules.iter().enumerate() {
            let mut module_actions: HashMap<String, SynchronisedActionModule> = HashMap::new();
            for (command_index, command) in module.commands.iter().enumerate() {
                if let Some(action) = &command.action {
                    if let Some(module_action) = module_actions.get_mut(&action.name) {
                        module_action.command_indices.push(command_index);
                    } else {
                        module_actions.insert(
                            action.name.clone(),
                            SynchronisedActionModule {
                                module_index,
                                command_indices: vec![command_index],
                            },
                        );
                    }
                }
            }
            for (action_name, module_action) in module_actions {
                if let Some(action) = actions.get_mut(&action_name) {
                    action.participating_modules.push(module_action);
                } else {
                    let synchronised_action_into = SynchronisedAction {
                        name: action_name.clone(),
                        participating_modules: vec![module_action],
                    };
                    actions.insert(action_name, synchronised_action_into);
                }
            }
        }

        SynchronisedActions {
            actions: actions.into_iter().map(|(_, a)| a).collect(),
        }
    }
}
