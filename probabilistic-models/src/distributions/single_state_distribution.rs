use crate::Successor;

pub struct SingleStateDistribution {
    successor: Successor,
}

impl super::Distribution for SingleStateDistribution {
    type Builder = Builder;
    type Iter<'a>
        = std::iter::Once<&'a Successor>
    where
        Self: 'a;

    fn get_builder() -> Self::Builder {
        Builder::new()
    }

    fn number_of_successors(&self) -> usize {
        1
    }

    fn get_successor(&self, index: usize) -> Successor {
        if index != 0 {
            panic!("Successor index out of bounds");
        }
        self.successor
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        std::iter::once(&self.successor)
    }
}

pub struct Builder {
    successor: Option<Successor>,
}

impl Builder {
    pub fn new() -> Self {
        Self { successor: None }
    }
}

impl super::DistributionBuilder<SingleStateDistribution> for Builder {
    fn add_successor(&mut self, action: Successor) {
        match &self.successor {
            None => self.successor = Some(action),
            Some(_) => panic!("Cannot add a second transition to a state of this model type"),
        }
    }

    fn finish(self) -> SingleStateDistribution {
        match self.successor {
            Some(successor) => SingleStateDistribution { successor },
            None => panic!("Must add at least one transition to each state in this model type"),
        }
    }
}
