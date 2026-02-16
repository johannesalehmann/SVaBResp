use crate::{
    Assignment, Command, Formula, FormulaManager, Label, LabelManager, Module, ModuleManager,
    Rewards, RewardsElement, RewardsManager, Update, VariableInfo, VariableManager, VariableRange,
};

impl<AM: Default, A, E, V, S: Clone> super::Model<AM, A, E, V, S> {
    pub fn map_expressions_into<E2, F: Fn(E) -> E2>(self, f: F) -> super::Model<AM, A, E2, V, S> {
        let mut variables = Vec::new();
        for variable in self.variable_manager.variables {
            let range = match variable.range {
                VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                    min: f(min),
                    max: f(max),
                    span,
                },

                VariableRange::UnboundedInt { span } => VariableRange::UnboundedInt { span },
                VariableRange::Boolean { span } => VariableRange::Boolean { span },
                VariableRange::Float { span } => VariableRange::Float { span },
            };
            let initial_value = variable.initial_value.map(|i| f(i));
            variables.push(VariableInfo::with_optional_initial_value(
                variable.name,
                range,
                variable.is_constant,
                variable.scope,
                initial_value,
                variable.span,
            ));
        }
        let variable_manager = VariableManager { variables };

        let formulas = FormulaManager {
            formulas: self
                .formulas
                .formulas
                .into_iter()
                .map(|formula| Formula {
                    name: formula.name,
                    condition: f(formula.condition),
                    span: formula.span,
                })
                .collect::<Vec<_>>(),
        };

        let mut modules = Vec::new();

        for module in self.modules.modules {
            let mut commands = Vec::new();
            for command in module.commands {
                let mut updates = Vec::new();
                for update in command.updates {
                    updates.push(Update {
                        probability: f(update.probability),
                        assignments: update
                            .assignments
                            .into_iter()
                            .map(|assg| Assignment {
                                target: assg.target,
                                value: f(assg.value),
                                target_span: assg.target_span,
                                span: assg.span,
                            })
                            .collect(),
                        span: update.span,
                    });
                }
                commands.push(Command {
                    action: command.action,
                    guard: f(command.guard),
                    updates,
                    span: command.span,
                })
            }
            modules.push(Module {
                name: module.name,
                commands,
                span: module.span,
            })
        }
        let modules = ModuleManager { modules };

        let init_constraint = self.init_constraint.map(|i| f(i));

        let labels = LabelManager {
            labels: self
                .labels
                .labels
                .into_iter()
                .map(|label| Label {
                    name: label.name,
                    condition: f(label.condition),
                    span: label.span,
                })
                .collect(),
        };

        let mut rewards = Vec::new();
        for reward in self.rewards.rewards {
            let entries = reward
                .entries
                .into_iter()
                .map(|reward_element| RewardsElement {
                    condition: f(reward_element.condition),
                    value: f(reward_element.value),
                    target: reward_element.target,
                    span: reward_element.span,
                })
                .collect();
            rewards.push(Rewards {
                name: reward.name,
                entries,
                span: reward.span,
            });
        }
        let rewards = RewardsManager { rewards };

        super::Model {
            model_type: self.model_type,
            variable_manager,
            formulas,
            action_manager: self.action_manager,
            modules,
            renamed_modules: self.renamed_modules,
            init_constraint,
            labels,
            rewards,
            span: self.span,
        }
    }
}
