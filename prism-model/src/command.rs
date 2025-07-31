use crate::expressions::Expression;

pub struct Command<A, V, S> {
    pub action: Option<A>,
    pub guard: Expression<V, S>,
    pub updates: Vec<Update<V, S>>,
    pub span: S,
}

impl<A, V, S> Command<A, V, S> {
    pub fn new(action: Option<A>, guard: Expression<V, S>, span: S) -> Self {
        Self {
            action,
            guard,
            updates: Vec::new(),
            span,
        }
    }

    pub fn with_updates(
        action: Option<A>,
        guard: Expression<V, S>,
        updates: Vec<Update<V, S>>,
        span: S,
    ) -> Self {
        Self {
            action,
            guard,
            updates,
            span,
        }
    }
}

pub struct Update<V, S> {
    pub probability: Expression<V, S>,
    pub assignments: Vec<Assignment<V, S>>,
    pub span: S,
}

impl<V, S> Update<V, S> {
    pub fn new(probability: Expression<V, S>, span: S) -> Self {
        Self {
            probability,
            assignments: Vec::new(),
            span,
        }
    }
    pub fn with_assignemnts(
        probability: Expression<V, S>,
        assignments: Vec<Assignment<V, S>>,
        span: S,
    ) -> Self {
        Self {
            probability,
            assignments,
            span,
        }
    }
}

pub struct Assignment<V, S> {
    pub target: V,
    pub value: Expression<V, S>,
    pub target_span: S,
    pub span: S,
}

impl<V, S> Assignment<V, S> {
    pub fn new(target: V, value: Expression<V, S>, target_span: S, span: S) -> Self {
        Self {
            target,
            value,
            target_span,
            span,
        }
    }
}
