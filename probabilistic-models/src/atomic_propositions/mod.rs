pub trait AtomicPropositions {
    fn get_empty(capacity: usize) -> Self;
    fn set_value(&mut self, index: usize, value: bool);
    fn get_value(&self, index: usize) -> bool;
}

pub struct BitFlagsAtomicPropositions {
    values: u64,
}

impl AtomicPropositions for BitFlagsAtomicPropositions {
    fn get_empty(capacity: usize) -> Self {
        if capacity > 64 {
            panic!("Bitflags can only be used in models with up to 64 atomic propositions");
        }
        BitFlagsAtomicPropositions { values: 0 }
    }

    fn set_value(&mut self, index: usize, value: bool) {
        match value {
            true => self.values = self.values | 1 << index,
            false => self.values = self.values & !(1 << index),
        }
    }

    fn get_value(&self, index: usize) -> bool {
        (self.values & 1 << index) != 0
    }
}
