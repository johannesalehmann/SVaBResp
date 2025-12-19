mod minimal_coalition_cache;
pub use minimal_coalition_cache::MinimalCoalitionCache;

pub trait PlayerDescriptions {
    type IntoIter: Iterator<Item = Self::PlayerType>;
    type PlayerType;
    fn get_player_description(&self, index: usize) -> &Self::PlayerType;
    fn into_iterator(self) -> Self::IntoIter;
}

impl<P> PlayerDescriptions for Vec<P> {
    type IntoIter = std::vec::IntoIter<P>;
    type PlayerType = P;

    fn get_player_description(&self, index: usize) -> &Self::PlayerType {
        &self.get(index).unwrap()
    }

    fn into_iterator(self) -> Self::IntoIter {
        IntoIterator::into_iter(self)
    }
}

pub trait CooperativeGame {
    type PlayerDescriptions: PlayerDescriptions;

    fn get_player_count(&self) -> usize;
    fn player_descriptions(&self) -> &Self::PlayerDescriptions;
    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions;
    fn into_player_descriptions(self) -> Self::PlayerDescriptions;
    fn get_value<C: CoalitionSpecifier>(&mut self, coalition: C) -> f64;
}

pub trait SimpleCooperativeGame {
    type PlayerDescriptions: PlayerDescriptions;

    fn get_player_count(&self) -> usize;
    fn player_descriptions(&self) -> &Self::PlayerDescriptions;
    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions;
    fn into_player_descriptions(self) -> Self::PlayerDescriptions;
    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool;
}

impl<G: SimpleCooperativeGame> CooperativeGame for G {
    type PlayerDescriptions = <G as SimpleCooperativeGame>::PlayerDescriptions;

    fn get_player_count(&self) -> usize {
        SimpleCooperativeGame::get_player_count(self)
    }

    fn player_descriptions(&self) -> &Self::PlayerDescriptions {
        SimpleCooperativeGame::player_descriptions(self)
    }

    fn player_descriptions_mut(&mut self) -> &mut Self::PlayerDescriptions {
        SimpleCooperativeGame::player_descriptions_mut(self)
    }

    fn into_player_descriptions(self) -> Self::PlayerDescriptions {
        SimpleCooperativeGame::into_player_descriptions(self)
    }

    fn get_value<C: CoalitionSpecifier>(&mut self, coalition: C) -> f64 {
        match self.is_winning(coalition) {
            true => 1.0,
            false => 0.0,
        }
    }
}

pub trait MonotoneCooperativeGame {}

pub trait CoalitionSpecifier {
    fn max_size() -> usize;
    fn is_in_coalition(&self, index: usize) -> bool;

    fn to_mask(&self) -> u64 {
        assert!(
            Self::max_size() <= 64,
            "Can only create a bit mask for coalitions that have a size of at most 64"
        );

        let mut mask = 0;
        for i in 0..Self::max_size() {
            if self.is_in_coalition(i) {
                mask = mask | 1 << i;
            }
        }
        mask
    }
}

impl CoalitionSpecifier for u64 {
    fn max_size() -> usize {
        64
    }

    fn is_in_coalition(&self, index: usize) -> bool {
        (1 << index) & self != 0
    }

    fn to_mask(&self) -> u64 {
        *self
    }
}
