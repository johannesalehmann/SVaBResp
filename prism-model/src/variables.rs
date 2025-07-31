use crate::{Expression, Identifier};
use std::fmt::Formatter;

pub struct VariableManager<V, S> {
    variables: Vec<VariableInfo<V, S>>,
}

impl<V, S> VariableManager<V, S> {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    pub fn add_variable(
        &mut self,
        variable_info: VariableInfo<V, S>,
    ) -> Result<VariableReference, VariableAddError> {
        if let Some(existing_variable) = self.get_reference(&variable_info.name) {
            Err(VariableAddError::VariableExists {
                reference: existing_variable,
            })
        } else {
            let index = VariableReference::new(self.variables.len());
            self.variables.push(variable_info);
            Ok(index)
        }
    }

    pub fn get_reference(&self, name: &Identifier<S>) -> Option<VariableReference> {
        for (index, var) in self.variables.iter().enumerate() {
            if &var.name == name {
                return Some(VariableReference::new(index));
            }
        }
        None
    }

    pub fn get(&self, reference: &VariableReference) -> Option<&VariableInfo<V, S>> {
        self.variables.get(reference.index)
    }
}

pub enum VariableAddError {
    VariableExists { reference: VariableReference },
}

impl std::fmt::Debug for VariableAddError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableAddError::VariableExists { reference } => {
                write!(
                    f,
                    "Variable already exists in this manager (index {})",
                    reference.index
                )
            }
        }
    }
}

pub struct VariableInfo<V, S> {
    pub range: VariableRange<V, S>,
    pub name: Identifier<S>,
    pub initial_value: Option<Expression<V, S>>,
    pub span: S,
}

impl<V, S> VariableInfo<V, S> {
    pub fn new(name: Identifier<S>, range: VariableRange<V, S>, span: S) -> Self {
        Self {
            name,
            range,
            initial_value: None,
            span,
        }
    }

    pub fn with_initial_value(
        name: Identifier<S>,
        range: VariableRange<V, S>,
        initial_value: Expression<V, S>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value: Some(initial_value),
            span,
        }
    }

    pub fn with_optional_initial_value(
        name: Identifier<S>,
        range: VariableRange<V, S>,
        initial_value: Option<Expression<V, S>>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value,
            span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VariableRange<V, S> {
    BoundedInt {
        min: Expression<V, S>,
        max: Expression<V, S>,
        span: S,
    },
    UnboundedInt {
        span: S,
    },
    Boolean {
        span: S,
    },
    Double {
        span: S,
    },
}

impl<V, S> VariableRange<V, S> {
    pub fn is_legal_for_constant(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => false,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Double { .. } => true,
        }
    }
    pub fn is_legal_for_variable(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => true,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Double { .. } => false,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            VariableRange::BoundedInt { .. } => "bounded int",
            VariableRange::UnboundedInt { .. } => "int",
            VariableRange::Boolean { .. } => "bool",
            VariableRange::Double { .. } => "double",
        }
    }

    pub fn span(&self) -> &S {
        match self {
            VariableRange::BoundedInt { span, .. } => span,
            VariableRange::UnboundedInt { span } => span,
            VariableRange::Boolean { span } => span,
            VariableRange::Double { span } => span,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct VariableReference {
    index: usize,
}

impl VariableReference {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}
