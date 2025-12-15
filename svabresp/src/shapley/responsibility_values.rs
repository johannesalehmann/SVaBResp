use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{FromPrimitive, Zero};
use std::ops::Mul;

pub struct CriticalPairCounter {
    states: Vec<CriticalPairCounterState>,
}

impl CriticalPairCounter {
    pub fn new(state_count: usize) -> Self {
        let mut states = Vec::with_capacity(state_count);

        for i in 0..state_count {
            let counts = vec![0; state_count + 1];
            states.push(CriticalPairCounterState { counts });
        }

        CriticalPairCounter { states }
    }

    pub fn to_responsibility_values(self, weights: Vec<BigRational>) -> ResponsibilityValues {
        let mut states = Vec::with_capacity(self.states.len());

        for state in self.states {
            let mut value = BigRational::zero();
            for (n, &count) in state.counts.iter().enumerate() {
                value += &weights[n] * BigRational::from_usize(count).unwrap();
            }
            states.push(ResponsibilityValue {
                value,
                details: state,
            })
        }

        ResponsibilityValues { states }
    }

    pub fn increment(&mut self, state: usize, size: usize) {
        self.states[state].counts[size] += 1;
    }
}

pub struct CriticalPairCounterState {
    counts: Vec<usize>,
}

impl CriticalPairCounterState {
    pub fn to_responsibility_value(self, weights: Vec<BigRational>) -> ResponsibilityValue {
        let mut value = BigRational::zero();

        for (weight, &count) in weights.iter().zip(self.counts.iter()) {
            value += weight.clone().mul(BigRational::from_usize(count).unwrap());
        }

        ResponsibilityValue {
            value,
            details: self,
        }
    }
}

pub struct ResponsibilityValues {
    pub states: Vec<ResponsibilityValue>,
}

pub struct ResponsibilityValue {
    pub value: BigRational,
    pub details: CriticalPairCounterState,
}
