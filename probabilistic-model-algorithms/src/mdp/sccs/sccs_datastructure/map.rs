use probabilistic_models::{ModelTypes, ProbabilisticModel};
use std::fmt::Formatter;

pub struct StateToSccMap {
    state_to_scc: Vec<usize>,
    scc_count: usize,
    is_trivial: Vec<bool>,
}

impl StateToSccMap {
    pub fn scc_index(&self, state_index: usize) -> Option<usize> {
        let value = self.state_to_scc[state_index];
        if value == usize::MAX {
            None
        } else {
            Some(self.state_to_scc[state_index])
        }
    }

    pub fn scc_count(&self) -> usize {
        self.scc_count
    }

    pub fn is_trivial(&self, state_index: usize) -> bool {
        self.is_trivial[state_index]
    }
}

impl std::fmt::Debug for StateToSccMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "StateToSccMap {{")?;
        writeln!(f, "  scc_count: {}", self.scc_count)?;
        writeln!(f, "  is_trivial: {{")?;
        for (i, &trivial) in self.is_trivial.iter().enumerate() {
            writeln!(
                f,
                "    {} is {}trivial",
                i,
                if trivial { "" } else { "not " }
            )?;
        }
        writeln!(f, "  }},")?;
        writeln!(f, "  state_to_scc: {{")?;
        for (i, &state) in self.state_to_scc.iter().enumerate() {
            writeln!(f, "    {} -> {}", i, state)?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "}}")
    }
}

pub struct StateToSccMapBuilder {
    result: StateToSccMap,
}

impl super::BuildableScc for StateToSccMap {
    type BuilderType = StateToSccMapBuilder;

    fn builder<M: ModelTypes>(model: &ProbabilisticModel<M>) -> Self::BuilderType {
        StateToSccMapBuilder {
            result: Self {
                state_to_scc: vec![usize::MAX; model.states.len()],
                scc_count: 0,
                is_trivial: Vec::new(),
            },
        }
    }
}

impl super::SccBuilder<StateToSccMap> for StateToSccMapBuilder {
    fn add_scc(&mut self) -> usize {
        let value = self.result.scc_count;
        self.result.scc_count += 1;
        self.result.is_trivial.push(true);
        value
    }

    fn add_to_scc(&mut self, state_index: usize, scc_index: usize) {
        self.result.state_to_scc[state_index] = scc_index;
    }

    fn mark_non_trivial(&mut self, scc_index: usize) {
        self.result.is_trivial[scc_index] = false;
    }

    fn finish(self) -> StateToSccMap {
        self.result
    }
}
