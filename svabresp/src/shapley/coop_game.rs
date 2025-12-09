pub trait CooperativeGame {
    fn get_player_count(&self) -> usize;
    fn get_value<C: CoalitionSpecifier>(&mut self, coalition: C) -> f64;
}

pub trait SimpleCooperativeGame {
    fn get_player_count(&self) -> usize;
    fn is_winning<C: CoalitionSpecifier>(&mut self, coalition: C) -> bool;
}

impl<G: SimpleCooperativeGame> CooperativeGame for G {
    fn get_player_count(&self) -> usize {
        CooperativeGame::get_player_count(self)
    }

    fn get_value<C: CoalitionSpecifier>(&mut self, coalition: C) -> f64 {
        match self.is_winning(coalition) {
            true => 1.0,
            false => 0.0,
        }
    }
}

pub trait CoalitionSpecifier {
    fn max_size() -> usize;
    fn is_in_coalition(&self, index: usize) -> bool;
}

impl CoalitionSpecifier for u64 {
    fn max_size() -> usize {
        64
    }

    fn is_in_coalition(&self, index: usize) -> bool {
        (1 << index) & self != 0
    }
}
