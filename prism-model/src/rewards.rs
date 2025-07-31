use crate::{AddFormulaError, Expression, Identifier};

pub struct RewardsManager<A, V, S> {
    rewards: Vec<Rewards<A, V, S>>,
}

impl<A, V, S> RewardsManager<A, V, S> {
    pub fn new() -> Self {
        Self {
            rewards: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Rewards<A, V, S>> {
        self.rewards.get(index)
    }

    pub fn add_rewards(&mut self, rewards: Rewards<A, V, S>) -> Result<(), AddRewardsError> {
        for (index, other_rewards) in self.rewards.iter().enumerate() {
            if other_rewards.name == rewards.name {
                return Err(AddRewardsError::RewardsExist { index });
            }
        }
        self.rewards.push(rewards);
        Ok(())
    }
}

#[derive(Debug)]
pub enum AddRewardsError {
    RewardsExist { index: usize },
}

pub struct Rewards<A, V, S> {
    pub name: Option<Identifier<S>>,
    pub entries: Vec<RewardsElement<A, V, S>>,
    pub span: S,
}

impl<A, V, S> Rewards<A, V, S> {
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
}

pub enum RewardsTarget<A> {
    State,
    Action(Option<A>),
}

pub struct RewardsElement<A, V, S> {
    pub condition: Expression<V, S>,
    pub value: Expression<V, S>,
    pub target: RewardsTarget<A>,
    pub span: S,
}

impl<A, V, S> RewardsElement<A, V, S> {
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
}
