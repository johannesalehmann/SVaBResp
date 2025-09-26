use super::Successor;

pub struct DistributionVector {
    successors: Vec<Successor>,
}

impl super::Distribution for DistributionVector {
    type Builder = Builder;

    fn get_builder() -> Self::Builder {
        Builder::new()
    }

    fn number_of_successors(&self) -> usize {
        self.successors.len()
    }

    fn get_successor(&self, index: usize) -> Successor {
        self.successors[index]
    }
}

pub struct Builder {
    distribution: DistributionVector,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            distribution: DistributionVector {
                successors: Vec::new(),
            },
        }
    }
}

impl super::DistributionBuilder<DistributionVector> for Builder {
    fn add_successor(&mut self, successor: Successor) {
        self.distribution.successors.push(successor);
    }

    fn finish(self) -> DistributionVector {
        self.distribution
    }
}
