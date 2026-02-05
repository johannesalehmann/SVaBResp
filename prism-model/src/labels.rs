use crate::{Expression, Identifier};
use std::fmt::{Display, Formatter};

pub struct LabelManager<E, S: Clone> {
    pub labels: Vec<Label<E, S>>,
}

impl<E, S: Clone> LabelManager<E, S> {
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    pub fn add_label(&mut self, label: Label<E, S>) -> Result<(), AddLabelError> {
        for (index, other_label) in self.labels.iter().enumerate() {
            if other_label.name == label.name {
                return Err(AddLabelError::LabelExists { index });
            }
        }
        self.labels.push(label);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&Label<E, S>> {
        self.labels.get(index)
    }

    pub fn get_index(&self, name: &str) -> Option<usize> {
        for (i, label) in self.labels.iter().enumerate() {
            if label.name.name == name {
                return Some(i);
            }
        }
        None
    }
}

impl<V, S: Clone> LabelManager<Expression<V, S>, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> LabelManager<Expression<V, S2>, S2> {
        LabelManager {
            labels: self.labels.into_iter().map(|l| l.map_span(map)).collect(),
        }
    }
}

impl<E: Display, S: Clone> Display for LabelManager<E, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for formula in &self.labels {
            writeln!(f, "{}", formula)?;
        }
        if self.labels.len() > 0 {
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AddLabelError {
    LabelExists { index: usize },
}

pub struct Label<E, S: Clone> {
    pub name: Identifier<S>,
    pub condition: E,
    pub span: S,
}

impl<E, S: Clone> Label<E, S> {
    pub fn new(name: Identifier<S>, condition: E, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }
}
impl<V, S: Clone> Label<Expression<V, S>, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Label<Expression<V, S2>, S2> {
        Label {
            name: self.name.map_span(map),
            condition: self.condition.map_span(map),
            span: map(self.span),
        }
    }
}

impl<V: Display, S: Clone> Display for Label<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "label \"{}\" = {};", self.name, self.condition)
    }
}
