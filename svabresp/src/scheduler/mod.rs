use crate::grouping::StateIdx;
use prism_model_builder::To1;
use probabilistic_models::traits::{ReadChoiceLabels, ReadStateSpace, ReadValuations};
use probabilistic_models::valuations::{
    StandaloneValuation, Type, ValuationBitsMut, ValuationClass,
};
use probabilistic_models::{
    ChoiceIndex, DefaultIx, ValuationClassEntryIndex, ValuationClassIndex, ValuationIndex,
};
use std::collections::HashMap;

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

    pub fn parse<
        M: ReadStateSpace<Ix = DefaultIx>
            + ReadValuations<Ix = DefaultIx>
            + ReadChoiceLabels<E = Option<String>>,
    >(
        model: &M,
        file: String,
    ) -> Self {
        let mut res = Self::new(model);
        let lookup = model.generate_valuation_lookup();
        let class_index = model.valuation_classes().index(0);
        let class = model.valuation_class(class_index);
        for line in file.lines() {
            if line.trim() == "" {
                continue;
            }
            let (valuation, actions) = line.split_once(":").unwrap_or_else(|| {
                panic!("scheduler file line does not contain `:` (was: \"{line}\"")
            });
            let valuation = valuation.trim();
            let standalone = Self::parse_valuation(class_index, class, valuation);
            let state = lookup
                .get(&standalone)
                .unwrap_or_else(|| panic!("Could not find state for valuation `{}`", standalone));

            let mut action_to_prob = HashMap::new();
            for prob_action_pair in actions.split('+') {
                let prob_action_pair = prob_action_pair.trim();
                if let Some((prob, action)) = prob_action_pair.split_once(':') {
                    action_to_prob.insert(action.trim(), prob.trim().parse::<f64>().unwrap());
                } else {
                    action_to_prob.insert(prob_action_pair, 1.0);
                }
            }
            let probs = model
                .choices_of_state(state)
                .into_iter()
                .map(|c| {
                    model
                        .choice_label(c)
                        .as_ref()
                        .and_then(|l| action_to_prob.get(l.as_str()).cloned())
                        .unwrap_or(0.0)
                })
                .collect::<Vec<_>>();
            res.set_for_state(model, state, &probs);
        }
        res
    }

    fn parse_valuation<'a>(
        class_index: ValuationClassIndex<DefaultIx>,
        class: &'a ValuationClass<ValuationClassEntryIndex<DefaultIx>>,
        valuation: &str,
    ) -> StandaloneValuation<
        'a,
        ValuationClassIndex<DefaultIx>,
        ValuationClassEntryIndex<DefaultIx>,
        ValuationIndex<DefaultIx>,
    > {
        assert_eq!(
            valuation.chars().next(),
            Some('('),
            "scheduler file line contains invalid valuation (must start with `(`, was: \"{valuation}\")"
        );
        assert_eq!(
            valuation.chars().last(),
            Some(')'),
            "scheduler file line contains invalid valuation (must end with `)`, was: \"{valuation}\")"
        );
        let values = valuation[1..valuation.len() - 1]
            .split(',')
            .collect::<Vec<_>>();
        assert_eq!(
            values.len(),
            class.entries().len(),
            "scheduler file line contains {} values, but the model has {} variables (was: \"{valuation}\")",
            values.len(),
            class.entries().len()
        );
        let mut standalone = StandaloneValuation::new(class_index, class);
        for (value, (variable_index, variable)) in
            values.into_iter().zip(class.entries().enumerate())
        {
            let value = value.trim();
            match variable.variable_type {
                Type::Bool => {
                    standalone.set_bool(variable_index, value.parse::<bool>().unwrap());
                }
                Type::Int => {
                    standalone.set_int(variable_index, value.parse::<i64>().unwrap());
                }
                Type::Uint => {
                    // Uint variables can still support negative values with the right offset, so
                    // we take the int path here. This fails if the value fits into a u64, but not
                    // i64, but I suspect that rarely happens in practice.
                    standalone.set_int(variable_index, value.parse::<i64>().unwrap());
                }
                Type::Double => {
                    standalone.set_double(variable_index, value.parse::<f64>().unwrap());
                }
                Type::Rational => {
                    panic!(
                        "variables of type `Rational` are not yet supported by the scheduler parser"
                    )
                }
                Type::String => {
                    panic!(
                        "variables of type `String` are not yet supported by the scheduler parser"
                    )
                }
            }
        }
        standalone
    }

    pub fn set_for_state<M: ReadStateSpace<Ix = DefaultIx>>(
        &mut self,
        model: &M,
        state: StateIdx,
        distribution: &[f64],
    ) {
        let choices = model.choices_of_state(state);
        assert_eq!(
            distribution.len(),
            choices.len(),
            "distribution must have one entry per choice of the state"
        );
        if let Err(message) = check_distribution(distribution) {
            panic!("{message}");
        }

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

fn check_distribution(distribution: &[f64]) -> Result<(), String> {
    const DISTRIBUTION_SUM_TOLERANCE: f64 = 1e-9;

    if let Some(p) = distribution.iter().find(|&&p| p < 0.0 || p.is_nan()) {
        return Err(format!("probabilities must be non-negative, got {p}"));
    }
    let sum: f64 = distribution.iter().sum();
    if (sum - 1.0).abs() > DISTRIBUTION_SUM_TOLERANCE {
        return Err(format!(
            "the sum of probabilities must be 1 (it is {}={sum})",
            distribution
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join("+")
        ));
    }
    Ok(())
}
