use crate::shapley::{CoalitionSpecifier, CooperativeGame, PlayerDescriptions};
use log::info;

pub struct GameValueCache<P: PlayerDescriptions> {
    player_descriptions: P,
    player_count: usize,
    pub values: Vec<f64>,
}

impl<P: PlayerDescriptions + Clone> GameValueCache<P> {
    pub fn create<C: CooperativeGame<PlayerDescriptions = P>>(coop_game: &mut C) -> Self {
        let mut values = Vec::new();

        let n = coop_game.get_player_count();

        info!("Building game value cache for n={} players", n);
        let start = std::time::Instant::now();
        for coalition in 0..1u64 << n {
            let value = coop_game.get_value(coalition);
            values.push(value);
        }
        info!(
            "Finished building game value cache in {:?}",
            start.elapsed()
        );

        Self {
            player_descriptions: coop_game.player_descriptions().clone(),
            player_count: coop_game.get_player_count(),
            values,
        }
    }
}

impl<P: PlayerDescriptions + Clone> CooperativeGame for GameValueCache<P> {
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

    fn get_value<C: CoalitionSpecifier>(&mut self, coalition: C) -> f64 {
        self.values[coalition.to_mask() as usize]
    }
}
