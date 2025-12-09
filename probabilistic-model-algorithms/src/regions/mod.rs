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
    fn get_size(&self) -> usize;

    fn is_set(&self, index: usize) -> bool;

    fn inverted(self) -> InvertedStateRegion<Self> {
        InvertedStateRegion::new(self)
    }
}

pub trait OrderedStateRegion: IntoIterator<Item = usize> {}
