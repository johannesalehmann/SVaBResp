use crate::{Expression, Identifier};
use std::fmt::{Display, Formatter};

pub struct RewardsManager<A, E, S: Clone> {
    pub rewards: Vec<Rewards<A, E, S>>,
}

impl<A, E, S: Clone> RewardsManager<A, E, S> {
    pub fn new() -> Self {
        Self {
            rewards: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Rewards<A, E, S>> {
        self.rewards.get(index)
    }

    pub fn add(&mut self, rewards: Rewards<A, E, S>) -> Result<(), AddRewardsError> {
        for (index, other_rewards) in self.rewards.iter().enumerate() {
            if other_rewards.name == rewards.name {
                return Err(AddRewardsError::RewardsExist { index });
            }
        }
        self.rewards.push(rewards);
        Ok(())
    }
}

impl<A, V, S: Clone> RewardsManager<A, Expression<V, S>, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> RewardsManager<A, Expression<V, S2>, S2> {
        RewardsManager {
            rewards: self.rewards.into_iter().map(|r| r.map_span(map)).collect(),
        }
    }
}

#[derive(Debug)]
pub enum AddRewardsError {
    RewardsExist { index: usize },
}

pub struct Rewards<A, E, S: Clone> {
    pub name: Option<Identifier<S>>,
    pub entries: Vec<RewardsElement<A, E, S>>,
    pub span: S,
}

impl<A, E, S: Clone> Rewards<A, E, S> {
    pub fn new(name: Option<Identifier<S>>, span: S) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
            span,
        }
    }
    pub fn with_entries(
        name: Option<Identifier<S>>,
        entries: Vec<RewardsElement<A, E, S>>,
        span: S,
    ) -> Self {
        Self {
            name,
            entries,
            span,
        }
    }
}
impl<A, V, S: Clone> Rewards<A, Expression<V, S>, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Rewards<A, Expression<V, S2>, S2> {
        Rewards {
            name: self.name.map(|i| i.map_span(map)),
            entries: self.entries.into_iter().map(|e| e.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<A: Display, E: Display, S: Clone> Display for Rewards<A, E, S> {
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

#[derive(Clone)]
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

pub struct RewardsElement<A, E, S: Clone> {
    pub condition: E,
    pub value: E,
    pub target: RewardsTarget<A>,
    pub span: S,
}

impl<A, E, S: Clone> RewardsElement<A, E, S> {
    pub fn new(condition: E, value: E, span: S) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::State,
            span,
        }
    }
    pub fn with_action(condition: E, value: E, action: Option<A>, span: S) -> Self {
        Self {
            condition,
            value,
            target: RewardsTarget::Action(action),
            span,
        }
    }
    pub fn with_target(condition: E, value: E, target: RewardsTarget<A>, span: S) -> Self {
        Self {
            condition,
            value,
            target,
            span,
        }
    }
}
impl<A, V, S: Clone> RewardsElement<A, Expression<V, S>, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> RewardsElement<A, Expression<V, S2>, S2> {
        RewardsElement {
            condition: self.condition.map_span(map),
            value: self.value.map_span(map),
            target: self.target,
            span: map(self.span),
        }
    }
}

impl<A: Display, E: Display, S: Clone> Display for RewardsElement<A, E, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}:{};", self.target, self.condition, self.value)
    }
}
