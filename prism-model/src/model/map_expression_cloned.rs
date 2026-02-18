use crate::{
    Assignment, Command, Formula, FormulaManager, Label, LabelManager, Module, ModuleManager,
    Rewards, RewardsElement, RewardsManager, Update, VariableInfo, VariableManager, VariableRange,
};

impl<AM: Default + Clone, A: Clone, E, V: Clone, S: Clone> super::Model<AM, A, E, V, S> {
    pub fn map_expressions_cloned<E2, F: FnMut(&E) -> E2>(
        &self,
        mut f: F,
    ) -> super::Model<AM, A, E2, V, S> {
        let mut variables = Vec::new();
        for variable in &self.variable_manager.variables {
            let range = match &variable.range {
                VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                    min: f(min),
                    max: f(max),
                    span: span.clone(),
                },

                VariableRange::UnboundedInt { span } => {
                    VariableRange::UnboundedInt { span: span.clone() }
                }
                VariableRange::Boolean { span } => VariableRange::Boolean { span: span.clone() },
                VariableRange::Float { span } => VariableRange::Float { span: span.clone() },
            };
            let initial_value = variable.initial_value.as_ref().map(|i| f(i));
            variables.push(VariableInfo::with_optional_initial_value(
                variable.name.clone(),
                range,
                variable.is_constant,
                variable.scope,
                initial_value,
                variable.span.clone(),
            ));
        }
        let variable_manager = VariableManager { variables };

        let formulas = FormulaManager {
            formulas: self
                .formulas
                .formulas
                .iter()
                .map(|formula| Formula {
                    name: formula.name.clone(),
                    condition: f(&formula.condition),
                    span: formula.span.clone(),
                })
                .collect::<Vec<_>>(),
        };

        let mut modules = Vec::new();

        for module in &self.modules.modules {
            let mut commands = Vec::new();
            for command in &module.commands {
                let mut updates = Vec::new();
                for update in &command.updates {
                    updates.push(Update {
                        probability: f(&update.probability),
                        assignments: update
                            .assignments
                            .iter()
                            .map(|assg| Assignment {
                                target: assg.target.clone(),
                                value: f(&assg.value),
                                target_span: assg.target_span.clone(),
                                span: assg.span.clone(),
                            })
                            .collect(),
                        span: update.span.clone(),
                    });
                }
                commands.push(Command {
                    action: command.action.clone(),
                    guard: f(&command.guard),
                    updates,
                    span: command.span.clone(),
                })
            }
            modules.push(Module {
                name: module.name.clone(),
                commands,
                span: module.span.clone(),
            })
        }
        let modules = ModuleManager { modules };

        let init_constraint = self.init_constraint.as_ref().map(|i| f(i));

        let labels = LabelManager {
            labels: self
                .labels
                .labels
                .iter()
                .map(|label| Label {
                    name: label.name.clone(),
                    condition: f(&label.condition),
                    span: label.span.clone(),
                })
                .collect(),
        };

        let mut rewards = Vec::new();
        for reward in &self.rewards.rewards {
            let entries = reward
                .entries
                .iter()
                .map(|reward_element| RewardsElement {
                    condition: f(&reward_element.condition),
                    value: f(&reward_element.value),
                    target: reward_element.target.clone(),
                    span: reward_element.span.clone(),
                })
                .collect();
            rewards.push(Rewards {
                name: reward.name.clone(),
                entries,
                span: reward.span.clone(),
            });
        }
        let rewards = RewardsManager { rewards };

        super::Model {
            model_type: self.model_type.clone(),
            variable_manager,
            formulas,
            action_manager: self.action_manager.clone(),
            modules,
            renamed_modules: self.renamed_modules.iter().map(|r| r.clone()).collect(),
            init_constraint,
            labels,
            rewards,
            span: self.span.clone(),
        }
    }
}
