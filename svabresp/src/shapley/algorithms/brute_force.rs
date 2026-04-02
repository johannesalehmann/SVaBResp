use crate::shapley::responsibility_values::{CriticalPairCounter, ResponsibilityValues};
use crate::shapley::{CooperativeGame, PlayerDescriptions, SimpleCooperativeGame};
use log::{info, trace};
use num_traits::ToPrimitive;

pub struct BruteForceAlgorithm {}

impl BruteForceAlgorithm {
    pub fn new() -> Self {
        Self {}
    }

    fn get_n_and_coalition_count<G: CooperativeGame>(&self, game: &G) -> (usize, u64) {
        let n = game.get_player_count();
        trace!("Running brute-force algorithm for n={} groups", n);
        if n >= 64 {
            panic!(
                "The brute-force Shapley algorithm can only handle cooperative games with up to 63 players "
            )
        }
        let coalition_count = 1u64 << n;
        (n, coalition_count)
    }
}

impl super::super::ShapleyAlgorithm for BruteForceAlgorithm {
    type Output<PD> = ResponsibilityValues<PD, f64, f64>;

    fn compute_with_switching_pairs<
        G: CooperativeGame,
        SPC: crate::shapley::SwitchingPairCollector,
    >(
        &mut self,
        mut game: G,
        switching_pair_collector: &mut SPC,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        let (n, coalition_count) = self.get_n_and_coalition_count(&game);

        let mut counts = CriticalPairCounter::new(n);

        let weights = super::super::auxiliary::compute_weights(n);
        let weights_float = weights
            .iter()
            .map(|w| w.to_f64().unwrap())
            .collect::<Vec<_>>();

        let start = std::time::Instant::now();
        let mut next_round_number = 1_000_000;
        for base_coalition in 0..coalition_count {
            // TODO: Factor out status reporting for both simple and non-simple algorithm
            if base_coalition == next_round_number && base_coalition > 0 {
                next_round_number += 1_000_000;
                if start.elapsed().as_secs_f32() > 5.0 {
                    info!(
                        "Checked {}m/{:.1}m ({:.2}%) switching pairs",
                        base_coalition / 1000_000,
                        coalition_count as f64 / 1000_000.0,
                        (base_coalition as f64 / coalition_count as f64) * 100.0
                    );
                }
            }
            let base_value = game.get_value(base_coalition);
            let size = base_coalition.count_ones() as usize;
            for added_state in 0..n {
                let coalition = base_coalition | 1 << added_state;
                if coalition != base_coalition {
                    let coalition_value = game.get_value(coalition);
                    counts.increase_by(added_state, size + 1, coalition_value - base_value);
                    if coalition_value > base_value {
                        let pair_value = coalition_value - base_value;
                        switching_pair_collector.register_switching_pair(
                            added_state,
                            base_coalition,
                            pair_value,
                            pair_value * weights_float[size + 1],
                        );
                    }
                }
            }
        }

        counts.to_responsibility_values(weights, game.into_player_descriptions())
    }

    fn compute_simple_with_switching_pairs<
        G: SimpleCooperativeGame,
        SPC: crate::shapley::SwitchingPairCollector,
    >(
        &mut self,
        mut game: G,
        switching_pair_collector: &mut SPC,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        let (n, coalition_count) = self.get_n_and_coalition_count(&game);

        let mut counts = CriticalPairCounter::new(n);

        let weights = super::super::auxiliary::compute_weights(n);
        let weights_float = weights
            .iter()
            .map(|w| w.to_f64().unwrap())
            .collect::<Vec<_>>();

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
                        switching_pair_collector.register_switching_pair(
                            added_state,
                            base_coalition,
                            1.0,
                            weights_float[size + 1],
                        );
                    }
                }
            }
        }

        counts
            .map_counts(|c| c as f64)
            .to_responsibility_values(weights, game.into_player_descriptions())
    }
}
