use crate::expressions::UnknownVariableError;
use crate::module::RenameRules;
use crate::{Expression, Identifier};
use std::fmt::{Display, Formatter};

pub struct VariableManager<V, S: Clone> {
    pub variables: Vec<VariableInfo<V, S>>,
}

impl<V, S: Clone> VariableManager<V, S> {
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
    pub fn get_reference_by_str(&self, name: &str) -> Option<VariableReference> {
        for (index, var) in self.variables.iter().enumerate() {
            if &var.name.name == name {
                return Some(VariableReference::new(index));
            }
        }
        None
    }

    pub fn get(&self, reference: &VariableReference) -> Option<&VariableInfo<V, S>> {
        self.variables.get(reference.index)
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> VariableManager<V, S2> {
        VariableManager {
            variables: self
                .variables
                .into_iter()
                .map(|v| v.map_span(map))
                .collect(),
        }
    }

    pub fn format_as_consts(&self) -> PrintableVariableManager<'_, V, S> {
        PrintableVariableManager {
            vm: &self,
            display_kind: VariablePrintingStyle::Const,
            filter: VariableFilter::Constant,
        }
    }
    pub fn format_as_global_vars(&self) -> PrintableVariableManager<'_, V, S> {
        PrintableVariableManager {
            vm: &self,
            display_kind: VariablePrintingStyle::GlobalVar,
            filter: VariableFilter::GlobalVar,
        }
    }
    pub fn format_as_local_vars(&self, module: usize) -> PrintableVariableManager<'_, V, S> {
        PrintableVariableManager {
            vm: &self,
            display_kind: VariablePrintingStyle::LocalVar,
            filter: VariableFilter::LocalVar(module),
        }
    }
}
impl<S: Clone> VariableManager<Identifier<S>, S> {
    pub fn add_renamed(
        &mut self,
        old_module_index: usize,
        new_module_index: usize,
        rename_rules: &RenameRules<S>,
    ) -> Result<VariableManager<Identifier<S>, S>, MissingVariableRenaming<S>> {
        let variables = Vec::with_capacity(self.variables.len());
        for i in 0..self.variables.len() {
            let variable = &self.variables[i];
            if variable.is_constant || variable.scope != Some(old_module_index) {
                continue;
            }
            match rename_rules.get_renaming(&variable.name) {
                None => {
                    return Err(MissingVariableRenaming {
                        variable_name: variable.name.clone(),
                        original_definition: variable.span.clone(),
                    });
                }
                Some(renaming) => {
                    let new_var = VariableInfo {
                        range: variable.range.renamed(rename_rules),
                        name: renaming,
                        initial_value: variable
                            .initial_value
                            .as_ref()
                            .map(|i| i.renamed(rename_rules)),
                        span: variable.span.clone(),
                        is_constant: false,
                        scope: Some(new_module_index),
                    };
                    self.variables.push(new_var)
                }
            }
        }

        Ok(VariableManager { variables })
    }
}

pub enum VariableAddError {
    VariableExists { reference: VariableReference },
}

pub struct MissingVariableRenaming<S: Clone> {
    pub variable_name: Identifier<S>,
    pub original_definition: S,
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

pub struct VariableInfo<V, S: Clone> {
    pub is_constant: bool,
    pub scope: Option<usize>,
    pub range: VariableRange<V, S>,
    pub name: Identifier<S>,
    pub initial_value: Option<Expression<V, S>>,
    pub span: S,
}

impl<V, S: Clone> VariableInfo<V, S> {
    pub fn new(
        name: Identifier<S>,
        range: VariableRange<V, S>,
        span: S,
        is_constant: bool,
        scope: Option<usize>,
    ) -> Self {
        Self {
            name,
            range,
            initial_value: None,
            span,
            is_constant,
            scope,
        }
    }

    pub fn with_initial_value(
        name: Identifier<S>,
        range: VariableRange<V, S>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: Expression<V, S>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value: Some(initial_value),
            span,
            is_constant,
            scope,
        }
    }

    pub fn with_optional_initial_value(
        name: Identifier<S>,
        range: VariableRange<V, S>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: Option<Expression<V, S>>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value,
            span,
            is_constant,
            scope,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> VariableInfo<V, S2> {
        VariableInfo {
            is_constant: self.is_constant,
            scope: self.scope,
            range: self.range.map_span(map),
            name: self.name.map_span(map),
            initial_value: self.initial_value.map(|i| i.map_span(map)),
            span: map(self.span),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VariableRange<V, S: Clone> {
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
    Float {
        span: S,
    },
}

impl<V, S: Clone> VariableRange<V, S> {
    pub fn is_legal_for_constant(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => false,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Float { .. } => true,
        }
    }
    pub fn is_legal_for_variable(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => true,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Float { .. } => false,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            VariableRange::BoundedInt { .. } => "bounded int",
            VariableRange::UnboundedInt { .. } => "int",
            VariableRange::Boolean { .. } => "bool",
            VariableRange::Float { .. } => "double",
        }
    }

    pub fn span(&self) -> &S {
        match self {
            VariableRange::BoundedInt { span, .. } => span,
            VariableRange::UnboundedInt { span } => span,
            VariableRange::Boolean { span } => span,
            VariableRange::Float { span } => span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> VariableRange<V, S2> {
        match self {
            VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                min: min.map_span(map),
                max: max.map_span(map),
                span: map(span),
            },
            VariableRange::UnboundedInt { span } => VariableRange::UnboundedInt { span: map(span) },
            VariableRange::Boolean { span } => VariableRange::Boolean { span: map(span) },
            VariableRange::Float { span } => VariableRange::Float { span: map(span) },
        }
    }
}
impl<S: Clone> VariableRange<Identifier<S>, S> {
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        match self {
            VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                min: min.renamed(rename_rules),
                max: max.renamed(rename_rules),
                span: span.clone(),
            },
            VariableRange::UnboundedInt { span } => {
                VariableRange::UnboundedInt { span: span.clone() }
            }
            VariableRange::Boolean { span } => VariableRange::Boolean { span: span.clone() },
            VariableRange::Float { span } => VariableRange::Float { span: span.clone() },
        }
    }

    pub fn replace_identifiers_by_variable_indices(
        &self,
        variable_manager: &VariableManager<Identifier<S>, S>,
    ) -> Result<VariableRange<VariableReference, S>, Vec<UnknownVariableError<S>>> {
        match self {
            VariableRange::BoundedInt { min, max, span } => {
                let mut errors = Vec::new();
                let min = min
                    .clone()
                    .replace_identifiers_by_variable_indices(variable_manager);
                let max = max
                    .clone()
                    .replace_identifiers_by_variable_indices(variable_manager);
                if let Err(err) = &min {
                    errors.extend_from_slice(&err[..]);
                }
                if let Err(err) = &max {
                    errors.extend_from_slice(&err[..]);
                }
                if let (Ok(min), Ok(max)) = (min, max) {
                    Ok(VariableRange::BoundedInt {
                        min,
                        max,
                        span: span.clone(),
                    })
                } else {
                    Err(errors)
                }
            }
            VariableRange::UnboundedInt { span } => {
                Ok(VariableRange::UnboundedInt { span: span.clone() })
            }
            VariableRange::Boolean { span } => Ok(VariableRange::Boolean { span: span.clone() }),
            VariableRange::Float { span } => Ok(VariableRange::Float { span: span.clone() }),
        }
    }
}

impl<V: Display, S: Clone> Display for VariableRange<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableRange::BoundedInt { min, max, .. } => {
                write!(f, "[{}..{}]", min, max)
            }
            VariableRange::UnboundedInt { .. } => {
                write!(f, "int")
            }
            VariableRange::Boolean { .. } => {
                write!(f, "bool")
            }
            VariableRange::Float { .. } => {
                write!(f, "double")
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct VariableReference {
    pub index: usize,
}

impl VariableReference {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl Display for VariableReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.index)
    }
}

impl std::fmt::Debug for VariableReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.index)
    }
}

#[derive(PartialEq)]
enum VariablePrintingStyle {
    Const,
    GlobalVar,
    LocalVar,
}

enum VariableFilter {
    Constant,
    GlobalVar,
    LocalVar(usize),
}

impl VariableFilter {
    fn accepts<V, S: Clone>(&self, variable: &VariableInfo<V, S>) -> bool {
        match self {
            VariableFilter::Constant => variable.is_constant,
            VariableFilter::GlobalVar => !variable.is_constant && variable.scope.is_none(),
            VariableFilter::LocalVar(index) => {
                !variable.is_constant && variable.scope == Some(*index)
            }
        }
    }
}

pub struct PrintableVariableManager<'a, V, S: Clone> {
    vm: &'a VariableManager<V, S>,
    display_kind: VariablePrintingStyle,
    filter: VariableFilter,
}

impl<'a, V: Display, S: Clone> Display for PrintableVariableManager<'a, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for variable in &self.vm.variables {
            if !self.filter.accepts(variable) {
                continue;
            }
            if self.display_kind == VariablePrintingStyle::Const {
                write!(f, "const {} {}", variable.range, variable.name)?;
                if let Some(initial) = &variable.initial_value {
                    write!(f, " = {}", initial)?;
                }
            } else {
                if self.display_kind == VariablePrintingStyle::GlobalVar {
                    write!(f, "global ")?;
                } else {
                    write!(f, "    ")?;
                }
                write!(f, "{} : {}", variable.name, variable.range)?;
                if let Some(initial) = &variable.initial_value {
                    write!(f, " init {}", initial)?;
                }
            }
            writeln!(f, ";")?;
        }
        if self.vm.variables.len() > 0 {
            writeln!(f, "")?;
        }
        Ok(())
    }
}
