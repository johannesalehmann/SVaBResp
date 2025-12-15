use crate::shapley::{
    CoalitionSpecifier, CooperativeGame, MonotoneCooperativeGame, SimpleCooperativeGame,
};

pub struct MinimalCoalitionCache {
    player_count: usize,
    minimal_coalitions: Vec<u64>,
}

impl MinimalCoalitionCache {
    pub fn create<C: SimpleCooperativeGame + MonotoneCooperativeGame>(mut coop_game: C) -> Self {
        let mut minimal_coalitions = Vec::new();

        let max_coalition = 1u64 << coop_game.get_player_count();

        let mut game_counter = 0;
        let mut skipped_counter = 0;

        for size in 0..=coop_game.get_player_count() as u32 {
            println!("Size {}/{}", size, coop_game.get_player_count());
            for coalition in 0..max_coalition {
                if coalition.count_ones() == size {
                    let mut subset_winning = false;
                    for &other_coalition in &minimal_coalitions {
                        if Self::subset_of(other_coalition, coalition) {
                            subset_winning = true;
                            break;
                        }
                    }

                    if subset_winning {
                        skipped_counter += 1;
                    } else {
                        game_counter += 1;
                    }

                    if !subset_winning && coop_game.is_winning(coalition) {
                        minimal_coalitions.push(coalition)
                    }
                }
            }
            println!(
                "Solved {} games (skipped {})",
                game_counter, skipped_counter
            );
            game_counter = 0;
            skipped_counter = 0;
        }

        println!(
            "Minimal coalition cache contains {} coalitions",
            minimal_coalitions.len()
        );

        Self {
            player_count: coop_game.get_player_count(),
            minimal_coalitions,
        }
    }

    fn subset_of(a: u64, b: u64) -> bool {
        (a | b) == b
    }
}

impl SimpleCooperativeGame for MinimalCoalitionCache {
    fn get_player_count(&self) -> usize {
        self.player_count
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

impl MonotoneCooperativeGame for MinimalCoalitionCache {}
