use crate::{CyclicDependency, Expression, Identifier, IdentityMapExpression, VariableManager};

impl<AM, A, S: Clone> super::Model<AM, A, crate::Identifier<S>, S> {
    pub fn substitute_formulas(&mut self, default_span: S) -> Result<(), CyclicDependency<S>> {
        let order = self.formulas.get_formula_replacement_ordering()?;

        for formula_index in order {
            let formula = self.formulas.get(formula_index).unwrap();
            let mut visitor = FormulaSubstitutionVisitor {
                formula_name: &formula.name,
                expression: &formula.condition,
            };

            let mut replace_expression = |e: &mut Expression<Identifier<S>, S>| {
                let condition = std::mem::replace(e, Expression::Bool(false, default_span.clone()));
                *e = condition.visit(&mut visitor);
            };

            for label in &mut self.labels.labels {
                replace_expression(&mut label.condition);
            }
            for reward in &mut self.rewards.rewards {
                for entry in &mut reward.entries {
                    replace_expression(&mut entry.condition);
                    replace_expression(&mut entry.value);
                }
            }
            if let Some(init_constraint) = &mut self.init_constraint {
                replace_expression(init_constraint);
            }
            for module in &mut self.modules.modules {
                for command in &mut module.commands {
                    replace_expression(&mut command.guard);
                    for update in &mut command.updates {
                        replace_expression(&mut update.probability);
                        for assignment in &mut update.assignments {
                            replace_expression(&mut assignment.value);
                        }
                    }
                }
            }
            let mut replace_in_var_defs = |vm: &mut VariableManager<Identifier<S>, S>| {
                for variable in &mut vm.variables {
                    if let Some(initial_value) = &mut variable.initial_value {
                        replace_expression(initial_value);
                    }
                    if let crate::VariableRange::BoundedInt { min, max, .. } = &mut variable.range {
                        replace_expression(min);
                        replace_expression(max);
                    }
                }
            };
            replace_in_var_defs(&mut self.global_constants);
            replace_in_var_defs(&mut self.global_variables);
            for module in &mut self.modules.modules {
                replace_in_var_defs(&mut module.variables);
            }
        }

        self.formulas.formulas.clear();

        Ok(())
    }
}

pub struct FormulaSubstitutionVisitor<'a, S: Clone> {
    formula_name: &'a Identifier<S>,
    expression: &'a crate::Expression<crate::Identifier<S>, S>,
}

impl<'a, S: Clone> crate::expressions::identity_map::Private for FormulaSubstitutionVisitor<'a, S> {}
impl<'a, S: Clone> IdentityMapExpression<crate::Identifier<S>, S>
    for FormulaSubstitutionVisitor<'a, S>
{
    fn visit_var_or_const(&mut self, name: Identifier<S>, span: S) -> Expression<Identifier<S>, S> {
        if &name == self.formula_name {
            self.expression.clone()
        } else {
            Expression::VarOrConst(name, span)
        }
    }
}
