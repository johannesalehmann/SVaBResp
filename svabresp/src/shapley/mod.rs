mod algorithms;

pub use algorithms::*;
use std::collections::HashMap;

mod auxiliary;

mod coop_game;
pub use coop_game::{
    CoalitionSpecifier, CooperativeGame, MinimalCoalitionCache, MonotoneCooperativeGame,
    PlayerDescriptions, SimpleCooperativeGame,
};

mod responsibility_values;
pub use responsibility_values::{ResponsibilityValue, ResponsibilityValues};

pub trait SwitchingPairCollector {
    fn register_switching_pair(
        &mut self,
        state: usize,
        coalition: u64,
        pair_value: f64, // How much the game value increases by adding `state` to the coalition.
        contribution: f64, // How much this switching pair adds to the coalition
    );
}

pub struct DiscardingSwitchingPairCollector {}

impl DiscardingSwitchingPairCollector {
    pub fn new() -> Self {
        Self {}
    }
}

impl SwitchingPairCollector for DiscardingSwitchingPairCollector {
    fn register_switching_pair(
        &mut self,
        state: usize,
        coalition: u64,
        pair_value: f64,
        contribution: f64,
    ) {
        let _ = (state, coalition, pair_value, contribution);
    }
}

pub struct SwitchingPair {
    pub coalition: u64,
    pub pair_value: f64,
    pub contribution: f64,
}
pub struct FullSwitchingPairCollector {
    switching_pairs: HashMap<usize, Vec<SwitchingPair>>,
}

impl FullSwitchingPairCollector {
    pub fn new() -> Self {
        Self {
            switching_pairs: HashMap::new(),
        }
    }

    pub fn into_switching_pair_collection(self) -> SwitchingPairCollection {
        SwitchingPairCollection {
            switching_pairs: self.switching_pairs,
        }
    }
}

impl SwitchingPairCollector for FullSwitchingPairCollector {
    fn register_switching_pair(
        &mut self,
        state: usize,
        coalition: u64,
        pair_value: f64,
        contribution: f64,
    ) {
        let list = if let Some(list) = self.switching_pairs.get_mut(&state) {
            list
        } else {
            self.switching_pairs.insert(state, Vec::new());
            self.switching_pairs.get_mut(&state).unwrap()
        };
        list.push(SwitchingPair {
            coalition,
            pair_value,
            contribution,
        })
    }
}

pub struct OnePairPerStateCollector {
    pairs: HashMap<usize, SwitchingPair>,
}

impl OnePairPerStateCollector {
    pub fn get_coalition_for_player(&self, index: usize) -> Option<u64> {
        self.pairs.get(&index).map(|p| p.coalition)
    }
}

impl SwitchingPairCollector for OnePairPerStateCollector {
    fn register_switching_pair(
        &mut self,
        state: usize,
        coalition: u64,
        pair_value: f64,
        contribution: f64,
    ) {
        if !self.pairs.contains_key(&state) {
            self.pairs.insert(
                state,
                SwitchingPair {
                    coalition,
                    pair_value,
                    contribution,
                },
            );
        }
    }
}

pub struct SwitchingPairCollection {
    switching_pairs: HashMap<usize, Vec<SwitchingPair>>,
}

impl SwitchingPairCollection {
    pub fn switching_pairs(&self, state_index: usize) -> &[SwitchingPair] {
        if let Some(list) = self.switching_pairs.get(&state_index) {
            &list[..]
        } else {
            &[]
        }
    }
}

pub trait ShapleyAlgorithm {
    type Output<PD>;

    fn compute<G: CooperativeGame>(
        &mut self,
        game: G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        self.compute_with_switching_pairs(game, &mut DiscardingSwitchingPairCollector {})
    }

    fn compute_with_switching_pairs<G: CooperativeGame, SPC: SwitchingPairCollector>(
        &mut self,
        game: G,
        switching_pair_collector: &mut SPC,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType>;

    fn compute_simple<G: SimpleCooperativeGame>(
        &mut self,
        game: G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        self.compute_simple_with_switching_pairs(game, &mut DiscardingSwitchingPairCollector {})
    }
    fn compute_simple_with_switching_pairs<G: SimpleCooperativeGame, SPC: SwitchingPairCollector>(
        &mut self,
        game: G,
        switching_pair_collector: &mut SPC,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType>;
}
