use probabilistic_models::{ActionCollection, Distribution, ModelTypes, ProbabilisticModel};

pub type SccsWithDependencies = SccList<SccWithDependencies>;

pub struct SccList<T = SccWithoutDependencies> {
    pub sccs: Vec<T>,
}

impl<T> SccList<T> {
    pub fn iter(&self) -> <&'_ Self as IntoIterator>::IntoIter {
        IntoIterator::into_iter(self)
    }

    pub fn iter_mut(&mut self) -> <&'_ mut Self as IntoIterator>::IntoIter {
        IntoIterator::into_iter(self)
    }

    pub fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        IntoIterator::into_iter(self)
    }
}

impl<T> IntoIterator for SccList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sccs.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a SccList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sccs.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SccList<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sccs.iter_mut()
    }
}

pub trait Scc {
    fn get_members(&self) -> &[usize];
    fn is_trivial(&self) -> bool;
}

pub struct SccWithoutDependencies {
    pub members: Vec<usize>,
    pub is_trivial: bool,
}

impl Scc for SccWithoutDependencies {
    fn get_members(&self) -> &[usize] {
        &self.members[..]
    }

    fn is_trivial(&self) -> bool {
        self.is_trivial
    }
}

pub struct SccListBuilder {
    result: SccList<SccWithoutDependencies>,
}

impl super::BuildableScc for SccList<SccWithoutDependencies> {
    type BuilderType = SccListBuilder;

    fn builder<M: ModelTypes>(model: &ProbabilisticModel<M>) -> Self::BuilderType {
        let _ = model;
        SccListBuilder {
            result: SccList { sccs: Vec::new() },
        }
    }
}

impl super::SccBuilder<SccList<SccWithoutDependencies>> for SccListBuilder {
    fn add_scc(&mut self) -> usize {
        let index = self.result.sccs.len();
        self.result.sccs.push(SccWithoutDependencies {
            members: Vec::new(),
            is_trivial: true,
        });
        index
    }

    fn add_to_scc(&mut self, state_index: usize, scc_index: usize) {
        self.result.sccs[scc_index].members.push(state_index);
    }

    fn mark_non_trivial(&mut self, scc_index: usize) {
        self.result.sccs[scc_index].is_trivial = false;
    }

    fn finish(self) -> SccList<SccWithoutDependencies> {
        self.result
    }
}

impl SccList<SccWithoutDependencies> {
    pub fn compute_dependencies<M: ModelTypes>(
        self,
        model: &ProbabilisticModel<M>,
    ) -> SccList<SccWithDependencies> {
        let mut res = SccList {
            sccs: self
                .sccs
                .into_iter()
                .map(|scc| SccWithDependencies {
                    members: scc.members,
                    depends_on: Vec::new(),
                    is_trivial: scc.is_trivial,
                })
                .collect(),
        };

        let mut state_to_scc = vec![0; model.states.len()];
        for (scc_index, scc) in res.sccs.iter_mut().enumerate() {
            for &state in &scc.members {
                state_to_scc[state] = scc_index;
            }
        }

        for (scc_index, scc) in res.sccs.iter_mut().enumerate() {
            for &state in &scc.members {
                for action in model.states[state].actions.iter() {
                    for successor in action.successors.iter() {
                        let successor_scc = state_to_scc[successor.index];
                        if successor_scc != scc_index && !scc.depends_on.contains(&successor_scc) {
                            scc.depends_on.push(successor_scc);
                        }
                    }
                }
            }
        }

        res
    }
}

pub struct SccWithDependencies {
    pub members: Vec<usize>,
    pub is_trivial: bool,
    pub depends_on: Vec<usize>,
}

impl Scc for SccWithDependencies {
    fn get_members(&self) -> &[usize] {
        &self.members[..]
    }

    fn is_trivial(&self) -> bool {
        self.is_trivial
    }
}

impl SccList<SccWithDependencies> {
    pub fn get_reverse_topological_order(&self) -> Vec<usize> {
        let mut visited = vec![false; self.sccs.len()];
        let mut order = Vec::new();

        for scc in 0..self.sccs.len() {
            if !visited[scc] {
                visited[scc] = true;
                self.dfs(scc, &mut visited, &mut order);
            }
        }

        order
    }

    fn dfs(&self, index: usize, visited: &mut Vec<bool>, order: &mut Vec<usize>) {
        for &scc in &self.sccs[index].depends_on {
            if !visited[scc] {
                visited[scc] = true;
                self.dfs(scc, visited, order);
            }
        }
        order.push(index);
    }
}
