use crate::shapley::coop_game::PlayerDescriptions;
use crate::shapley::{
    CoalitionSpecifier, CooperativeGame, MonotoneCooperativeGame, SimpleCooperativeGame,
};

pub struct MinimalCoalitionCache<P: PlayerDescriptions> {
    player_descriptions: P,
    player_count: usize,
    minimal_coalitions: Vec<u64>,
}

impl<P: PlayerDescriptions> MinimalCoalitionCache<P> {
    pub fn create<C: SimpleCooperativeGame<PlayerDescriptions = P> + MonotoneCooperativeGame>(
        mut coop_game: C,
    ) -> Self {
        let mut minimal_coalitions = Vec::new();

        let max_coalition = 1u64 << coop_game.get_player_count();

        let mut game_counter = 0;
        let mut skipped_counter = 0;

        for size in 0..=coop_game.get_player_count() as u32 {
            print!("Size {}/{}", size, coop_game.get_player_count());
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
                " (solved {} games, skipped {})",
                game_counter, skipped_counter
            );
            game_counter = 0;
            skipped_counter = 0;
        }

        println!(
            "Minimal coalition cache contains {} coalitions",
            minimal_coalitions.len()
        );
        for c in &minimal_coalitions {
            println!("  {:b}", c);
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

impl<P: PlayerDescriptions> SimpleCooperativeGame for MinimalCoalitionCache<P> {
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
