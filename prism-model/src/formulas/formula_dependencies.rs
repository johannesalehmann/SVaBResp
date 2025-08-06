use crate::expressions::DefaultMapExpression;
use crate::{Expression, Formula, FormulaManager, Identifier};
use std::collections::HashMap;

pub struct SpannedDependency<S> {
    dependency: usize,
    span: S,
}

impl<S> SpannedDependency<S> {
    pub fn new(dependency: usize, span: S) -> Self {
        Self { dependency, span }
    }
}

pub struct FormulaCountingVisitor<'a, S: Clone> {
    formulas: &'a Vec<Formula<Identifier<S>, S>>,
    found_formulas: Vec<SpannedDependency<S>>,
}

impl<'a, S: Clone> DefaultMapExpression<Identifier<S>, S, ()> for FormulaCountingVisitor<'a, S> {
    fn visit_var_or_const(&mut self, name: Identifier<S>, span: S) -> () {
        for (i, formula) in self.formulas.iter().enumerate() {
            if formula.name == name {
                if !self.found_formulas.iter().any(|f| f.dependency == i) {
                    self.found_formulas.push(SpannedDependency::new(i, span));
                }
                break;
            }
        }
    }
}

impl<S: Clone> FormulaManager<Identifier<S>, S> {
    pub fn get_spanned_formulas_in_expression(
        &self,
        expression: Expression<Identifier<S>, S>,
    ) -> Vec<SpannedDependency<S>> {
        let mut formulas = HashMap::new();
        for formula in &self.formulas {
            formulas.insert(formula.name.name.clone(), false);
        }
        let mut visitor = FormulaCountingVisitor {
            formulas: &self.formulas,
            found_formulas: Vec::new(),
        };

        expression.visit(&mut visitor);

        visitor.found_formulas
    }
}

impl<S: Clone> FormulaManager<Identifier<S>, S> {
    // The S: Clone dependency can be removed by adding a visitor pattern for &Expression and passing &f.condition instead of f.condition.clone() to get_formulas_in_expression
    pub fn get_formula_replacement_ordering(&self) -> Result<Vec<usize>, CyclicDependency<S>> {
        let mut dependencies = self
            .formulas
            .iter()
            .map(|f| {
                DependencyData::new(self.get_spanned_formulas_in_expression(f.condition.clone()))
            })
            .collect::<Vec<_>>();

        let mut output = Vec::new();

        let mut stack: Vec<TraversalData<S>> = Vec::new();
        for i in 0..dependencies.len() {
            if dependencies[i].was_visited {
                continue;
            }

            stack.push(TraversalData {
                dep_index: i,
                next_child: 0,
                span_in_parent_expression: None,
            });

            while let Some(data) = stack.last_mut() {
                let deps = &dependencies[data.dep_index];
                if deps.was_visited && deps.was_emitted {
                    stack.pop();
                } else if data.next_child == 0 && deps.was_visited && !deps.was_emitted {
                    let mut cycle = Vec::new();
                    let mut previous_span =
                        data.span_in_parent_expression.as_ref().unwrap().clone();
                    let first_index = data.dep_index;

                    for element in stack.iter().rev().skip(1) {
                        cycle.push(CyclicDependencyEntry {
                            formula_name: self.formulas[element.dep_index].name.clone(),
                            formula_span: self.formulas[element.dep_index].span.clone(),
                            dependency_span: previous_span,
                        });
                        if element.dep_index == first_index {
                            break;
                        } else {
                            previous_span =
                                element.span_in_parent_expression.as_ref().unwrap().clone();
                        }
                    }

                    return Err(CyclicDependency { entries: cycle });
                } else {
                    dependencies[data.dep_index].was_visited = true;
                    if data.next_child < dependencies[data.dep_index].dependencies.len() {
                        let child_index =
                            dependencies[data.dep_index].dependencies[data.next_child].dependency;
                        let span_in_parent_expression = Some(
                            dependencies[data.dep_index].dependencies[data.next_child]
                                .span
                                .clone(),
                        );
                        let child = TraversalData {
                            dep_index: child_index,
                            next_child: 0,
                            span_in_parent_expression,
                        };
                        data.next_child += 1;
                        stack.push(child);
                    } else {
                        let index = stack.pop().unwrap().dep_index;
                        output.push(index);
                        dependencies[index].was_emitted = true;
                    }
                }
            }
        }

        output.reverse();
        Ok(output)
    }
}

#[derive(Debug, PartialEq)]
pub struct CyclicDependency<S: Clone> {
    pub entries: Vec<CyclicDependencyEntry<S>>,
}

#[derive(Debug, PartialEq)]
pub struct CyclicDependencyEntry<S: Clone> {
    pub formula_name: Identifier<S>,
    pub formula_span: S,
    pub dependency_span: S,
}

struct DependencyData<S: Clone> {
    dependencies: Vec<SpannedDependency<S>>,
    was_emitted: bool,
    was_visited: bool,
}

impl<S: Clone> DependencyData<S> {
    pub fn new(dependencies: Vec<SpannedDependency<S>>) -> Self {
        Self {
            dependencies,
            was_emitted: false,
            was_visited: false,
        }
    }
}

struct TraversalData<S: Clone> {
    dep_index: usize,
    next_child: usize,
    span_in_parent_expression: Option<S>,
}
