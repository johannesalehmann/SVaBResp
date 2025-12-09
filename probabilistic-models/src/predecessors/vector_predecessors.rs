use crate::{Predecessor, Predecessors, PredecessorsBuilder};
use std::array::IntoIter;

pub struct VectorPredecessors {
    predecessors: Vec<Predecessor>,
}

impl Predecessors for VectorPredecessors {
    type Iter<'a>
        = std::iter::Cloned<std::slice::Iter<'a, Predecessor>>
    where
        Self: 'a;
    type Builder = VectorPredecessorsBuilder;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        self.predecessors.iter().cloned()
    }
}

pub struct VectorPredecessorsBuilder {
    predecessors: Vec<Predecessor>,
}

impl PredecessorsBuilder<VectorPredecessors> for VectorPredecessorsBuilder {
    fn create() -> Self {
        Self {
            predecessors: Vec::new(),
        }
    }

    fn add(&mut self, predecessor: Predecessor) {
        self.predecessors.push(predecessor)
    }

    fn finish(self) -> VectorPredecessors {
        VectorPredecessors {
            predecessors: self.predecessors,
        }
    }
}
