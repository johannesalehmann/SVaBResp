mod formula_dependencies;

pub use formula_dependencies::*;
use std::fmt::{Display, Formatter};

use crate::{Expression, Identifier};

pub struct FormulaManager<V, S: Clone> {
    pub formulas: Vec<Formula<V, S>>,
}

impl<V, S: Clone> FormulaManager<V, S> {
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

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> FormulaManager<V, S2> {
        FormulaManager {
            formulas: self.formulas.into_iter().map(|f| f.map_span(map)).collect(),
        }
    }
}

impl<V: Display, S: Clone> Display for FormulaManager<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for formula in &self.formulas {
            writeln!(f, "{}", formula)?;
        }
        if self.formulas.len() > 0 {
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AddFormulaError {
    FormulaExists { index: usize },
}

pub struct Formula<V, S: Clone> {
    pub name: Identifier<S>,
    pub condition: Expression<V, S>,
    pub span: S,
}

impl<V, S: Clone> Formula<V, S> {
    pub fn new(name: Identifier<S>, condition: Expression<V, S>, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Formula<V, S2> {
        Formula {
            name: self.name.map_span(map),
            condition: self.condition.map_span(&map),
            span: map(self.span),
        }
    }
}

impl<V: Display, S: Clone> Display for Formula<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "formula {} = {};", self.name, self.condition)
    }
}
