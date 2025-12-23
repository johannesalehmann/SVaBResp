use super::{MutableStateRegion, OrderedStateRegion, StateRegion};

pub struct FlagStateRegion {
    flags: Vec<bool>,
}

impl FlagStateRegion {
    fn iter(&self) -> <&FlagStateRegion as IntoIterator>::IntoIter {
        (&self).into_iter()
    }
}

impl MutableStateRegion for FlagStateRegion {
    fn create(size: usize) -> Self {
        Self {
            flags: vec![false; size],
        }
    }

    fn clear(&mut self) {
        for flag in &mut self.flags {
            *flag = false;
        }
    }

    fn add_state(&mut self, index: usize) {
        self.flags[index] = true;
    }
}

impl StateRegion for FlagStateRegion {
    fn get_size(&self) -> usize {
        self.flags.len()
    }

    fn contains(&self, index: usize) -> bool {
        self.flags[index]
    }
}

impl OrderedStateRegion for &FlagStateRegion {}

impl<'a> IntoIterator for &'a FlagStateRegion {
    type Item = usize;
    type IntoIter = FlagStateRegionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

pub struct FlagStateRegionIterator<'a> {
    index: usize,
    data: &'a [bool],
}

impl<'a> Iterator for FlagStateRegionIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.data.len() {
            if self.data[self.index] {
                let index = self.index;
                self.index += 1;
                return Some(index);
            } else {
                self.index += 1;
            }
        }
        None
    }
}
