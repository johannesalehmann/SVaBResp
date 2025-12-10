mod group_extraction;
pub use group_extraction::*;

pub trait StateGroups {
    type Iter<'a>: Iterator<Item = usize>
    where
        Self: 'a;

    fn get_count(&self) -> usize;
    fn get_states<'a>(&'a self, group: usize) -> Self::Iter<'a>;
}

pub struct VectorStateGroups {
    groups: Vec<VectorStateGroup>,
}

impl VectorStateGroups {
    pub fn get_builder() -> VectorStateGroupBuilder {
        VectorStateGroupBuilder::new()
    }
}

pub struct VectorStateGroup {
    states: Vec<usize>,
}

pub struct VectorStateGroupBuilder {
    groups: Vec<VectorStateGroup>,
    group_in_progress: Vec<usize>,
}

impl VectorStateGroupBuilder {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            group_in_progress: Vec::new(),
        }
    }

    pub fn add_state(&mut self, state: usize) {
        self.group_in_progress.push(state);
    }

    pub fn finish_group(&mut self) {
        let states = std::mem::replace(&mut self.group_in_progress, Vec::new());
        self.groups.push(VectorStateGroup { states })
    }

    pub fn finish(mut self) -> VectorStateGroups {
        self.finish_group();
        VectorStateGroups {
            groups: self.groups,
        }
    }
}

impl StateGroups for VectorStateGroups {
    type Iter<'a> = std::iter::Cloned<std::slice::Iter<'a, usize>>;

    fn get_count(&self) -> usize {
        self.groups.len()
    }

    fn get_states<'a>(&'a self, group: usize) -> Self::Iter<'a> {
        self.groups[group].states.iter().cloned()
    }
}
