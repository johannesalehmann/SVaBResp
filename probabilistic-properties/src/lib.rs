#[derive(Clone)]
pub enum Query<I, F, E> {
    ProbabilityValue {
        non_determinism: Option<NonDeterminismKind>,
        path: PathFormula<I, F, E>,
    },
    StateFormula(StateFormula<I, F, E>),
    RewardBound {
        non_determinism: Option<NonDeterminismKind>,
        name: Option<String>,
        bound: Bound<F>,
        reward: RewardFormula<I, F, E>,
    },
    RewardValue {
        non_determinism: Option<NonDeterminismKind>,
        name: Option<String>,
        reward: RewardFormula<I, F, E>,
    },
    TimeBound {
        non_determinism: Option<NonDeterminismKind>,
        bound: Bound<F>,
        reward: RewardFormula<I, F, E>,
    },
    TimeValue {
        non_determinism: Option<NonDeterminismKind>,
        reward: RewardFormula<I, F, E>,
    },
}

impl<I, F, E> Query<I, F, E> {
    pub fn as_mut(&mut self) -> Query<&mut I, &mut F, &mut E> {
        match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism: *non_determinism,
                path: path.as_mut(),
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.as_mut()),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => {
                Query::RewardBound {
                    non_determinism: *non_determinism,
                    name: name.clone(), // TODO: Avoid cloning name here?
                    bound: bound.as_mut(),
                    reward: reward.as_mut(),
                }
            }
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism: *non_determinism,
                name: name.clone(),
                reward: reward.as_mut(),
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism: *non_determinism,
                bound: bound.as_mut(),
                reward: reward.as_mut(),
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism: *non_determinism,
                reward: reward.as_mut(),
            },
        }
    }

    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> Query<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I2, F, E>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                path: path.try_map_i(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_i(map)?),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward: reward.try_map_i(map)?,
            },
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                name,
                reward: reward.try_map_i(map)?,
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                bound,
                reward: reward.try_map_i(map)?,
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism,
                reward: reward.try_map_i(map)?,
            },
        })
    }

    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> Query<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I, F2, E>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                path: path.try_map_f(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_f(map)?),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                name,
                bound: bound.try_map_value(map)?,
                reward: reward.try_map_f(map)?,
            },
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                name,
                reward: reward.try_map_f(map)?,
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                bound: bound.try_map_value(map)?,
                reward: reward.try_map_f(map)?,
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism,
                reward: reward.try_map_f(map)?,
            },
        })
    }

    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> Query<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<Query<I, F, E2>, Er> {
        Ok(match self {
            Query::ProbabilityValue {
                non_determinism,
                path,
            } => Query::ProbabilityValue {
                non_determinism,
                path: path.try_map_e(map)?,
            },
            Query::StateFormula(formula) => Query::StateFormula(formula.try_map_e(map)?),
            Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward,
            } => Query::RewardBound {
                non_determinism,
                name,
                bound,
                reward: reward.try_map_e(map)?,
            },
            Query::RewardValue {
                non_determinism,
                name,
                reward,
            } => Query::RewardValue {
                non_determinism,
                name,
                reward: reward.try_map_e(map)?,
            },
            Query::TimeBound {
                non_determinism,
                bound,
                reward,
            } => Query::TimeBound {
                non_determinism,
                bound,
                reward: reward.try_map_e(map)?,
            },
            Query::TimeValue {
                non_determinism,
                reward,
            } => Query::TimeValue {
                non_determinism,
                reward: reward.try_map_e(map)?,
            },
        })
    }
}

#[derive(Clone)]
pub enum StateFormula<I, F, E> {
    Expression(E),
    ProbabilityBound {
        non_determinism: Option<NonDeterminismKind>,
        bound: Bound<F>,
        path: Box<PathFormula<I, F, E>>,
    },
    LongRunAverage {
        non_determinism: Option<NonDeterminismKind>,
        bound: Bound<F>,
        path: Box<PathFormula<I, F, E>>,
    },
}

impl<I, F, E> StateFormula<I, F, E> {
    pub fn as_mut(&mut self) -> StateFormula<&mut I, &mut F, &mut E> {
        match self {
            StateFormula::Expression(e) => StateFormula::Expression(e),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism: *non_determinism,
                bound: bound.as_mut(),
                path: Box::new(PathFormula::as_mut(path)),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound,
                path,
            } => StateFormula::LongRunAverage {
                non_determinism: *non_determinism,
                bound: bound.as_mut(),
                path: Box::new(PathFormula::as_mut(path)),
            },
        }
    }

    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> StateFormula<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I2, F, E>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(expression),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path: Box::new(path.try_map_i(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound: operator,
                path,
            } => StateFormula::LongRunAverage {
                non_determinism,
                bound: operator,
                path: Box::new(path.try_map_i(map)?),
            },
        })
    }

    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> StateFormula<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I, F2, E>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(expression),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                bound: bound.try_map_value(map)?,
                path: Box::new(path.try_map_f(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound: operator,
                path,
            } => StateFormula::LongRunAverage {
                non_determinism,
                bound: operator.try_map_value(map)?,
                path: Box::new(path.try_map_f(map)?),
            },
        })
    }

    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> StateFormula<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<StateFormula<I, F, E2>, Er> {
        Ok(match self {
            StateFormula::Expression(expression) => StateFormula::Expression(map(expression)?),
            StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path,
            } => StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path: Box::new(path.try_map_e(map)?),
            },
            StateFormula::LongRunAverage {
                non_determinism,
                bound: operator,
                path,
            } => StateFormula::LongRunAverage {
                non_determinism,
                bound: operator,
                path: Box::new(path.try_map_e(map)?),
            },
        })
    }
}

#[derive(Clone)]
pub enum PathFormula<I, F, E> {
    Until {
        before: Box<StateFormula<I, F, E>>,
        after: Box<StateFormula<I, F, E>>,
    },
    Eventually {
        condition: Box<StateFormula<I, F, E>>,
    },
    BoundedUntil {
        before: Box<StateFormula<I, F, E>>,
        after: Box<StateFormula<I, F, E>>,
        bound: Bound<I>,
    },
    BoundedEventually {
        condition: Box<StateFormula<I, F, E>>,
        bound: Bound<I>,
    },
    Generally {
        condition: Box<StateFormula<I, F, E>>,
    },
}

impl<I, F, E> PathFormula<I, F, E> {
    pub fn eventually_condition(&self) -> Option<&StateFormula<I, F, E>> {
        match self {
            PathFormula::Eventually { condition } => Some(condition),
            _ => None,
        }
    }
    pub fn generally_condition(&self) -> Option<&StateFormula<I, F, E>> {
        match self {
            PathFormula::Generally { condition } => Some(condition),
            _ => None,
        }
    }

    pub fn as_mut(&mut self) -> PathFormula<&mut I, &mut F, &mut E> {
        match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(StateFormula::as_mut(before)),
                after: Box::new(StateFormula::as_mut(after)),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(StateFormula::as_mut(condition)),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(StateFormula::as_mut(before)),
                after: Box::new(StateFormula::as_mut(after)),
                bound: bound.as_mut(),
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(StateFormula::as_mut(condition)),
                bound: bound.as_mut(),
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(StateFormula::as_mut(condition)),
            },
        }
    }

    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> PathFormula<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I2, F, E>, Er> {
        Ok(match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(before.try_map_i(map)?),
                after: Box::new(after.try_map_i(map)?),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(condition.try_map_i(map)?),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(before.try_map_i(map)?),
                after: Box::new(after.try_map_i(map)?),
                bound: bound.try_map_value(map)?,
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(condition.try_map_i(map)?),
                bound: bound.try_map_value(map)?,
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(condition.try_map_i(map)?),
            },
        })
    }

    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> PathFormula<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I, F2, E>, Er> {
        Ok(match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(before.try_map_f(map)?),
                after: Box::new(after.try_map_f(map)?),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(condition.try_map_f(map)?),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(before.try_map_f(map)?),
                after: Box::new(after.try_map_f(map)?),
                bound,
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(condition.try_map_f(map)?),
                bound,
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(condition.try_map_f(map)?),
            },
        })
    }

    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> PathFormula<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<PathFormula<I, F, E2>, Er> {
        Ok(match self {
            PathFormula::Until { before, after } => PathFormula::Until {
                before: Box::new(before.try_map_e(map)?),
                after: Box::new(after.try_map_e(map)?),
            },
            PathFormula::Eventually { condition } => PathFormula::Eventually {
                condition: Box::new(condition.try_map_e(map)?),
            },
            PathFormula::BoundedUntil {
                before,
                after,
                bound,
            } => PathFormula::BoundedUntil {
                before: Box::new(before.try_map_e(map)?),
                after: Box::new(after.try_map_e(map)?),
                bound,
            },
            PathFormula::BoundedEventually { condition, bound } => PathFormula::BoundedEventually {
                condition: Box::new(condition.try_map_e(map)?),
                bound,
            },
            PathFormula::Generally { condition } => PathFormula::Generally {
                condition: Box::new(condition.try_map_e(map)?),
            },
        })
    }
}
#[derive(Clone)]
pub enum RewardFormula<I, F, E> {
    Instantaneous { k: F },
    Cumulative { k: F },
    Finally { states: StateFormula<I, F, E> },
    LongRunAverage,
}

impl<I, F, E> RewardFormula<I, F, E> {
    pub fn as_mut(&mut self) -> RewardFormula<&mut I, &mut F, &mut E> {
        match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.as_mut(),
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        }
    }

    pub fn map_i<I2, M: FnMut(I) -> I2>(self, map: &mut M) -> RewardFormula<I2, F, E> {
        self.try_map_i(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_i<Er, I2, M: FnMut(I) -> Result<I2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I2, F, E>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_i(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }

    pub fn map_f<F2, M: FnMut(F) -> F2>(self, map: &mut M) -> RewardFormula<I, F2, E> {
        self.try_map_f(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_f<Er, F2, M: FnMut(F) -> Result<F2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I, F2, E>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k: map(k)? },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k: map(k)? },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_f(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }

    pub fn map_e<E2, M: FnMut(E) -> E2>(self, map: &mut M) -> RewardFormula<I, F, E2> {
        self.try_map_e(&mut |ex| Result::<_, ()>::Ok(map(ex)))
            .unwrap()
    }

    pub fn try_map_e<Er, E2, M: FnMut(E) -> Result<E2, Er>>(
        self,
        map: &mut M,
    ) -> Result<RewardFormula<I, F, E2>, Er> {
        Ok(match self {
            RewardFormula::Instantaneous { k } => RewardFormula::Instantaneous { k },
            RewardFormula::Cumulative { k } => RewardFormula::Cumulative { k },
            RewardFormula::Finally { states: state } => RewardFormula::Finally {
                states: state.try_map_e(map)?,
            },
            RewardFormula::LongRunAverage => RewardFormula::LongRunAverage,
        })
    }
}

#[derive(Clone)]
pub struct Bound<V> {
    pub operator: BoundOperator,
    pub value: V,
}

impl<V> Bound<V> {
    pub fn as_mut(&mut self) -> Bound<&mut V> {
        Bound {
            operator: self.operator,
            value: &mut self.value,
        }
    }

    pub fn map_value<V2, F: FnMut(V) -> V2>(self, map: &mut F) -> Bound<V2> {
        Bound {
            operator: self.operator,
            value: map(self.value),
        }
    }

    pub fn try_map_value<Er, V2, F: FnMut(V) -> Result<V2, Er>>(
        self,
        map: &mut F,
    ) -> Result<Bound<V2>, Er> {
        Ok(Bound {
            operator: self.operator,
            value: map(self.value)?,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BoundOperator {
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NonDeterminismKind {
    Maximise,
    Minimise,
}
