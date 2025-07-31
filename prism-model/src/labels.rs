use crate::{Expression, Identifier};

pub struct LabelManager<V, S> {
    labels: Vec<Label<V, S>>,
}

impl<V, S> LabelManager<V, S> {
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
}

#[derive(Debug)]
pub enum AddLabelError {
    LabelExists { index: usize },
}

pub struct Label<V, S> {
    pub name: Identifier<S>,
    pub condition: Expression<V, S>,
    pub span: S,
}

impl<V, S> Label<V, S> {
    pub fn new(name: Identifier<S>, condition: Expression<V, S>, span: S) -> Self {
        Self {
            name,
            condition,
            span,
        }
    }
}
