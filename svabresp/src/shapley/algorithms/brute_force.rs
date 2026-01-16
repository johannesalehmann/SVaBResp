use crate::shapley::responsibility_values::{CriticalPairCounter, ResponsibilityValues};
use crate::shapley::{CooperativeGame, PlayerDescriptions, SimpleCooperativeGame};

pub struct BruteForceAlgorithm {}

impl BruteForceAlgorithm {
    pub fn new() -> Self {
        Self {}
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
        mut game: G,
    ) -> Self::Output<<G::PlayerDescriptions as PlayerDescriptions>::PlayerType> {
        let n = game.get_player_count();
        println!("Computing responsibility for n={} groups", n);
        if n >= 64 {
            panic!(
                "The brute-force Shapley algorithm can only handle cooperative games with up to 63 players "
            )
        }

        let coalition_count = 1u64 << n;

        let mut counts = CriticalPairCounter::new(n);

        let start = std::time::Instant::now();
        for base_coalition in 0..coalition_count {
            if base_coalition % 20_000_000 == 0
                && base_coalition > 0
                && start.elapsed().as_secs_f32() > 2.0
            {
                println!(
                    "{}k/{}k ({} games per second)",
                    base_coalition / 1000,
                    coalition_count / 1000,
                    base_coalition as f32 / start.elapsed().as_secs_f32()
                );
            }
            let size = base_coalition.count_ones() as usize;
            if !game.is_winning(base_coalition) {
                for added_state in 0..n {
                    let coalition = base_coalition | 1 << added_state;
                    if coalition != base_coalition && game.is_winning(coalition) {
                        counts.increment(added_state, size + 1);
                    }
                }
            }
        }

        let weights = super::super::auxiliary::compute_weights(n);
        counts.to_responsibility_values(weights, game.into_player_descriptions())
    }
}
