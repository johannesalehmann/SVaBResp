use crate::shapley::coop_game::PlayerDescriptions;
use crate::shapley::{CoalitionSpecifier, MonotoneCooperativeGame, SimpleCooperativeGame};
use log::trace;
use std::io::Write;

pub struct MinimalCoalitionCache<P: PlayerDescriptions> {
    player_descriptions: P,
    player_count: usize,
    minimal_coalitions: Vec<u64>,
}

// TODO: Remove PlayerType=String restriction (only used for debugging)
impl<P: PlayerDescriptions<PlayerType = String>> MinimalCoalitionCache<P> {
    pub fn large_losing_coalitions<
        C: SimpleCooperativeGame<PlayerDescriptions = P> + MonotoneCooperativeGame,
    >(
        coop_game: &mut C,
        n: usize,
    ) -> Vec<u64> {
        let full_coalition = (1u64 << coop_game.get_player_count()) - 1;

        let mut losing_coalitions = Vec::new();
        for size in 0..=n {
            for coalition in SizedCoalitionIterator::new(size, coop_game.get_player_count()) {
                let coalition = full_coalition ^ coalition;
                let mut superset_losing = false;
                for &other_coalition in &losing_coalitions {
                    if Self::subset_of(coalition, other_coalition) {
                        superset_losing = true;
                        break;
                    }
                }

                if !superset_losing && !coop_game.is_winning(coalition) {
                    losing_coalitions.push(coalition)
                }
            }
        }
        losing_coalitions
    }
    pub fn create<C: SimpleCooperativeGame<PlayerDescriptions = P> + MonotoneCooperativeGame>(
        mut coop_game: C,
    ) -> Self {
        trace!("Building minimal coalition cache");
        let mut minimal_coalitions = Vec::new();

        let max_coalition = 1u64 << coop_game.get_player_count();

        let mut game_counter = 0;
        let mut skipped_counter = 0;

        let large_losing_coalitions = Self::large_losing_coalitions(&mut coop_game, 4);

        std::io::stdout().flush().unwrap();
        for coalition in 0..max_coalition {
            if coalition % 10_000_000 == 0 && coalition > 0 {
                println!(
                    "{}/{} ({:.2}%)  (solved {} games, skipped {})",
                    coalition,
                    max_coalition,
                    (coalition as f64 / max_coalition as f64) * 100.0,
                    game_counter,
                    skipped_counter
                );
            }
            let mut subset_winning = false;
            for &other_coalition in &minimal_coalitions {
                if Self::subset_of(other_coalition, coalition) {
                    subset_winning = true;
                    break;
                }
            }
            let mut superset_losing = false;
            for &other_coalition in &large_losing_coalitions {
                if Self::subset_of(coalition, other_coalition) {
                    superset_losing = true;
                    break;
                }
            }

            if subset_winning || superset_losing {
                skipped_counter += 1;
            } else {
                game_counter += 1;
            }

            if !subset_winning && !superset_losing && coop_game.is_winning(coalition) {
                minimal_coalitions.push(coalition)
            }
        }

        Self {
            player_count: coop_game.get_player_count(),
            minimal_coalitions,
            player_descriptions: coop_game.into_player_descriptions(),
        }
    }

    fn subset_of(a: u64, b: u64) -> bool {
        (a | b) == b
    }
}

impl<P: PlayerDescriptions<PlayerType = String>> SimpleCooperativeGame
    for MinimalCoalitionCache<P>
{
    type PlayerDescriptions = P;

    fn get_player_count(&self) -> usize {
        self.player_count
    }

    fn player_descriptions(&self) -> &Self::PlayerDescriptions {
        &self.player_descriptions
    }

    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions {
        &mut self.player_descriptions
    }

    fn into_player_descriptions(self) -> Self::PlayerDescriptions {
        self.player_descriptions
    }

    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool {
        for &other_coalition in &self.minimal_coalitions {
            if Self::subset_of(other_coalition, coalition.to_mask()) {
                return true;
            }
        }
        false
    }
}

impl<P: PlayerDescriptions> MonotoneCooperativeGame for MinimalCoalitionCache<P> {}

struct SizedCoalitionIterator {
    ones: Vec<usize>,
    n: usize,
    done: bool,
}

impl SizedCoalitionIterator {
    pub fn new(count: usize, n: usize) -> Self {
        let mut ones = Vec::with_capacity(count);
        for i in 0..count {
            ones.push(i);
        }
        Self {
            ones,
            n,
            done: false,
        }
    }
}

impl Iterator for SizedCoalitionIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let mut res = 0;

            for one in &self.ones {
                res = res | 1u64 << one;
            }

            let mut incremented = false;
            for i in 0..self.ones.len() {
                if (i + 1 == self.ones.len() && self.ones[i] + 1 < self.n)
                    || (i + 1 < self.ones.len() && self.ones[i] + 1 < self.ones[i + 1])
                {
                    self.ones[i] += 1;
                    for j in 0..i {
                        self.ones[j] = j;
                    }
                    incremented = true;
                    break;
                }
            }

            if !incremented {
                self.done = true;
            }

            Some(res)
        }
    }
}
