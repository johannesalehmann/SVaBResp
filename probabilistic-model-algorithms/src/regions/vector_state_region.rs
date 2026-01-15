use super::{MutableStateRegion, OrderedStateRegion, StateRegion};

pub struct VectorStateRegion {
    states: Vec<usize>,
    size: usize,
}
impl VectorStateRegion {
    pub fn iter(&self) -> <&VectorStateRegion as IntoIterator>::IntoIter {
        (&self).into_iter()
    }
}

impl MutableStateRegion for VectorStateRegion {
    fn create(size: usize) -> Self {
        VectorStateRegion {
            states: Vec::new(),
            size,
        }
    }

    fn clear(&mut self) {
        self.states.clear()
    }

    fn add_state(&mut self, index: usize) {
        self.states.push(index);
    }
}

impl StateRegion for VectorStateRegion {
    fn get_size(&self) -> usize {
        self.size
    }

    fn contains(&self, index: usize) -> bool {
        self.states.iter().any(|i| *i == index)
    }
}

impl<'a> IntoIterator for &'a VectorStateRegion {
    type Item = usize;
    type IntoIter = std::iter::Cloned<std::slice::Iter<'a, usize>>;

    fn into_iter(self) -> Self::IntoIter {
        self.states.iter().cloned()
    }
}

impl VectorStateRegion {
    pub fn sorted(mut self) -> OrderedVectorStateRegion {
        self.states.sort();
        OrderedVectorStateRegion { base: self }
    }
}

pub struct OrderedVectorStateRegion {
    base: VectorStateRegion,
}
impl OrderedVectorStateRegion {
    pub fn iter(&self) -> <&OrderedVectorStateRegion as IntoIterator>::IntoIter {
        (&self).into_iter()
    }
}

impl StateRegion for OrderedVectorStateRegion {
    fn get_size(&self) -> usize {
        self.base.size
    }

    fn contains(&self, index: usize) -> bool {
        self.base.contains(index)
    }
}

impl<'a> IntoIterator for &'a OrderedVectorStateRegion {
    type Item = <&'a VectorStateRegion as IntoIterator>::Item;
    type IntoIter = <&'a VectorStateRegion as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.base.into_iter()
    }
}

impl OrderedStateRegion for &OrderedVectorStateRegion {}
