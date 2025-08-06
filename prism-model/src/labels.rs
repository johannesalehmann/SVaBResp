use crate::{Expression, Identifier};
use std::fmt::{Display, Formatter};

pub struct LabelManager<V, S: Clone> {
    pub labels: Vec<Label<V, S>>,
}

impl<V, S: Clone> LabelManager<V, S> {
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    pub fn add_label(&mut self, label: Label<V, S>) -> Result<(), AddLabelError> {
        for (index, other_label) in self.labels.iter().enumerate() {
            if other_label.name == label.name {
                return Err(AddLabelError::LabelExists { index });
            }
        }
        self.labels.push(label);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&Label<V, S>> {
        self.labels.get(index)
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> LabelManager<V, S2> {
        LabelManager {
            labels: self.labels.into_iter().map(|l| l.map_span(map)).collect(),
        }
    }
}

impl<V: Display, S: Clone> Display for LabelManager<V, S> {
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

pub struct Label<V, S: Clone> {
    pub name: Identifier<S>,
    pub condition: Expression<V, S>,
    pub span: S,
}

impl<V, S: Clone> Label<V, S> {
    pub fn new(name: Identifier<S>, condition: Expression<V, S>, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Label<V, S2> {
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
