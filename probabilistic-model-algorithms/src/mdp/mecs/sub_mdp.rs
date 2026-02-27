use probabilistic_models::{ActionCollection, ModelTypes, ProbabilisticModel};
use std::fmt::Formatter;

pub struct SubMdp {
    pub states: Vec<SubMdpState>,
}

impl SubMdp {
    pub fn new<M: ModelTypes>(model: &ProbabilisticModel<M>) -> Self {
        let mut states = Vec::new();
        for state in &model.states {
            states.push(SubMdpState {
                enabled: true,
                enabled_actions: vec![true; state.actions.get_number_of_actions()],
            })
        }
        Self { states }
    }
}

impl std::fmt::Debug for SubMdp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.states.iter().enumerate().map(|(i, s)| (i, s)))
            .finish()
    }
}

pub struct SubMdpState {
    pub enabled: bool,
    pub enabled_actions: Vec<bool>,
}

impl SubMdpState {
    pub fn any_action_enabled(&self) -> bool {
        self.enabled_actions.iter().any(|a| *a)
    }
}

impl std::fmt::Debug for SubMdpState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubMdpState")
            .field("enabled", &self.enabled)
            .field(
                "enabled_actions",
                &self
                    .enabled_actions
                    .iter()
                    .enumerate()
                    .filter(|(_, enabled)| **enabled)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl super::super::sccs::ExclusionCriterion for SubMdp {
    type Iterator<'a>
        = super::super::sccs::ExclusionCriterionIterator<'a, Self>
    where
        Self: 'a;

    fn iter_states<'a>(&'a self) -> Self::Iterator<'a> {
        self.automatic_iter_states(self.states.len())
    }

    fn is_state_excluded(&self, index: usize) -> bool {
        !self.states[index].enabled
    }

    fn is_action_excluded(&self, state_index: usize, action_index: usize) -> bool {
        !self.states[state_index].enabled || !self.states[state_index].enabled_actions[action_index]
    }
}
