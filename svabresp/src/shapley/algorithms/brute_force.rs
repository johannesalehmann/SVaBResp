use crate::shapley::responsibility_values::{CriticalPairCounter, ResponsibilityValues};
use crate::shapley::{CooperativeGame, PlayerDescriptions, SimpleCooperativeGame};
use log::{info, trace};

pub struct BruteForceAlgorithm {}

impl BruteForceAlgorithm {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute_switching_pairs_simple<G: SimpleCooperativeGame, S: SwitchingPairCollector>(
        &mut self,
        mut game: G,
    ) -> (
        <BruteForceAlgorithm as super::super::ShapleyAlgorithm>::Output<
            <G::PlayerDescriptions as PlayerDescriptions>::PlayerType,
        >,
        S,
    ) {
        let n = game.get_player_count();
        trace!("Running brute-force algorithm for n={} groups", n);
        if n >= 64 {
            panic!(
                "The brute-force Shapley algorithm can only handle cooperative games with up to 63 players "
            )
        }

        let mut switching_pair_collector = S::initialise(n);

        let coalition_count = 1u64 << n;

        let mut counts = CriticalPairCounter::new(n);

        let start = std::time::Instant::now();
        let mut next_round_number = 10_000_000;
        for base_coalition in 0..coalition_count {
            if base_coalition == next_round_number && base_coalition > 0 {
                next_round_number += 10_000_000;
                if start.elapsed().as_secs_f32() > 5.0 {
                    info!(
                        "Checked {}m/{:.1}m ({:.2}%) switching pairs",
                        base_coalition / 1000_000,
                        coalition_count as f64 / 1000_000.0,
                        (base_coalition as f64 / coalition_count as f64) * 100.0
                    );
                }
            }
            if !game.is_winning(base_coalition) {
                let size = base_coalition.count_ones() as usize;
                for added_state in 0..n {
                    let coalition = base_coalition | 1 << added_state;
                    if coalition != base_coalition && game.is_winning(coalition) {
                        counts.increment(added_state, size + 1);
                        switching_pair_collector.add_pair(added_state, base_coalition);
                    }
                }
            }
        }

        let weights = super::super::auxiliary::compute_weights(n);
        (
            counts.to_responsibility_values(weights, game.into_player_descriptions()),
            switching_pair_collector,
        )
    }
}

impl super::super::ShapleyAlgorithm for BruteForceAlgorithm {
    type Output<PD> = ResponsibilityValues<PD>;

    fn compute<G: CooperativeGame>(
        &mut self,
        mut game: G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        let _ = &mut game;
        panic!("The brute force algorithm does not yet support non-simple cooperative games")
    }

    fn compute_simple<G: SimpleCooperativeGame>(
        &mut self,
        game: G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        let (responsibility, _) =
            self.compute_switching_pairs_simple::<_, DiscardingSwitchingPairCollector>(game);

        responsibility
    }
}

pub trait SwitchingPairCollector {
    fn initialise(player_count: usize) -> Self;
    fn add_pair(&mut self, state: usize, coalition: u64);
}

pub struct DiscardingSwitchingPairCollector {}

impl SwitchingPairCollector for DiscardingSwitchingPairCollector {
    fn initialise(player_count: usize) -> Self {
        let _ = player_count;
        Self {}
    }

    fn add_pair(&mut self, state: usize, coalition: u64) {
        let _ = (state, coalition);
    }
}

pub struct OnePairPerStateCollector {
    pairs: Vec<Option<u64>>,
}

impl OnePairPerStateCollector {
    pub fn get_coalition_for_player(&self, index: usize) -> Option<u64> {
        self.pairs[index]
    }
}

impl SwitchingPairCollector for OnePairPerStateCollector {
    fn initialise(player_count: usize) -> Self {
        Self {
            pairs: vec![None; player_count],
        }
    }

    fn add_pair(&mut self, state: usize, coalition: u64) {
        if self.pairs[state] == None {
            self.pairs[state] = Some(coalition);
        }
    }
}
