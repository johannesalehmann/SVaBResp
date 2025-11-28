use std::fmt::Formatter;

#[derive(Clone)]
pub struct Property<S: StateSpecifier> {
    pub operator: Operator,
    pub path: Path<S>,
}

#[derive(Clone)]
pub enum Operator {
    ValueOfPMax,
    ValueOfPMin,
    ValueOfP,
}

#[derive(Clone)]
pub enum Path<S: StateSpecifier> {
    Eventually(S),
}

pub trait StateSpecifier {}

impl<S: std::fmt::Debug + StateSpecifier> std::fmt::Debug for Property<S> {
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
impl<S: std::fmt::Debug + StateSpecifier> std::fmt::Debug for Path<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::Eventually(e) => {
                write!(f, "F {:?}", e)
            }
        }
    }
}
