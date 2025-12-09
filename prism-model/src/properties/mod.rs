use crate::expressions::UnknownVariableError;
use crate::{
    CyclicDependency, Expression, FormulaManager, Identifier, LabelManager, VariableManager,
    VariableReference,
};
use std::fmt::Formatter;

// #[derive(Clone)]
// pub struct Property<V, S: Clone> {
//     pub operator: Operator,
//     pub path: Path<V, S>,
// }
//
// #[derive(Clone)]
// pub enum Operator {
//     ValueOfPMax,
//     ValueOfPMin,
//     ValueOfP,
// }
//
// #[derive(Clone)]
// pub enum Path<V, S: Clone> {
//     Eventually(Expression<V, S>),
// }
use probabilistic_properties::{
    Path, ProbabilityConstraint, ProbabilityOperator, ProbabilitySpecifier, Property,
    StateSpecifier,
};
impl<V, S: Clone> StateSpecifier for Expression<V, S> {}
impl<V, S: Clone> ProbabilitySpecifier for Expression<V, S> {}

pub trait SubstitutableProperty<S: Clone> {
    fn substitute_labels(&mut self, default_span: S, labels: &LabelManager<Identifier<S>, S>);

    fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<Identifier<S>, S>,
    ) -> Result<(), CyclicDependency<S>>;

    fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<R, S>,
    ) -> Result<
        Property<Expression<VariableReference, S>, Expression<VariableReference, S>>,
        Vec<UnknownVariableError<S>>,
    >;
}
impl<S: Clone> SubstitutableProperty<S>
    for Property<Expression<Identifier<S>, S>, Expression<Identifier<S>, S>>
{
    fn substitute_labels(&mut self, default_span: S, labels: &LabelManager<Identifier<S>, S>) {
        self.path.substitute_labels(default_span.clone(), labels);
        self.operator
            .constraint
            .transform_probability_specifier(|p| p.substitute_labels(default_span.clone(), labels));
    }
    fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<Identifier<S>, S>,
    ) -> Result<(), CyclicDependency<S>> {
        self.path
            .substitute_formulas(default_span.clone(), formulas)?;
        self.operator
            .constraint
            .transform_probability_specifier_with_result(|p| {
                p.substitute_formulas(default_span.clone(), formulas)
            })?;
        Ok(())
    }

    fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<R, S>,
    ) -> Result<
        Property<Expression<VariableReference, S>, Expression<VariableReference, S>>,
        Vec<UnknownVariableError<S>>,
    > {
        let path = self
            .path
            .replace_identifiers_by_variable_indices(variable_manager)?;

        let constraint = self
            .operator
            .constraint
            .map_probability_specifier_with_result(|p| {
                p.replace_identifiers_by_variable_indices(variable_manager)
            })?;
        let operator = ProbabilityOperator {
            kind: self.operator.kind,
            constraint: constraint,
        };

        Ok(Property { operator, path })
    }
}

pub trait SubstitutablePath<S: Clone> {
    fn substitute_labels(&mut self, default_span: S, labels: &LabelManager<Identifier<S>, S>);
    fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<Identifier<S>, S>,
    ) -> Result<(), CyclicDependency<S>>;

    fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<R, S>,
    ) -> Result<Path<Expression<VariableReference, S>>, Vec<UnknownVariableError<S>>>;
}

impl<S: Clone> SubstitutablePath<S> for Path<Expression<Identifier<S>, S>> {
    fn substitute_labels(&mut self, default_span: S, labels: &LabelManager<Identifier<S>, S>) {
        match self {
            Path::Eventually(e) => e.substitute_labels(default_span, labels),
            Path::Never(e) => e.substitute_labels(default_span, labels),
            Path::InfinitelyOften(e) => e.substitute_labels(default_span, labels),
        }
    }
    fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<Identifier<S>, S>,
    ) -> Result<(), CyclicDependency<S>> {
        match self {
            Path::Eventually(e) => e.substitute_formulas(default_span, formulas),
            Path::Never(e) => e.substitute_formulas(default_span, formulas),
            Path::InfinitelyOften(e) => e.substitute_formulas(default_span, formulas),
        }
    }

    fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<R, S>,
    ) -> Result<Path<Expression<VariableReference, S>>, Vec<UnknownVariableError<S>>> {
        Ok(match self {
            Path::Eventually(e) => {
                Path::Eventually(e.replace_identifiers_by_variable_indices(variable_manager)?)
            }
            Path::Never(e) => {
                Path::Eventually(e.replace_identifiers_by_variable_indices(variable_manager)?)
            }
            Path::InfinitelyOften(e) => {
                Path::Eventually(e.replace_identifiers_by_variable_indices(variable_manager)?)
            }
        })
    }
}
