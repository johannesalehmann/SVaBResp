mod formula_dependencies;

pub use formula_dependencies::*;
use std::fmt::{Display, Formatter};

use crate::{Expression, Identifier};

pub struct FormulaManager<E, S: Clone> {
    pub formulas: Vec<Formula<E, S>>,
}

impl<E, S: Clone> FormulaManager<E, S> {
    pub fn new() -> Self {
        Self {
            formulas: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Formula<E, S>> {
        self.formulas.get(index)
    }

    pub fn add_formula(&mut self, formula: Formula<E, S>) -> Result<(), AddFormulaError> {
        for (index, other_formula) in self.formulas.iter().enumerate() {
            if other_formula.name == formula.name {
                return Err(AddFormulaError::FormulaExists { index });
            }
        }
        self.formulas.push(formula);
        Ok(())
    }
}

impl<V, S: Clone> FormulaManager<Expression<V, S>, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> FormulaManager<Expression<V, S2>, S2> {
        FormulaManager {
            formulas: self.formulas.into_iter().map(|f| f.map_span(map)).collect(),
        }
    }
}

impl<E: Display, S: Clone> Display for FormulaManager<E, S> {
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

pub struct Formula<E, S: Clone> {
    pub name: Identifier<S>,
    pub condition: E,
    pub span: S,
}

impl<E, S: Clone> Formula<E, S> {
    pub fn new(name: Identifier<S>, condition: E, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }
}

impl<V, S: Clone> Formula<Expression<V, S>, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Formula<Expression<V, S2>, S2> {
        Formula {
            name: self.name.map_span(map),
            condition: self.condition.map_span(&map),
            span: map(self.span),
        }
    }
}

impl<E: Display, S: Clone> Display for Formula<E, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "formula {} = {};", self.name, self.condition)
    }
}
