use crate::{Expression, Identifier};

pub struct FormulaManager<V, S> {
    formulas: Vec<Formula<V, S>>,
}

impl<V, S> FormulaManager<V, S> {
    pub fn new() -> Self {
        Self {
            formulas: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Formula<V, S>> {
        self.formulas.get(index)
    }

    pub fn add_formula(&mut self, formula: Formula<V, S>) -> Result<(), AddFormulaError> {
        for (index, other_formula) in self.formulas.iter().enumerate() {
            if other_formula.name == formula.name {
                return Err(AddFormulaError::FormulaExists { index });
            }
        }
        self.formulas.push(formula);
        Ok(())
    }
}

#[derive(Debug)]
pub enum AddFormulaError {
    FormulaExists { index: usize },
}

pub struct Formula<V, S> {
    pub name: Identifier<S>,
    pub condition: Expression<V, S>,
    pub span: S,
}

impl<V, S> Formula<V, S> {
    pub fn new(name: Identifier<S>, condition: Expression<V, S>, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }
}
