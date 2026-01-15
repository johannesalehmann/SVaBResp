use crate::expressions::Expression;
use crate::module::RenameRules;
use crate::Identifier;
use std::fmt::{Display, Formatter};

pub struct Command<A, V, S: Clone> {
    pub action: Option<A>,
    pub guard: Expression<V, S>,
    pub updates: Vec<Update<V, S>>,
    pub span: S,
}

impl<A, V, S: Clone> Command<A, V, S> {
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

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Command<A, V, S2> {
        Command {
            action: self.action,
            guard: self.guard.map_span(map),
            updates: self.updates.into_iter().map(|u| u.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<S: Clone> Command<Identifier<S>, Identifier<S>, S> {
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

impl<A: Display, V: Display, S: Clone> Display for Command<A, V, S> {
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

pub struct Update<V, S: Clone> {
    pub probability: Expression<V, S>,
    pub assignments: Vec<Assignment<V, S>>,
    pub span: S,
}

impl<V, S: Clone> Update<V, S> {
    pub fn new(probability: Expression<V, S>, span: S) -> Self {
        Self {
            probability,
            assignments: Vec::new(),
            span,
        }
    }
    pub fn with_assignments(
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

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Update<V, S2> {
        let mut update = Update::new(self.probability.map_span(map), map(self.span));
        for assignment in self.assignments {
            update.assignments.push(assignment.map_span(map));
        }
        update
    }
}

impl<S: Clone> Update<Identifier<S>, S> {
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

impl<V: Display, S: Clone> Display for Update<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.probability {
            Expression::Int(1, _) => {}
            e => {
                write!(f, "{} : ", e)?;
            }
        }

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

pub struct Assignment<V, S: Clone> {
    pub target: V,
    pub value: Expression<V, S>,
    pub target_span: S,
    pub span: S,
}

impl<V, S: Clone> Assignment<V, S> {
    pub fn new(target: V, value: Expression<V, S>, target_span: S, span: S) -> Self {
        Self {
            target,
            value,
            target_span,
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Assignment<V, S2> {
        Assignment {
            target: self.target,
            value: self.value.map_span(map),
            target_span: map(self.target_span),
            span: map(self.span),
        }
    }
}
impl<S: Clone> Assignment<Identifier<S>, S> {
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        Self {
            target: self.target.renamed(rename_rules),
            value: self.value.renamed(rename_rules),
            target_span: self.target_span.clone(),
            span: self.span.clone(),
        }
    }
}

impl<V: Display, S: Clone> Display for Assignment<V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'={}", self.target, self.value)
    }
}
