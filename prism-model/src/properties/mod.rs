use crate::expressions::UnknownVariableError;
use crate::{
    CyclicDependency, Expression, FormulaManager, Identifier, LabelManager, VariableManager,
    VariableReference,
};
use std::fmt::Formatter;

#[derive(Clone)]
pub struct Property<V, S: Clone> {
    pub operator: Operator,
    pub path: Path<V, S>,
}

#[derive(Clone)]
pub enum Operator {
    ValueOfPMax,
    ValueOfPMin,
    ValueOfP,
}

#[derive(Clone)]
pub enum Path<V, S: Clone> {
    Eventually(Expression<V, S>),
}

impl<S: Clone> Property<Identifier<S>, S> {
    pub fn substitute_labels(&mut self, default_span: S, labels: &LabelManager<Identifier<S>, S>) {
        self.path.substitute_labels(default_span, labels);
    }
    pub fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<Identifier<S>, S>,
    ) -> Result<(), CyclicDependency<S>> {
        self.path.substitute_formulas(default_span, formulas)
    }

    pub fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<R, S>,
    ) -> Result<Property<VariableReference, S>, Vec<UnknownVariableError<S>>> {
        let path = self
            .path
            .replace_identifiers_by_variable_indices(variable_manager)?;

        Ok(Property {
            operator: self.operator,
            path,
        })
    }
}
impl<S: Clone> Path<Identifier<S>, S> {
    pub fn substitute_labels(&mut self, default_span: S, labels: &LabelManager<Identifier<S>, S>) {
        match self {
            Path::Eventually(e) => e.substitute_labels(default_span, labels),
        }
    }
    pub fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<Identifier<S>, S>,
    ) -> Result<(), CyclicDependency<S>> {
        match self {
            Path::Eventually(e) => e.substitute_formulas(default_span, formulas),
        }
    }

    pub fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<R, S>,
    ) -> Result<Path<VariableReference, S>, Vec<UnknownVariableError<S>>> {
        Ok(match self {
            Path::Eventually(e) => {
                Path::Eventually(e.replace_identifiers_by_variable_indices(variable_manager)?)
            }
        })
    }
}

impl<V: std::fmt::Debug, S: Clone> std::fmt::Debug for Property<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} [{:?}]", self.operator, self.path)
    }
}
impl std::fmt::Debug for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::ValueOfPMax => {
                write!(f, "Pmax=?")
            }
            Operator::ValueOfPMin => {
                write!(f, "Pmin=?")
            }
            Operator::ValueOfP => {
                write!(f, "P=?")
            }
        }
    }
}
impl<V: std::fmt::Debug, S: Clone> std::fmt::Debug for Path<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::Eventually(e) => {
                write!(f, "F {:?}", e)
            }
        }
    }
}
