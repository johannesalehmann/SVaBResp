mod expressions;
mod model_in_progress;
mod synchronised_actions;
mod variables;

use crate::expressions::{Evaluator, TreeWalkingEvaluator, ValuationSource, VariableType};
use crate::model_in_progress::ModelInProgress;
use crate::synchronised_actions::{SynchronisedAction, SynchronisedActions};
use crate::variables::{ConstAndVarValuationSource, ModelVariableInfo};
use log::info;
use prism_model::{
    Command, Expression, Identifier, Model, Update, VariableManager, VariableRange,
    VariableReference,
};
use probabilistic_models::probabilistic_properties::{ProbabilityOperator, Property};
use probabilistic_models::{
    Action, AtomicProposition, AtomicPropositions, Builder, Distribution, MdpType, ModelTypes,
    PredecessorsBuilder, ProbabilisticModel, Successor, Valuation, ValuationBuilder,
};
use probabilistic_models::{DistributionBuilder, Predecessor};
use std::collections::HashMap;

pub fn build_model<S: Clone, M: ModelTypes>(
    model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    atomic_propositions: &[Expression<VariableReference, S>],
    user_provided_consts: &HashMap<String, UserProvidedConstValue>,
) -> Result<ProbabilisticModel<M>, ModelBuildingError> {
    ExplicitModelBuilder::<M>::run(model, atomic_propositions, user_provided_consts)
}
pub fn build_properties<
    S: Clone,
    I: Iterator<Item = Property<AtomicProposition, Expression<VariableReference, S>>>,
>(
    model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    properties: I,
    user_provided_consts: &HashMap<String, UserProvidedConstValue>,
) -> Result<Vec<Property<AtomicProposition, f64>>, ModelBuildingError> {
    ExplicitModelBuilder::<MdpType>::build_property(model, properties, user_provided_consts)
}

pub enum UserProvidedConstValue {
    Int(i64),
    Bool(bool),
    Float(f64),
}

pub struct ExplicitModelBuilder<M: ModelTypes> {
    model_in_progress: ModelInProgress<M>,
    open_states: Vec<usize>,
    variable_info: variables::ModelVariableInfo<M::Valuation>,
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
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
    ) -> Result<ProbabilisticModel<M>, ModelBuildingError> {
        let start_time = std::time::Instant::now();
        let variable_info = variables::ModelVariableInfo::new(model, user_provided_consts)?;

        let synchronised_actions = SynchronisedActions::from_prism(model);

        let mut builder = Self {
            model_in_progress: ModelInProgress::new(atomic_propositions.len()),
            open_states: Vec::new(),
            variable_info,
        };

        builder.create_initial_states(model)?;

        while let Some(state) = builder.open_states.pop() {
            builder.process_state(state, &model, atomic_propositions, &synchronised_actions)?;
        }

        let result = builder
            .model_in_progress
            .into_model(builder.variable_info.valuation_context);

        info!(
            "Model built in {:?} ({} states)",
            start_time.elapsed(),
            result.states.len()
        );
        Ok(result)
    }

    fn get_or_add_state(&mut self, valuation: M::Valuation) -> usize {
        let index = self.model_in_progress.get_state_index(&valuation);
        match index {
            Some(index) => index,
            None => {
                let index = self.model_in_progress.add_state(valuation);
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
                self.process_nonsynchronised_command(state, &model, &mut action_index, &command);
            }
        }

        for synchronised_action in synchronised_actions {
            self.process_synchronising_action(
                state,
                &model,
                &mut action_index,
                &synchronised_action,
            );
        }

        Ok(())
    }

    fn process_nonsynchronised_command<S: Clone>(
        &mut self,
        state: usize,
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        action_index: &mut usize,
        command: &Command<Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    ) {
        let valuation = &self.model_in_progress.get_state(state).valuation;
        let val_source = self.variable_info.get_valuation_source(valuation);
        let guard = TreeWalkingEvaluator::new().evaluate_as_bool(&command.guard, &val_source);
        if guard {
            let mut distribution = <M::Distribution as Distribution>::get_builder();

            for update_index in 0..command.updates.len() {
                let valuation = &self.model_in_progress.get_state(state).valuation;
                let val_source = self.variable_info.get_valuation_source(valuation);

                let update = &command.updates[update_index];
                let probability =
                    TreeWalkingEvaluator::new().evaluate_as_float(&update.probability, &val_source);
                let new_valuation = self.apply_assignments(
                    &model.variable_manager,
                    valuation,
                    &val_source,
                    &[&update],
                );

                let index = self.get_or_add_state(new_valuation);
                distribution.add_successor(Successor { probability, index });

                self.model_in_progress
                    .get_state_mut(index)
                    .predecessors
                    .add(Predecessor {
                        from: state,
                        action_index: *action_index,
                        probability,
                    });
            }

            let action_name_index = self.model_in_progress.get_unnamed_action_name_index();
            let successors = distribution.finish();
            if successors.number_of_successors() == 0 {
                println!(
                    "State {} a local action with zero successors",
                    self.model_in_progress
                        .get_state(state)
                        .valuation
                        .displayable(&self.variable_info.valuation_context)
                )
            }
            self.model_in_progress
                .get_state_mut(state)
                .actions
                .add_action(Action {
                    successors,
                    action_name_index,
                });
            *action_index += 1;
        }
    }

    fn process_synchronising_action<S: Clone>(
        &mut self,
        state: usize,
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        action_index: &mut usize,
        synchronised_action: &&SynchronisedAction,
    ) {
        let action_name_index = self
            .model_in_progress
            .get_action_name_index(&synchronised_action.name);

        let valuation = &self.model_in_progress.get_state(state).valuation;
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
                    < model.modules.modules[modules[0].module_index].commands[command_indices[0]]
                        .updates
                        .len()
                        .max(1)
                // max(1) is required because a synchronising action may have an empty update ("true")
                {
                    let valuation = &self.model_in_progress.get_state(state).valuation;
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
                            let ith_expression = &command.updates[update_indices[i]].probability;
                            let ith_probability = TreeWalkingEvaluator::new()
                                .evaluate_as_float(ith_expression, &val_source);
                            probability *= ith_probability;
                        }
                    }

                    let index = self.get_or_add_state(new_valuation);
                    distribution.add_successor(Successor { probability, index });
                    self.model_in_progress
                        .get_state_mut(index)
                        .predecessors
                        .add(Predecessor {
                            from: state,
                            action_index: *action_index,
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

                self.model_in_progress
                    .get_state_mut(state)
                    .actions
                    .add_action(Action {
                        successors: distribution.finish(),
                        action_name_index,
                    });
                *action_index += 1;

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

    fn evaluate_atomic_propositions<S: Clone>(
        &mut self,
        state_index: usize,
        atomic_propositions: &[Expression<VariableReference, S>],
    ) {
        let state = &mut self.model_in_progress.get_state_mut(state_index);
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
    ) -> Result<(), ModelBuildingError> {
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

        let index = self.get_or_add_state(valuation);

        self.model_in_progress.add_initial_state(index);

        Ok(())
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
