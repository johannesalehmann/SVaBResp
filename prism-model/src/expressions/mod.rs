mod label_substitution;
mod maps;

pub use maps::*;
use std::fmt::{Display, Formatter};

use crate::expressions::label_substitution::LabelSubstitutionVisitor;
use crate::expressions::map_variable::MapVariable;
use crate::module::RenameRules;
use crate::{
    CyclicDependency, FormulaManager, Identifier, LabelManager, VariableManager, VariableReference,
};

#[derive(PartialEq, Clone)]
pub struct GlobalVariableReference {
    reference: VariableReference,
    scope: VariableScope,
}

#[derive(PartialEq, Clone)]
pub enum VariableScope {
    GlobalVariable,
    GlobalConst,
    Formula,
    LocalVariable { module: String },
}

#[derive(PartialEq, Clone)]
pub enum Expression<V, S: Clone> {
    Int(i64, S),
    Float(f64, S),
    Bool(bool, S),
    VarOrConst(V, S),
    Label(V, S),
    Function(Identifier<S>, Vec<Expression<V, S>>, S),
    Minus(Box<Expression<V, S>>, S),
    Multiplication(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Division(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Addition(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Subtraction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    LessThan(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    LessOrEqual(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    GreaterThan(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    GreaterOrEqual(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Equals(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    NotEquals(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Negation(Box<Expression<V, S>>, S),
    Conjunction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Disjunction(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    IfAndOnlyIf(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Implies(Box<Expression<V, S>>, Box<Expression<V, S>>, S),
    Ternary(
        Box<Expression<V, S>>,
        Box<Expression<V, S>>,
        Box<Expression<V, S>>,
        S,
    ),
}

impl<V, S: Clone> Expression<V, S> {
    pub fn span(&self) -> &S {
        match self {
            Expression::Int(_, s) => s,
            Expression::Float(_, s) => s,
            Expression::Bool(_, s) => s,
            Expression::VarOrConst(_, s) => s,
            Expression::Label(_, s) => s,
            Expression::Function(_, _, s) => s,
            Expression::Minus(_, s) => s,
            Expression::Multiplication(_, _, s) => s,
            Expression::Division(_, _, s) => s,
            Expression::Addition(_, _, s) => s,
            Expression::Subtraction(_, _, s) => s,
            Expression::LessThan(_, _, s) => s,
            Expression::LessOrEqual(_, _, s) => s,
            Expression::GreaterThan(_, _, s) => s,
            Expression::GreaterOrEqual(_, _, s) => s,
            Expression::Equals(_, _, s) => s,
            Expression::NotEquals(_, _, s) => s,
            Expression::Negation(_, s) => s,
            Expression::Conjunction(_, _, s) => s,
            Expression::Disjunction(_, _, s) => s,
            Expression::IfAndOnlyIf(_, _, s) => s,
            Expression::Implies(_, _, s) => s,
            Expression::Ternary(_, _, _, s) => s,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Expression<V, S2> {
        let mut visitor = maps::map_span::MapSpan::new(map);
        self.visit(&mut visitor)
    }

    fn get_precedence(&self) -> usize {
        // As per https://www.prismmodelchecker.org/manual/ThePRISMLanguage/Expressions
        // ranging from 1 (for ternary) to 11 (for unary minus)
        // Precedence 12 is used for atoms (literals, variables)
        // 0 is used to indicate there is no surrounding precedence
        match self {
            Expression::Int(_, _) => 12,
            Expression::Float(_, _) => 12,
            Expression::Bool(_, _) => 12,
            Expression::VarOrConst(_, _) => 12,
            Expression::Label(_, _) => 12,
            Expression::Function(_, _, _) => 12,
            Expression::Minus(_, _) => 11,
            Expression::Multiplication(_, _, _) => 10,
            Expression::Division(_, _, _) => 10,
            Expression::Addition(_, _, _) => 9,
            Expression::Subtraction(_, _, _) => 9,
            Expression::LessThan(_, _, _) => 8,
            Expression::LessOrEqual(_, _, _) => 8,
            Expression::GreaterThan(_, _, _) => 8,
            Expression::GreaterOrEqual(_, _, _) => 8,
            Expression::Equals(_, _, _) => 7,
            Expression::NotEquals(_, _, _) => 7,
            Expression::Negation(_, _) => 6,
            Expression::Conjunction(_, _, _) => 5,
            Expression::Disjunction(_, _, _) => 4,
            Expression::IfAndOnlyIf(_, _, _) => 3,
            Expression::Implies(_, _, _) => 2,
            Expression::Ternary(_, _, _, _) => 1,
        }
    }
}
impl<S: Clone> Expression<Identifier<S>, S> {
    pub fn substitute_labels(
        &mut self,
        default_span: S,
        labels: &LabelManager<Expression<Identifier<S>, S>, S>,
    ) {
        for label in &labels.labels {
            let mut visitor = LabelSubstitutionVisitor {
                label_name: &label.name,
                expression: &label.condition,
            };

            let condition = std::mem::replace(self, Expression::Bool(false, default_span.clone()));
            *self = condition.visit(&mut visitor);
        }
    }
    pub fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<Expression<Identifier<S>, S>, S>,
    ) -> Result<(), CyclicDependency<S>> {
        let order = formulas.get_formula_replacement_ordering()?;

        for formula_index in order {
            let formula = formulas.get(formula_index).unwrap();
            let mut visitor = crate::model::FormulaSubstitutionVisitor {
                formula_name: &formula.name,
                expression: &formula.condition,
            };

            let condition = std::mem::replace(self, Expression::Bool(false, default_span.clone()));
            *self = condition.visit(&mut visitor);
        }

        Ok(())
    }
}

impl<S: Clone> Expression<Identifier<S>, S> {
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        let mut visitor = RenamingVisitor { rename_rules };
        self.clone().visit(&mut visitor) // This clone is not required in principle, but cannot be avoided as long as visitors consume their expression
    }
}
struct RenamingVisitor<'a, S: Clone> {
    rename_rules: &'a RenameRules<S>,
}

impl<'a, S: Clone> identity_map::Private for RenamingVisitor<'a, S> {}
impl<'a, S: Clone> IdentityMapExpression<Identifier<S>, S> for RenamingVisitor<'a, S> {
    fn visit_var_or_const(&mut self, name: Identifier<S>, span: S) -> Expression<Identifier<S>, S> {
        match self.rename_rules.get_renaming(&name) {
            None => Expression::VarOrConst(name, span),
            Some(renaming) => Expression::VarOrConst(renaming, span),
        }
    }
}
impl<S: Clone> Expression<Identifier<S>, S> {
    pub fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<R, S>,
    ) -> Result<Expression<VariableReference, S>, Vec<UnknownVariableError<S>>> {
        let errors = Vec::new();
        let mut replace_visitor: MapVariable<Identifier<S>, VariableReference, _, _> =
            MapVariable::new(
                |f, e| match variable_manager.get_reference(&f) {
                    Some(index) => index,
                    None => {
                        e.push(UnknownVariableError {
                            identifier: f.clone(),
                        });
                        VariableReference::new(0)
                    }
                },
                errors,
            );
        let new_expression = self.visit(&mut replace_visitor);
        if !replace_visitor.context.is_empty() {
            Err(replace_visitor.context)
        } else {
            Ok(new_expression)
        }
    }
}

#[derive(Clone)]
pub struct UnknownVariableError<S: Clone> {
    pub identifier: Identifier<S>,
}

impl<V: Display, S: Clone> Expression<V, S> {
    fn fmt_internal(
        &self,
        f: &mut Formatter<'_>,
        surrounding_precedence: usize,
    ) -> std::fmt::Result {
        let precedence = self.get_precedence();
        if surrounding_precedence > precedence {
            write!(f, "(")?;
        }
        match self {
            Expression::Int(a, _) => {
                write!(f, "{}", a)?;
            }
            Expression::Float(a, _) => {
                write!(f, "{}", a)?;
            }
            Expression::Bool(true, _) => {
                write!(f, "true")?;
            }
            Expression::VarOrConst(name, _) => {
                write!(f, "{}", name)?;
            }
            Expression::Label(name, _) => {
                write!(f, "\"{}\"", name)?;
            }
            Expression::Bool(false, _) => {
                write!(f, "false")?;
            }
            Expression::Function(n, a, _) => {
                write!(
                    f,
                    "{}({})",
                    n,
                    a.iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }
            Expression::Minus(inner, _) => {
                write!(f, "-")?;
                inner.fmt_internal(f, precedence)?;
            }
            Expression::Multiplication(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "*")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Division(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "/")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Addition(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "+")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Subtraction(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "-")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::LessThan(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "<")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::LessOrEqual(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "<=")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::GreaterThan(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, ">")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::GreaterOrEqual(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, ">=")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Equals(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "==")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::NotEquals(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "!=")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Negation(inner, _) => {
                write!(f, "!")?;
                inner.fmt_internal(f, precedence + 1)?;
            }
            Expression::Conjunction(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "&")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Disjunction(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "|")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::IfAndOnlyIf(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "<=>")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Implies(lhs, rhs, _) => {
                lhs.fmt_internal(f, precedence)?;
                write!(f, "=>")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
            Expression::Ternary(cond, lhs, rhs, _) => {
                cond.fmt_internal(f, precedence + 1)?;
                write!(f, "?")?;
                lhs.fmt_internal(f, precedence + 1)?;
                write!(f, ":")?;
                rhs.fmt_internal(f, precedence + 1)?;
            }
        }

        if surrounding_precedence > precedence {
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl<V: std::fmt::Debug, S: Clone> std::fmt::Debug for Expression<V, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Int(a, _) => {
                write!(f, "{}", a)
            }
            Expression::Float(a, _) => {
                write!(f, "{}", a)
            }
            Expression::Bool(true, _) => {
                write!(f, "true")
            }
            Expression::VarOrConst(name, _) => {
                write!(f, "{:?}", name)
            }
            Expression::Label(name, _) => {
                write!(f, "\"{:?}\"", name)
            }
            Expression::Bool(false, _) => {
                write!(f, "false")
            }
            Expression::Function(n, a, _) => {
                write!(
                    f,
                    "{:?}({})",
                    n,
                    a.iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expression::Minus(a, _) => {
                write!(f, "-({:?})", a)
            }
            Expression::Multiplication(a, b, _) => {
                write!(f, "({:?})*({:?})", a, b)
            }
            Expression::Division(a, b, _) => {
                write!(f, "({:?})/({:?})", a, b)
            }
            Expression::Addition(a, b, _) => {
                write!(f, "({:?})+({:?})", a, b)
            }
            Expression::Subtraction(a, b, _) => {
                write!(f, "({:?})-({:?})", a, b)
            }
            Expression::LessThan(a, b, _) => {
                write!(f, "({:?})<({:?})", a, b)
            }
            Expression::LessOrEqual(a, b, _) => {
                write!(f, "({:?})<=({:?})", a, b)
            }
            Expression::GreaterThan(a, b, _) => {
                write!(f, "({:?})>({:?})", a, b)
            }
            Expression::GreaterOrEqual(a, b, _) => {
                write!(f, "({:?})>=({:?})", a, b)
            }
            Expression::Equals(a, b, _) => {
                write!(f, "({:?})=({:?})", a, b)
            }
            Expression::NotEquals(a, b, _) => {
                write!(f, "({:?})!=({:?})", a, b)
            }
            Expression::Negation(a, _) => {
                write!(f, "!({:?})", a)
            }
            Expression::Conjunction(a, b, _) => {
                write!(f, "({:?})&({:?})", a, b)
            }
            Expression::Disjunction(a, b, _) => {
                write!(f, "({:?})|({:?})", a, b)
            }
            Expression::IfAndOnlyIf(a, b, _) => {
                write!(f, "({:?})<=>({:?})", a, b)
            }
            Expression::Implies(a, b, _) => {
                write!(f, "({:?})=>({:?})", a, b)
            }
            Expression::Ternary(a, b, c, _) => {
                write!(f, "({:?})?({:?}):({:?})", a, b, c)
            }
        }
    }
}

impl<V: Display, S: Clone> Display for Expression<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_internal(f, 0)
    }
}
