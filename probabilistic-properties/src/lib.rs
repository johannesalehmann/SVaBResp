use std::fmt::Formatter;

#[derive(Clone)]
pub struct Property<S: StateSpecifier, P: ProbabilitySpecifier> {
    pub operator: ProbabilityOperator<P>,
    pub path: Path<S>,
}

#[derive(Clone)]
pub struct ProbabilityOperator<P: ProbabilitySpecifier> {
    pub kind: ProbabilityKind,
    pub constraint: ProbabilityConstraint<P>,
}

#[derive(Clone, PartialEq)]
pub enum ProbabilityKind {
    PMax,
    PMin,
    P,
}

#[derive(Clone)]
pub enum ProbabilityConstraint<P: ProbabilitySpecifier> {
    ValueOf,
    EqualTo(P),
    GreaterThan(P),
    GreaterOrEqual(P),
    LessThan(P),
    LessOrEqual(P),
}

impl<P: ProbabilitySpecifier> ProbabilityConstraint<P> {
    pub fn map_probability_specifier<Q: ProbabilitySpecifier, F: Fn(P) -> Q>(
        self,
        f: F,
    ) -> ProbabilityConstraint<Q> {
        self.map_probability_specifier_with_result::<(), _, _>(|p| Ok(f(p)))
            .unwrap()
    }
    pub fn map_probability_specifier_with_result<
        Err,
        Q: ProbabilitySpecifier,
        F: Fn(P) -> Result<Q, Err>,
    >(
        self,
        f: F,
    ) -> Result<ProbabilityConstraint<Q>, Err> {
        Ok(match self {
            ProbabilityConstraint::ValueOf => ProbabilityConstraint::ValueOf,
            ProbabilityConstraint::EqualTo(p) => ProbabilityConstraint::EqualTo(f(p)?),
            ProbabilityConstraint::GreaterThan(p) => ProbabilityConstraint::GreaterThan(f(p)?),
            ProbabilityConstraint::GreaterOrEqual(p) => {
                ProbabilityConstraint::GreaterOrEqual(f(p)?)
            }
            ProbabilityConstraint::LessThan(p) => ProbabilityConstraint::LessThan(f(p)?),
            ProbabilityConstraint::LessOrEqual(p) => ProbabilityConstraint::LessOrEqual(f(p)?),
        })
    }
    pub fn transform_probability_specifier<F: Fn(&mut P)>(&mut self, f: F) {
        self.transform_probability_specifier_with_result::<(), _>(|p| {
            f(p);
            Ok(())
        })
        .unwrap();
    }
    pub fn transform_probability_specifier_with_result<Err, F: Fn(&mut P) -> Result<(), Err>>(
        &mut self,
        f: F,
    ) -> Result<(), Err> {
        match self {
            ProbabilityConstraint::ValueOf => Ok(()),
            ProbabilityConstraint::EqualTo(p) => f(p),
            ProbabilityConstraint::GreaterThan(p) => f(p),
            ProbabilityConstraint::GreaterOrEqual(p) => f(p),
            ProbabilityConstraint::LessThan(p) => f(p),
            ProbabilityConstraint::LessOrEqual(p) => f(p),
        }
    }
}

pub trait ProbabilitySpecifier {}
impl ProbabilitySpecifier for f64 {}

#[derive(Clone)]
pub enum Path<S: StateSpecifier> {
    Eventually(S),
    Generally(S),
    InfinitelyOften(S),
}

pub trait StateSpecifier {}

impl<S: StateSpecifier> Path<S> {
    pub fn map_state_specifier<T: StateSpecifier, F: Fn(S) -> T>(self, f: F) -> Path<T> {
        match self {
            Path::Eventually(s) => Path::Eventually(f(s)),
            Path::Generally(s) => Path::Generally(f(s)),
            Path::InfinitelyOften(s) => Path::InfinitelyOften(f(s)),
        }
    }
}

impl<S: std::fmt::Debug + StateSpecifier, P: std::fmt::Debug + ProbabilitySpecifier> std::fmt::Debug
    for Property<S, P>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} [{:?}]", self.operator, self.path)
    }
}
impl<P: std::fmt::Debug + ProbabilitySpecifier> std::fmt::Debug for ProbabilityOperator<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.kind, self.constraint)
    }
}
impl std::fmt::Debug for ProbabilityKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbabilityKind::PMax => {
                write!(f, "PMax")
            }
            ProbabilityKind::PMin => {
                write!(f, "PMin")
            }
            ProbabilityKind::P => {
                write!(f, "P")
            }
        }
    }
}
impl<P: std::fmt::Debug + ProbabilitySpecifier> std::fmt::Debug for ProbabilityConstraint<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbabilityConstraint::ValueOf => {
                write!(f, "=?")
            }
            ProbabilityConstraint::EqualTo(p) => {
                write!(f, "={:?}", p)
            }
            ProbabilityConstraint::GreaterThan(p) => {
                write!(f, ">{:?}", p)
            }
            ProbabilityConstraint::GreaterOrEqual(p) => {
                write!(f, ">={:?}", p)
            }
            ProbabilityConstraint::LessThan(p) => {
                write!(f, "<{:?}", p)
            }
            ProbabilityConstraint::LessOrEqual(p) => {
                write!(f, "<={:?}", p)
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
            Path::Generally(e) => {
                write!(f, "G {:?}", e)
            }
            Path::InfinitelyOften(e) => {
                write!(f, "G F {:?}", e)
            }
        }
    }
}
