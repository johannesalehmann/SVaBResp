mod group_extraction;
pub use group_extraction::*;

pub trait StateGroups {
    type Iter<'a>: Iterator<Item = usize>
    where
        Self: 'a;

    fn get_count(&self) -> usize;
    fn get_states<'a>(&'a self, group: usize) -> Self::Iter<'a>;
    fn get_label(&self, group: usize) -> String;
    fn get_dummy_states<'a>(&'a self) -> Self::Iter<'a>;
}

pub struct VectorStateGroups {
    groups: Vec<VectorStateGroup>,
    dummy_states: VectorStateGroup,
}

impl VectorStateGroups {
    pub fn get_builder() -> VectorStateGroupBuilder {
        VectorStateGroupBuilder::new()
    }
}

pub struct VectorStateGroup {
    states: Vec<usize>,
    label: String,
}

pub struct VectorStateGroupBuilder {
    groups: Vec<VectorStateGroup>,
    group_in_progress: Vec<usize>,
    dummy_states: Vec<usize>,
}

impl VectorStateGroupBuilder {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            group_in_progress: Vec::new(),
            dummy_states: Vec::new(),
        }
    }

    pub fn add_state(&mut self, state: usize) {
        self.group_in_progress.push(state);
    }

    pub fn add_dummy_state(&mut self, state: usize) {
        self.dummy_states.push(state);
    }

    pub fn create_group_from_vec(&mut self, states: Vec<usize>, label: String) {
        if self.group_in_progress.len() > 0 {
            panic!("must finish previous group before creating a state group from a vector");
        }
        self.groups.push(VectorStateGroup { states, label })
    }

    pub fn finish_group(&mut self, label: String) {
        let states = std::mem::replace(&mut self.group_in_progress, Vec::new());
        self.groups.push(VectorStateGroup { states, label })
    }

    pub fn finish(self) -> VectorStateGroups {
        if self.group_in_progress.len() > 0 {
            panic!("Must finish group before finishing vector state group");
        }
        VectorStateGroups {
            groups: self.groups,
            dummy_states: VectorStateGroup {
                states: self.dummy_states,
                label: "dummy states".to_string(),
            },
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

    fn get_label(&self, group: usize) -> String {
        self.groups[group].label.clone()
    }

    fn get_dummy_states<'a>(&'a self) -> Self::Iter<'a> {
        self.dummy_states.states.iter().cloned()
    }
}
