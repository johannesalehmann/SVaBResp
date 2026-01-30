mod flags_state_region;
pub use flags_state_region::*;

mod state_region_operations;
pub use state_region_operations::*;

mod vector_state_region;
pub use vector_state_region::*;

pub trait MutableStateRegion: StateRegion {
    fn create(size: usize) -> Self;
    fn clear(&mut self);
    fn add_state(&mut self, index: usize);
}

pub trait StateRegion: Sized {
    fn model_state_count(&self) -> usize;

    fn contains(&self, index: usize) -> bool;

    fn size(&self) -> usize;

    fn inverted(self) -> InvertedStateRegion<Self> {
        InvertedStateRegion::new(self)
    }
}

pub trait OrderedStateRegion: IntoIterator<Item = usize> {}

trait BoxableStateRegion {
    fn get_size(&self) -> usize;

    fn contains(&self, index: usize) -> bool;
}

impl<R: StateRegion> BoxableStateRegion for R {
    fn get_size(&self) -> usize {
        StateRegion::model_state_count(self)
    }

    fn contains(&self, index: usize) -> bool {
        StateRegion::contains(self, index)
    }
}

pub struct BoxedStateRegion {
    inner: Box<dyn BoxableStateRegion>,
}
impl BoxedStateRegion {
    pub fn get_size(&self) -> usize {
        self.inner.get_size()
    }
    pub fn contains(&self, index: usize) -> bool {
        self.inner.contains(index)
    }
}

impl<R: StateRegion + 'static> From<R> for BoxedStateRegion {
    fn from(value: R) -> Self {
        BoxedStateRegion {
            inner: Box::new(value),
        }
    }
}
