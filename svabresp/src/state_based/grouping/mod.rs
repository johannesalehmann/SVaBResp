mod group_extraction;
pub use group_extraction::*;

pub trait StateGroups {
    fn get_count(&self) -> usize;
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
    fn get_count(&self) -> usize {
        self.groups.len()
    }
}
