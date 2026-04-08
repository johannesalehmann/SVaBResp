use crate::shapley::PlayerDescriptions;
use log::trace;
use num_rational::BigRational;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use std::ops::Mul;

pub struct CriticalPairCounter<V: Default> {
    states: Vec<CriticalPairCounterState<V>>,
}

impl<V: Default> CriticalPairCounter<V> {
    pub fn new(state_count: usize) -> Self {
        let mut states = Vec::with_capacity(state_count);

        for _ in 0..state_count {
            let mut counts = Vec::with_capacity(state_count + 1);
            for _ in 0..=state_count {
                counts.push(V::default());
            }
            states.push(CriticalPairCounterState { counts });
        }

        CriticalPairCounter { states }
    }

    pub fn map_counts<V2: Default, F: Fn(V) -> V2>(self, map: F) -> CriticalPairCounter<V2> {
        CriticalPairCounter {
            states: self
                .states
                .into_iter()
                .map(|s| CriticalPairCounterState {
                    counts: s.counts.into_iter().map(|v| map(v)).collect(),
                })
                .collect(),
        }
    }
}

impl CriticalPairCounter<usize> {
    #[allow(unused)] // TODO: Properly support both integer and floating-point critical pair counting
    pub fn to_responsibility_values<P: PlayerDescriptions>(
        self,
        weights: Vec<BigRational>,
        player_infos: P,
    ) -> ResponsibilityValues<P::PlayerType, BigRational, usize> {
        trace!("Transforming counts into responsibility values");
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

impl CriticalPairCounter<f64> {
    pub fn to_responsibility_values<P: PlayerDescriptions>(
        self,
        weights: Vec<BigRational>,
        player_infos: P,
    ) -> ResponsibilityValues<P::PlayerType, f64, f64> {
        trace!("Transforming counts into responsibility values");
        let mut states = Vec::with_capacity(self.states.len());

        for (state, player_info) in self.states.into_iter().zip(player_infos.into_iterator()) {
            states.push(state.to_responsibility_value(player_info, &weights));
        }

        ResponsibilityValues { players: states }
    }

    pub fn increase_by(&mut self, state: usize, size: usize, amount: f64) {
        self.states[state].counts[size] += amount;
    }
}

#[derive(Debug)]
pub struct CriticalPairCounterState<V> {
    counts: Vec<V>,
}

impl CriticalPairCounterState<usize> {
    pub fn to_responsibility_value<P>(
        self,
        player_info: P,
        weights: &Vec<BigRational>,
    ) -> ResponsibilityValue<P, BigRational, usize> {
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

impl CriticalPairCounterState<f64> {
    pub fn to_responsibility_value<P>(
        self,
        player_info: P,
        weights: &Vec<BigRational>,
    ) -> ResponsibilityValue<P, f64, f64> {
        let mut value = 0.0;

        for (weight, &count) in weights.iter().zip(self.counts.iter()) {
            value += weight.to_f64().unwrap() * count;
        }

        ResponsibilityValue {
            player_info,
            value,
            details: self,
        }
    }
}

#[derive(Debug)]
pub struct ResponsibilityValues<P, V, VD> {
    pub players: Vec<ResponsibilityValue<P, V, VD>>,
}

impl<P, V, VD> ResponsibilityValues<P, V, VD>
where
    for<'a> &'a P: PartialEq,
{
    pub fn get(&self, index: &P) -> Option<&ResponsibilityValue<P, V, VD>> {
        for (i, value) in self.players.iter().enumerate() {
            if &value.player_info == index {
                return Some(&self.players[i]);
            }
        }
        None
    }
}
impl<P, V, VD> ResponsibilityValues<P, V, VD> {
    pub fn map_player_info<P2, F: FnMut(P) -> P2>(
        self,
        mut map: F,
    ) -> ResponsibilityValues<P2, V, VD> {
        ResponsibilityValues {
            players: self
                .players
                .into_iter()
                .map(|p| ResponsibilityValue {
                    player_info: map(p.player_info),
                    value: p.value,
                    details: p.details,
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct ResponsibilityValue<P, V, VD> {
    pub player_info: P,
    pub value: V,
    pub details: CriticalPairCounterState<VD>,
}
