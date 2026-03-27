use super::Successor;
use crate::Distribution;

pub struct DistributionVector {
    successors: Vec<Successor>,
}

impl DistributionVector {
    pub fn new() -> Self {
        Self {
            successors: Vec::new(),
        }
    }

    pub fn with_successors(successors: Vec<Successor>) -> Self {
        Self { successors }
    }

    pub fn successors(&self) -> &[Successor] {
        &self.successors
    }

    pub fn successors_mut(&mut self) -> &mut Vec<Successor> {
        &mut self.successors
    }
}

impl super::Distribution for DistributionVector {
    type Builder = Builder;
    type Iter<'a>
        = std::slice::Iter<'a, Successor>
    where
        Self: 'a;

    fn get_builder() -> Self::Builder {
        Builder::new()
    }

    fn number_of_successors(&self) -> usize {
        self.successors.len()
    }

    fn get_successor(&self, index: usize) -> Successor {
        self.successors[index]
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        self.successors.iter()
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
        let sum: f64 = self
            .distribution
            .successors
            .iter()
            .map(|s| s.probability)
            .sum();
        let eps = 0.000_000_001;
        if sum < 1.0 - eps || sum > 1.0 + eps {
            let detail = if self.distribution.number_of_successors() == 0 {
                "Distribution is empty".to_string()
            } else if self.distribution.number_of_successors() == 1 {
                format!(
                    "Distribution contains a single entry with probability {}",
                    sum
                )
            } else {
                format!(
                    "Distribution contains {} entries with probability {} = {}",
                    self.distribution.number_of_successors(),
                    self.distribution
                        .successors
                        .iter()
                        .map(|s| s.probability.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    sum
                )
            };
            panic!("Probabilities of distribution do not add to 1: {detail}");
        }
        self.distribution
    }
}
