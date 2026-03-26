#[derive(Clone, PartialEq)]
pub struct ModelFeatures {
    pub probabilism: bool,
    pub non_determinism: bool,
    pub ownership: Ownership,
}

#[derive(Clone, PartialEq)]
pub enum Ownership {
    SinglePlayer,
    TwoPlayer,
}

impl ModelFeatures {
    pub fn transition_system() -> Self {
        Self {
            probabilism: false,
            non_determinism: true,
            ownership: Ownership::SinglePlayer,
        }
    }

    pub fn nonstochastic_game() -> Self {
        Self {
            probabilism: false,
            non_determinism: true,
            ownership: Ownership::TwoPlayer,
        }
    }

    pub fn markov_chain() -> Self {
        Self {
            probabilism: true,
            non_determinism: false,
            ownership: Ownership::SinglePlayer,
        }
    }

    pub fn markov_decision_process() -> Self {
        Self {
            probabilism: true,
            non_determinism: true,
            ownership: Ownership::SinglePlayer,
        }
    }

    pub fn stochastic_game() -> Self {
        Self {
            probabilism: true,
            non_determinism: true,
            ownership: Ownership::TwoPlayer,
        }
    }

    pub fn from_model<M: super::ModelTypes>(model: &super::ProbabilisticModel<M>) -> Self {
        use crate::{ActionCollection, Distribution};

        let mut probabilism = false;
        let mut non_determinism = false;
        let ownership = match <M::Owners as crate::Owners>::max_player_count() {
            1 => Ownership::SinglePlayer,
            2 => Ownership::TwoPlayer,
            i => panic!("Cannot express model features of a model with {i} players"),
        };
        for state in &model.states {
            if !non_determinism {
                if state.actions.get_number_of_actions() > 1 {
                    non_determinism = true;
                }
            }

            if !probabilism {
                for action in state.actions.iter() {
                    if action.successors.number_of_successors() > 1 {
                        probabilism = true;
                        break;
                    }
                }
            }

            if non_determinism && probabilism {
                break;
            }
        }
        Self {
            probabilism,
            non_determinism,
            ownership,
        }
    }

    pub fn representable_as_transition_system(&self) -> bool {
        !self.probabilism && self.ownership == Ownership::SinglePlayer
    }

    pub fn representable_as_non_stochastic_game(&self) -> bool {
        !self.probabilism
    }

    pub fn representable_as_markov_chain(&self) -> bool {
        !self.non_determinism && self.ownership == Ownership::SinglePlayer
    }

    pub fn representable_as_markov_decision_process(&self) -> bool {
        self.ownership == Ownership::SinglePlayer
    }

    pub fn representable_as_stochastic_game(&self) -> bool {
        true
    }
}
