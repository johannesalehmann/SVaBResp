mod algorithms;

pub use algorithms::*;
use std::collections::HashMap;
use std::fmt::Display;

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
        value_without: f64, // The value of the coalition working alone
        value_with: f64,    // The value of the coalition and state working together
        contribution: f64,  // How much this switching pair adds to the coalition
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
        value_without: f64,
        value_with: f64,
        contribution: f64,
    ) {
        let _ = (state, coalition, value_without, value_with, contribution);
    }
}

#[derive(Clone)]
pub struct SwitchingPair<C: CoalitionSpecifier> {
    pub coalition: C,
    pub value_without: f64,
    pub value_with: f64,
    pub contribution: f64,
}

impl<C: CoalitionSpecifier> SwitchingPair<C> {
    pub fn value(&self) -> f64 {
        self.value_with - self.value_without
    }
}

pub struct FullSwitchingPairCollector {
    switching_pairs: HashMap<usize, Vec<SwitchingPair<u64>>>,
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
        value_without: f64,
        value_with: f64,
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
            value_without,
            value_with,
            contribution,
        })
    }
}

pub struct OnePairPerStateCollector {
    pairs: HashMap<usize, SwitchingPair<u64>>,
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
        value_without: f64,
        value_with: f64,
        contribution: f64,
    ) {
        if !self.pairs.contains_key(&state) {
            self.pairs.insert(
                state,
                SwitchingPair {
                    coalition,
                    value_without,
                    value_with,
                    contribution,
                },
            );
        }
    }
}

#[derive(Clone)]
pub struct SwitchingPairCollection {
    switching_pairs: HashMap<usize, Vec<SwitchingPair<u64>>>,
}

impl SwitchingPairCollection {
    pub fn switching_pairs(&self, state_index: usize) -> &[SwitchingPair<u64>] {
        if let Some(list) = self.switching_pairs.get(&state_index) {
            &list[..]
        } else {
            &[]
        }
    }

    pub fn aggregate_by_minimal_switching_pair(self) -> AggregatedSwitchingPairCollection<u64> {
        let mut res = AggregatedSwitchingPairCollection::new();

        for (index, switching_pairs) in &self.switching_pairs {
            let mut aggregated_pairs = Vec::new();

            let mut non_minimal_pairs = Vec::new();

            for (i, switching_pair) in switching_pairs.iter().enumerate() {
                let mut is_minimal = true;
                for (j, other_pair) in switching_pairs.iter().enumerate() {
                    if i != j {
                        // Check whether other_pair.coalition is a subset of pair.coalition
                        if switching_pair.coalition.to_mask() | other_pair.coalition.to_mask() // TODO: Add a function to CoalitionSpecifier to check subset inclusion
                            == switching_pair.coalition.to_mask()
                        {
                            is_minimal = false;
                            break;
                        }
                    }
                }
                if is_minimal {
                    aggregated_pairs.push(AggregatedSwitchingPair::from_switching_pair(
                        switching_pair.clone(),
                    ));
                } else {
                    non_minimal_pairs.push(i);
                }
            }

            for i in non_minimal_pairs {
                let mut contained_pairs = Vec::new();
                let pair = &switching_pairs[i];
                for (j, other_pair) in aggregated_pairs.iter().enumerate() {
                    if i != j {
                        // Check whether other_pair.coalition is a subset of pair.coalition
                        if pair.coalition.to_mask() | other_pair.coalition.to_mask()
                            == pair.coalition.to_mask()
                        {
                            contained_pairs.push(j);
                        }
                    }
                }

                assert!(
                    !contained_pairs.is_empty(),
                    "Found a switching pair that is neither minimal nor contains a minimal pair."
                );

                let average_contribution = pair.contribution / contained_pairs.len() as f64;
                for j in contained_pairs {
                    aggregated_pairs[j].aggregated_pair_count += 1;
                    aggregated_pairs[j].indirect_contribution += average_contribution;
                }
            }

            res.switching_pairs.insert(*index, aggregated_pairs);
        }

        res
    }

    pub fn contains_non_simple_pairs(&self) -> bool {
        for pairs in self.switching_pairs.values() {
            for pair in pairs {
                if pair.value_with != 1.0 || pair.value_without != 0.0 {
                    return true;
                }
            }
        }
        false
    }
}

pub struct AggregatedSwitchingPairCollection<C: CoalitionSpecifier> {
    switching_pairs: HashMap<usize, Vec<AggregatedSwitchingPair<C>>>,
}

impl<C: CoalitionSpecifier> AggregatedSwitchingPairCollection<C> {
    pub fn new() -> Self {
        Self {
            switching_pairs: HashMap::new(),
        }
    }
    pub fn switching_pairs(&self, state_index: usize) -> &[AggregatedSwitchingPair<C>] {
        if let Some(list) = self.switching_pairs.get(&state_index) {
            &list[..]
        } else {
            &[]
        }
    }

    pub fn value_and_tool_tip_text<S1: Display, S2: AsRef<str>, P: PartialEq + Display, VD>(
        &self,
        responsibility_name: S1,
        colour_ramp_index: usize,
        group_name: &P,
        values: &ResponsibilityValues<P, f64, VD>,
        player_names: &[S2],
        is_probabilistic: bool,
        only_switching_pairs: bool,
    ) -> (f64, String) {
        fn round_float(value: f64) -> String {
            format!("{:.3}", value)
                .trim_end_matches("0")
                .trim_end_matches(".")
                .to_string()
        }

        let value = if let Some(responsibility) = values.get(group_name) {
            responsibility.value
        } else {
            0.0
        };

        let tooltip_start = format!(
            "{responsibility_name} responsibility for `{group_name}`: <ColoredNumber>{value}, {colour_ramp_index}</ColoredNumber>"
        );

        let mut tooltip_text = Vec::new();

        if !only_switching_pairs {
            tooltip_text.push(tooltip_start);
        }

        let player_index = values.get_index(group_name);
        if let Some(player_index) = player_index {
            let switching_pairs = self.switching_pairs(player_index);
            if switching_pairs.len() > 0 {
                if !only_switching_pairs {
                    tooltip_text.push("\n\n## Switching pairs\n".to_string());
                }
            }

            let mut switching_pairs = switching_pairs.iter().collect::<Vec<_>>();
            switching_pairs.sort_unstable_by(|sp1, sp2| {
                sp2.direct_contribution
                    .partial_cmp(&sp1.direct_contribution)
                    .expect("Encountered NaN while sorting (aggregated) switching pairs")
            });

            let mut first = only_switching_pairs;

            for switching_pair in switching_pairs {
                if !first {
                    tooltip_text.push("\n".to_string());
                }
                first = false;
                tooltip_text.push("- ".to_string());
                tooltip_text.push(CoalitionSpecifier::to_string(
                    &switching_pair.coalition,
                    player_names,
                ));
                tooltip_text.push(format!(
                    ": <ColoredNumber>{}, {colour_ramp_index}</ColoredNumber>",
                    switching_pair.direct_contribution
                ));
                if switching_pair.indirect_contribution > 0.0 {
                    let superset_pair_text = if switching_pair.aggregated_pair_count == 1 {
                        "superset"
                    } else {
                        "supersets"
                    };
                    tooltip_text.push(format!(
                        " <grey>(+ <ColoredNumber>{},{}</ColoredNumber> from {} {})</grey>",
                        switching_pair.indirect_contribution,
                        colour_ramp_index,
                        switching_pair.aggregated_pair_count,
                        superset_pair_text,
                    ));
                }
                if is_probabilistic {
                    tooltip_text.push(format!(
                        "\n\n    <grey>Value: {} - {} = {}</grey>",
                        round_float(switching_pair.value_with),
                        round_float(switching_pair.value_without),
                        round_float(switching_pair.value()),
                    ));
                }
            }
        }

        (value, tooltip_text.join(""))
    }
}

pub struct AggregatedSwitchingPair<C: CoalitionSpecifier> {
    pub coalition: C,
    pub value_without: f64,
    pub value_with: f64,
    pub direct_contribution: f64,
    pub indirect_contribution: f64,
    pub aggregated_pair_count: usize,
}

impl<C: CoalitionSpecifier> AggregatedSwitchingPair<C> {
    pub fn new(
        coalition: C,
        value_without: f64,
        value_with: f64,
        direct_contribution: f64,
    ) -> Self {
        Self {
            coalition,
            value_without,
            value_with,
            direct_contribution,
            indirect_contribution: 0.0,
            aggregated_pair_count: 0,
        }
    }

    pub fn from_switching_pair(switching_pair: SwitchingPair<C>) -> Self {
        Self::new(
            switching_pair.coalition,
            switching_pair.value_without,
            switching_pair.value_with,
            switching_pair.contribution,
        )
    }

    pub fn contribution(&self) -> f64 {
        self.direct_contribution + self.indirect_contribution
    }

    pub fn value(&self) -> f64 {
        self.value_with + self.value_without
    }
}

pub trait ShapleyAlgorithm {
    type Output<PD>;

    fn compute<G: CooperativeGame>(
        &mut self,
        game: &mut G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        self.compute_with_switching_pairs(game, &mut DiscardingSwitchingPairCollector {})
    }

    fn compute_with_switching_pairs<G: CooperativeGame, SPC: SwitchingPairCollector>(
        &mut self,
        game: &mut G,
        switching_pair_collector: &mut SPC,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType>;

    fn compute_simple<G: SimpleCooperativeGame>(
        &mut self,
        game: &mut G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        self.compute_simple_with_switching_pairs(game, &mut DiscardingSwitchingPairCollector {})
    }
    fn compute_simple_with_switching_pairs<G: SimpleCooperativeGame, SPC: SwitchingPairCollector>(
        &mut self,
        game: &mut G,
        switching_pair_collector: &mut SPC,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType>;
}
