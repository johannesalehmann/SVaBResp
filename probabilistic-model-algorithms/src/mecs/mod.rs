mod sub_mdp;
use sub_mdp::*;

use crate::sccs::StateToSccMap;
use probabilistic_models::{
    ActionCollection, ActionVector, Distribution, DistributionVector, ModelTypes, Predecessors,
    ProbabilisticModel,
};

pub fn compute_mecs<M: ModelTypes>(model: &ProbabilisticModel<M>) -> Mecs {
    let mut sub_mdp = SubMdp::new(model);
    loop {
        let sccs: StateToSccMap = super::sccs::compute_sccs(model, &sub_mdp);

        let mut changed = false;

        let mut to_process = Vec::new();
        for (state_index, (sub_mdp_state, model_state)) in sub_mdp
            .states
            .iter_mut()
            .zip(model.states.iter())
            .enumerate()
        {
            let scc_index = sccs.scc_index(state_index);
            if sub_mdp_state.enabled {
                for (enabled, action) in sub_mdp_state
                    .enabled_actions
                    .iter_mut()
                    .zip(model_state.actions.iter())
                {
                    if *enabled {
                        let leaving_scc = action
                            .successors
                            .iter()
                            .any(|suc| sccs.scc_index(suc.index) != scc_index);
                        if leaving_scc {
                            *enabled = false;
                            changed = true;
                        }
                    }
                }
                if !sub_mdp_state.any_action_enabled() {
                    sub_mdp_state.enabled = false;
                    to_process.push(state_index);
                }
            }
        }

        while let Some(state_index) = to_process.pop() {
            for predecessor in model.states[state_index].predecessors.iter() {
                let from_state = &mut sub_mdp.states[predecessor.from];
                if from_state.enabled {
                    from_state.enabled_actions[predecessor.action_index] = false;

                    if !from_state.any_action_enabled() {
                        from_state.enabled = false;
                        to_process.push(predecessor.from);
                    }
                }
            }
        }

        if !changed {
            return Mecs::from_sub_mdp_and_sccs(sub_mdp, sccs);
        }
    }
}

pub struct Mecs {
    state_info: Vec<Option<MecStateInfo>>,
    mec_count: usize,
    identified_mec_state_index: Vec<usize>,
}

#[derive(Clone)]
pub struct MecStateInfo {
    mec_index: usize,
    enabled_actions: Vec<bool>,
}

impl Mecs {
    pub fn from_sub_mdp_and_sccs(sub_mdp: SubMdp, sccs: StateToSccMap) -> Self {
        let mut state_info = vec![None; sub_mdp.states.len()];

        // For every SCC index, the compression map stores the index of the corresponding MEC. These
        // do not coincide, as only non-trivial SCCs have MECs.
        let mut scc_compression_map = Vec::with_capacity(sccs.scc_count());
        let mut mec_count = 0;
        for scc in 0..sccs.scc_count() {
            if sccs.is_trivial(scc) {
                scc_compression_map.push(None);
            } else {
                scc_compression_map.push(Some(mec_count));
                mec_count += 1;
            }
        }
        let mut first_mec_state = vec![None; mec_count];

        for (state_index, state) in sub_mdp.states.into_iter().enumerate() {
            if let Some(scc_index) = sccs.scc_index(state_index) {
                if let Some(mec_index) = scc_compression_map[scc_index] {
                    if first_mec_state[mec_index].is_none() {
                        first_mec_state[mec_index] = Some(state_index);
                    }
                    state_info[state_index] = Some(MecStateInfo {
                        mec_index,
                        enabled_actions: state.enabled_actions,
                    })
                }
            }
        }

        let first_mec_state = first_mec_state
            .into_iter()
            .map(|s| s.expect("MEC computation resulted  in a MEC with zero states, which indicates MEC computation contains a bug."))
            .collect();

        Self {
            state_info,
            mec_count,
            identified_mec_state_index: first_mec_state,
        }
    }

    pub fn collapse_mecs<
        M: ModelTypes<
                Distribution = DistributionVector,
                ActionCollection = ActionVector<DistributionVector>,
            >,
    >(
        &self,
        model: &mut ProbabilisticModel<M>,
    ) {
        let mut mec_actions = Vec::new();
        for _ in 0..self.mec_count {
            mec_actions.push(ActionVector::new())
        }

        for state_index in 0..model.states.len() {
            if let Some(mec) = &self.state_info[state_index] {
                let mut actions = Vec::new();
                std::mem::swap(
                    &mut actions,
                    model.states[state_index].actions.actions_mut(),
                );

                for mut action in actions {
                    for destination in action.successors.successors_mut() {
                        destination.index = self.target_index(destination.index);
                    }
                    mec_actions[mec.mec_index].add_action(action);
                }
            } else {
                for action in model.states[state_index].actions.actions_mut() {
                    for destination in action.successors.successors_mut() {
                        destination.index = self.target_index(destination.index);
                    }
                }
            }
        }

        for (mec_index, actions) in mec_actions.into_iter().enumerate() {
            let first_mec_state = &mut model.states[self.identified_mec_state_index[mec_index]];
            first_mec_state.actions = actions;
        }

        // TODO: Merge probabilities
        // TODO: De-duplicate transitions

        model.rebuild_predecessors();
    }

    fn target_index(&self, state_index: usize) -> usize {
        if let Some(mec_info) = &self.state_info[state_index] {
            self.identified_mec_state_index[mec_info.mec_index]
        } else {
            state_index
        }
    }

    pub fn identified_mec_state_index(&self, mec_index: usize) -> usize {
        self.identified_mec_state_index[mec_index]
    }

    pub fn len(&self) -> usize {
        self.mec_count
    }

    pub fn count_states_in_mecs(&self) -> usize {
        let mut count = 0;
        for state in &self.state_info {
            if let Some(_) = state {
                count += 1;
            }
        }
        count
    }

    pub fn mec_of_state(&self, state_index: usize) -> Option<usize> {
        self.state_info[state_index]
            .as_ref()
            .map(|info| info.mec_index)
    }

    pub fn enabled_actions(&self, state_index: usize) -> impl Iterator<Item = usize> {
        self.state_info[state_index]
            .as_ref()
            .expect("Cannot retrieve enabled actions for a state that is not enabled")
            .enabled_actions
            .iter()
            .enumerate()
            .filter(|(_, enabled)| **enabled)
            .map(|(i, _)| i)
    }
}
