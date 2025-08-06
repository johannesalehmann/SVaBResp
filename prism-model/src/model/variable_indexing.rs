use crate::expressions::UnknownVariableError;
use crate::{
    Assignment, Command, Expression, Identifier, Label, LabelManager, Model, ModuleExpansionError,
    RewardsElement, RewardsManager, VariableInfo, VariableReference,
};
use log::error;

impl<S: Clone> super::Model<(), Identifier<S>, Identifier<S>, S> {
    pub fn replace_identifiers_by_variable_indices(
        self,
    ) -> Result<super::Model<(), Identifier<S>, VariableReference, S>, Vec<UnknownVariableError<S>>>
    {
        let mut errors: Vec<UnknownVariableError<_>> = Vec::new();

        let mut variables = Vec::with_capacity(self.variable_manager.variables.len());
        for variable in &self.variable_manager.variables {
            let range = variable
                .range
                .replace_identifiers_by_variable_indices(&self.variable_manager);
            let initial_value = match &variable.initial_value {
                None => None,
                Some(initial_value) => {
                    match initial_value
                        .clone()
                        .replace_identifiers_by_variable_indices(&self.variable_manager)
                    {
                        Ok(initial) => Some(initial),
                        Err(err) => {
                            errors.extend_from_slice(&err[..]);
                            None
                        }
                    }
                }
            };
            if let Ok(range) = range {
                variables.push(VariableInfo {
                    is_constant: variable.is_constant,
                    scope: variable.scope,
                    range,
                    name: variable.name.clone(),
                    initial_value,
                    span: variable.span.clone(),
                });
            }
        }
        let variables = crate::VariableManager { variables };

        let mut formulas = Vec::with_capacity(self.formulas.formulas.len());
        for formula in self.formulas.formulas {
            let condition = formula
                .condition
                .replace_identifiers_by_variable_indices(&self.variable_manager);
            match condition {
                Ok(condition) => formulas.push(crate::Formula {
                    name: formula.name,
                    condition,
                    span: formula.span,
                }),
                Err(err) => errors.extend_from_slice(&err[..]),
            }
        }
        let formulas = crate::FormulaManager { formulas };

        let mut modules = Vec::new();
        for module in self.modules.modules {
            let mut commands = Vec::new();
            for command in module.commands {
                let guard = command
                    .guard
                    .replace_identifiers_by_variable_indices(&self.variable_manager);
                let mut updates = Vec::new();
                for update in command.updates {
                    let probability = update
                        .probability
                        .replace_identifiers_by_variable_indices(&self.variable_manager);

                    let mut assignments = Vec::new();
                    for assignment in update.assignments {
                        let target = self.variable_manager.get_reference(&assignment.target);
                        if target.is_none() {
                            errors.push(UnknownVariableError {
                                identifier: assignment.target,
                            })
                        }
                        let value = assignment
                            .value
                            .replace_identifiers_by_variable_indices(&self.variable_manager);

                        if let Err(err) = &value {
                            errors.extend_from_slice(&err[..]);
                        }
                        if let (Some(target), Ok(value)) = (target, value) {
                            assignments.push(Assignment {
                                target,
                                value,
                                target_span: assignment.target_span,
                                span: assignment.span,
                            })
                        }
                    }
                    match probability {
                        Ok(probability) => updates.push(crate::Update {
                            probability,
                            assignments,
                            span: update.span,
                        }),
                        Err(err) => errors.extend(err),
                    }
                }
                match guard {
                    Ok(guard) => commands.push(Command {
                        action: command.action,
                        guard,
                        updates,
                        span: command.span,
                    }),
                    Err(err) => errors.extend(err),
                }
            }
            modules.push(crate::Module {
                name: module.name,
                commands,
                span: module.span,
            })
        }
        let modules = crate::ModuleManager { modules };

        if self.renamed_modules.len() > 0 {
            panic!("Cannot use variable indexing in models before expanding all renamed modules");
        }
        let renamed_modules = Vec::new();

        let init_constraint = match self.init_constraint {
            None => None,
            Some(init_constraint) => {
                match init_constraint
                    .replace_identifiers_by_variable_indices(&self.variable_manager)
                {
                    Ok(init_constraint) => Some(init_constraint),
                    Err(err) => {
                        errors.extend(err);
                        None
                    }
                }
            }
        };

        let mut labels = Vec::new();
        for label in self.labels.labels {
            match label
                .condition
                .replace_identifiers_by_variable_indices(&self.variable_manager)
            {
                Ok(condition) => labels.push(Label {
                    name: label.name,
                    condition,
                    span: label.span,
                }),
                Err(err) => errors.extend(err),
            }
        }
        let labels = LabelManager { labels };

        let mut rewards = Vec::new();
        for reward in self.rewards.rewards {
            let mut entries = Vec::new();
            for entry in reward.entries {
                let condition = entry
                    .condition
                    .replace_identifiers_by_variable_indices(&self.variable_manager);
                let value = entry
                    .value
                    .replace_identifiers_by_variable_indices(&self.variable_manager);
                if let Err(err) = &condition {
                    errors.extend_from_slice(&err[..]);
                }
                if let Err(err) = &value {
                    errors.extend_from_slice(&err[..]);
                }
                if let (Ok(condition), Ok(value)) = (condition, value) {
                    entries.push(RewardsElement {
                        condition,
                        value,
                        target: entry.target,
                        span: entry.span,
                    })
                }
            }
            rewards.push(crate::Rewards {
                name: reward.name,
                entries,
                span: reward.span,
            })
        }
        let rewards = RewardsManager { rewards };

        if errors.is_empty() {
            Ok(Model::from_components(
                self.model_type.clone(),
                variables,
                formulas,
                self.action_manager,
                modules,
                renamed_modules,
                init_constraint,
                labels,
                rewards,
                self.span,
            ))
        } else {
            Err(errors)
        }
    }
}
