use crate::shapley::PlayerDescriptions;
use num_rational::BigRational;
use num_traits::{FromPrimitive, Zero};
use std::ops::Mul;

pub struct CriticalPairCounter {
    states: Vec<CriticalPairCounterState>,
}

impl CriticalPairCounter {
    pub fn new(state_count: usize) -> Self {
        let mut states = Vec::with_capacity(state_count);

        for _ in 0..state_count {
            let counts = vec![0; state_count + 1];
            states.push(CriticalPairCounterState { counts });
        }

        CriticalPairCounter { states }
    }

    pub fn to_responsibility_values<P: PlayerDescriptions>(
        self,
        weights: Vec<BigRational>,
        player_infos: P,
    ) -> ResponsibilityValues<P::PlayerType> {
        let mut states = Vec::with_capacity(self.states.len());

        for (state, player_info) in self.states.into_iter().zip(player_infos.into_iterator()) {
            states.push(state.to_responsibility_value(player_info, &weights));
        }

        ResponsibilityValues { players: states }
    }

    pub fn increment(&mut self, state: usize, size: usize) {
        self.states[state].counts[size] += 1;
    }
}

pub struct CriticalPairCounterState {
    counts: Vec<usize>,
}

impl CriticalPairCounterState {
    pub fn to_responsibility_value<P>(
        self,
        player_info: P,
        weights: &Vec<BigRational>,
    ) -> ResponsibilityValue<P> {
        let mut value = BigRational::zero();

        for (weight, &count) in weights.iter().zip(self.counts.iter()) {
            value += weight.clone().mul(BigRational::from_usize(count).unwrap());
        }

        ResponsibilityValue {
            player_info,
            value,
            details: self,
        }
    }
}

pub struct ResponsibilityValues<P> {
    pub players: Vec<ResponsibilityValue<P>>,
}

impl<P> ResponsibilityValues<P>
where
    for<'a> &'a P: PartialEq,
{
    pub fn get(&self, index: &P) -> Option<&ResponsibilityValue<P>> {
        for (i, value) in self.players.iter().enumerate() {
            if &value.player_info == index {
                return Some(&self.players[i]);
            }
        }
        None
    }
}

pub struct ResponsibilityValue<P> {
    pub player_info: P,
    pub value: BigRational,
    pub details: CriticalPairCounterState,
}
