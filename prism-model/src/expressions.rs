use crate::{Identifier, VariableReference};

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
pub enum Expression<V, S> {
    Int(i64, S),
    Float(f64, S),
    Bool(bool, S),
    VarOrConst(V, S),
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

impl<V, S> Expression<V, S> {
    pub fn span(&self) -> &S {
        match self {
            Expression::Int(_, s) => s,
            Expression::Float(_, s) => s,
            Expression::Bool(_, s) => s,
            Expression::VarOrConst(_, s) => s,
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
}

impl<V, S> std::fmt::Debug for Expression<V, S>
where
    V: std::fmt::Debug,
{
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
