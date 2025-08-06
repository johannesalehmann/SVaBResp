use crate::{Expression, Identifier};
use std::fmt::{Display, Formatter};

pub struct RewardsManager<A, V, S: Clone> {
    pub rewards: Vec<Rewards<A, V, S>>,
}

impl<A, V, S: Clone> RewardsManager<A, V, S> {
    pub fn new() -> Self {
        Self {
            rewards: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Rewards<A, V, S>> {
        self.rewards.get(index)
    }

    pub fn add(&mut self, rewards: Rewards<A, V, S>) -> Result<(), AddRewardsError> {
        for (index, other_rewards) in self.rewards.iter().enumerate() {
            if other_rewards.name == rewards.name {
                return Err(AddRewardsError::RewardsExist { index });
            }
        }
        self.rewards.push(rewards);
        Ok(())
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> RewardsManager<A, V, S2> {
        RewardsManager {
            rewards: self.rewards.into_iter().map(|r| r.map_span(map)).collect(),
        }
    }
}

#[derive(Debug)]
pub enum AddRewardsError {
    RewardsExist { index: usize },
}

pub struct Rewards<A, V, S: Clone> {
    pub name: Option<Identifier<S>>,
    pub entries: Vec<RewardsElement<A, V, S>>,
    pub span: S,
}

impl<A, V, S: Clone> Rewards<A, V, S> {
    pub fn new(name: Option<Identifier<S>>, span: S) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
            span,
        }
    }
    pub fn with_entries(
        name: Option<Identifier<S>>,
        entries: Vec<RewardsElement<A, V, S>>,
        span: S,
    ) -> Self {
        Self {
            name,
            entries,
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Rewards<A, V, S2> {
        Rewards {
            name: self.name.map(|i| i.map_span(map)),
            entries: self.entries.into_iter().map(|e| e.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<A: Display, V: Display, S: Clone> Display for Rewards<A, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "rewards")?;
        if let Some(name) = &self.name {
            write!(f, " \"{}\"", name)?;
        }
        writeln!(f)?;
        for element in &self.entries {
            writeln!(f, "    {}", element)?;
        }
        writeln!(f, "endrewards")
    }
}

pub enum RewardsTarget<A> {
    State,
    Action(Option<A>),
}
impl<A: Display> Display for RewardsTarget<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RewardsTarget::State => Ok(()),
            RewardsTarget::Action(Some(a)) => {
                write!(f, "[{}] ", a)
            }
            RewardsTarget::Action(None) => {
                write!(f, "[] ")
            }
        }
    }
}

pub struct RewardsElement<A, V, S: Clone> {
    pub condition: Expression<V, S>,
    pub value: Expression<V, S>,
    pub target: RewardsTarget<A>,
    pub span: S,
}

impl<A, V, S: Clone> RewardsElement<A, V, S> {
    pub fn new(condition: Expression<V, S>, value: Expression<V, S>, span: S) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::State,
            span,
        }
    }
    pub fn with_action(
        condition: Expression<V, S>,
        value: Expression<V, S>,
        action: Option<A>,
        span: S,
    ) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::Action(action),
            span,
        }
    }
    pub fn with_target(
        condition: Expression<V, S>,
        value: Expression<V, S>,
        target: RewardsTarget<A>,
        span: S,
    ) -> Self {
        Self {
            condition,
            value,
            target,
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> RewardsElement<A, V, S2> {
        RewardsElement {
            condition: self.condition.map_span(map),
            value: self.value.map_span(map),
            target: self.target,
            span: map(self.span),
        }
    }
}

impl<A: Display, V: Display, S: Clone> Display for RewardsElement<A, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}:{};", self.target, self.condition, self.value)
    }
}
