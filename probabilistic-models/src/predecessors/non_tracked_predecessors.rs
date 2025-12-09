use super::{Predecessor, Predecessors, PredecessorsBuilder};

pub struct NonTrackedPredecessors {}

impl Predecessors for NonTrackedPredecessors {
    type Iter<'a> = std::vec::IntoIter<Predecessor>;
    type Builder = NonTrackedPredecessorsBuilder;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        panic!("Cannot iterate predecessors when they are not tracked")
    }
}

pub struct NonTrackedPredecessorsBuilder {}

impl PredecessorsBuilder<NonTrackedPredecessors> for NonTrackedPredecessorsBuilder {
    fn create() -> Self {
        Self {}
    }

    fn add(&mut self, predecessor: Predecessor) {
        panic!("Cannot add a predecessor in a model that does not track predecessors")
    }

    fn finish(self) -> NonTrackedPredecessors {
        NonTrackedPredecessors {}
    }
}
