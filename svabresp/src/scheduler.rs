use crate::grouping::StateIdx;
use prism_model_builder::To1;
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::{ChoiceIndex, DefaultIx};

pub type ChoiceIdx = ChoiceIndex<DefaultIx>;

/// A positional randomised scheduler that is only defined on some states.
pub struct PartialScheduler {
    defined: To1<StateIdx, bool>,
    /// NaN for the choices of undefined states, so that reading them by mistake poisons the result.
    probabilities: To1<ChoiceIdx, f64>,
}

impl PartialScheduler {
    pub fn new<M: ReadStateSpace<Ix = DefaultIx>>(model: &M) -> Self {
        Self {
            defined: To1::with_entries(vec![false; model.states().len()]),
            probabilities: To1::with_entries(vec![f64::NAN; model.choices().len()]),
        }
    }

    pub fn set_for_state<M: ReadStateSpace<Ix = DefaultIx>>(
        &mut self,
        model: &M,
        state: StateIdx,
        distribution: &[f64],
    ) {
        const DISTRIBUTION_SUM_TOLERANCE: f64 = 1e-9;

        let choices = model.choices_of_state(state);
        assert_eq!(
            distribution.len(),
            choices.len(),
            "distribution must have one entry per choice of the state"
        );
        assert!(
            distribution.iter().all(|&p| p >= 0.0),
            "probabilities must be non-negative: {distribution:?}"
        );
        let sum: f64 = distribution.iter().sum();
        assert!(
            (sum - 1.0).abs() <= DISTRIBUTION_SUM_TOLERANCE,
            "the sum of probabilities must be 1 (it is {}={sum})",
            distribution
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join("+")
        );

        self.probabilities[choices.start..choices.end].copy_from_slice(distribution);
        self.defined[state] = true;
    }

    pub fn is_defined(&self, state: StateIdx) -> bool {
        self.defined[state]
    }

    /// NaN if the scheduler is undefined on the state of `choice`.
    pub fn probability(&self, choice: ChoiceIdx) -> f64 {
        self.probabilities[choice]
    }
}
