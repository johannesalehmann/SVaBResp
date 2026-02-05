use crate::Identifier;
use crate::expressions::Expression;
use crate::module::RenameRules;
use std::fmt::{Display, Formatter};

pub struct Command<A, E, V, S: Clone> {
    pub action: Option<A>,
    pub guard: E,
    pub updates: Vec<Update<E, V, S>>,
    pub span: S,
}

impl<A, E, V, S: Clone> Command<A, E, V, S> {
    pub fn new(action: Option<A>, guard: E, span: S) -> Self {
        Self {
            action,
            guard,
            updates: Vec::new(),
            span,
        }
    }

    pub fn with_updates(
        action: Option<A>,
        guard: E,
        updates: Vec<Update<E, V, S>>,
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

impl<A, V, S: Clone> Command<A, Expression<V, S>, V, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Command<A, Expression<V, S2>, V, S2> {
        Command {
            action: self.action,
            guard: self.guard.map_span(map),
            updates: self.updates.into_iter().map(|u| u.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<S: Clone> Command<Identifier<S>, Expression<Identifier<S>, S>, Identifier<S>, S> {
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        Self {
            action: self.action.as_ref().map(|a| a.renamed(rename_rules)),
            guard: self.guard.renamed(rename_rules),
            updates: self
                .updates
                .iter()
                .map(|u| u.renamed(rename_rules))
                .collect(),
            span: self.span.clone(),
        }
    }
}

impl<A: Display, E: Display, V: Display, S: Clone> Display for Command<A, E, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        if let Some(action) = &self.action {
            write!(f, "{}", action)?;
        }
        write!(f, "] ")?;
        write!(f, "{} -> ", self.guard)?;
        if self.updates.len() == 0 {
            write!(f, "true")?;
        } else {
            let mut is_first = true;
            for update in &self.updates {
                if !is_first {
                    write!(f, " + ")?;
                }
                is_first = false;
                write!(f, "{}", update)?;
            }
        }

        write!(f, ";")
    }
}

pub struct Update<E, V, S: Clone> {
    pub probability: E,
    pub assignments: Vec<Assignment<E, V, S>>,
    pub span: S,
}

impl<E, V, S: Clone> Update<E, V, S> {
    pub fn new(probability: E, span: S) -> Self {
        Self {
            probability,
            assignments: Vec::new(),
            span,
        }
    }
    pub fn with_assignments(
        probability: E,
        assignments: Vec<Assignment<E, V, S>>,
        span: S,
    ) -> Self {
        Self {
            probability,
            assignments,
            span,
        }
    }
}
impl<V, S: Clone> Update<Expression<V, S>, V, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Update<Expression<V, S2>, V, S2> {
        let mut update = Update::new(self.probability.map_span(map), map(self.span));
        for assignment in self.assignments {
            update.assignments.push(assignment.map_span(map));
        }
        update
    }
}

impl<S: Clone> Update<Expression<Identifier<S>, S>, Identifier<S>, S> {
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        Self {
            probability: self.probability.renamed(rename_rules),
            assignments: self
                .assignments
                .iter()
                .map(|a| a.renamed(rename_rules))
                .collect(),
            span: self.span.clone(),
        }
    }
}

impl<E: Display, V: Display, S: Clone> Display for Update<E, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // This would produce nicer output, but require E to provide some way of inspecting its
        // value, e.g. by requiring a separate IsOne trait.
        // match &self.probability {
        //     Expression::Int(1, _) => {}
        //     e => {
        //         write!(f, "{} : ", e)?;
        //     }
        // }
        write!(f, "{} : ", self.probability)?;

        let mut is_first = true;
        for assignment in &self.assignments {
            if !is_first {
                write!(f, " & ")?;
            }
            is_first = false;
            write!(f, "({})", assignment)?;
        }

        Ok(())
    }
}

pub struct Assignment<E, V, S: Clone> {
    pub target: V,
    pub value: E,
    pub target_span: S,
    pub span: S,
}

impl<E, V, S: Clone> Assignment<E, V, S> {
    pub fn new(target: V, value: E, target_span: S, span: S) -> Self {
        Self {
            target,
            value,
            target_span,
            span,
        }
    }
}
impl<V, S: Clone> Assignment<Expression<V, S>, V, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Assignment<Expression<V, S2>, V, S2> {
        Assignment {
            target: self.target,
            value: self.value.map_span(map),
            target_span: map(self.target_span),
            span: map(self.span),
        }
    }
}
impl<S: Clone> Assignment<Expression<Identifier<S>, S>, Identifier<S>, S> {
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        Self {
            target: self.target.renamed(rename_rules),
            value: self.value.renamed(rename_rules),
            target_span: self.target_span.clone(),
            span: self.span.clone(),
        }
    }
}

impl<E: Display, V: Display, S: Clone> Display for Assignment<E, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'={}", self.target, self.value)
    }
}
