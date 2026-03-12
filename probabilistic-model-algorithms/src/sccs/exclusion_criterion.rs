pub trait ExclusionCriterion {
    type Iterator<'a>: Iterator<Item = usize>
    where
        Self: 'a;

    fn iter_states<'a>(&'a self) -> Self::Iterator<'a>;

    // This function iterates excluded states by repeatedly calling is_state_excluded. This is
    // useful for quickly implementing iter_states(...). However, if is_state_excluded is expensive
    // and the underlying data structure permits more efficient iteration (e.g. if the excluded
    // states are stored in a list, then it is recommended to manually implement iter_states(...)
    // Due to the lack of associated type defaults, it is not possible to provide a default
    // implementation for iter_states(...).
    fn automatic_iter_states<'a>(
        &'a self,
        model_size: usize,
    ) -> ExclusionCriterionIterator<'a, Self> {
        ExclusionCriterionIterator {
            next_index: 0,
            model_size,
            exclusion_criterion: &self,
        }
    }

    fn is_state_excluded(&self, index: usize) -> bool;
    fn is_action_excluded(&self, state_index: usize, action_index: usize) -> bool;
}

pub struct ExclusionCriterionIterator<'a, Ex: ExclusionCriterion + ?Sized> {
    next_index: usize,
    model_size: usize,
    exclusion_criterion: &'a Ex,
}

impl<'a, Ex: ExclusionCriterion> Iterator for ExclusionCriterionIterator<'a, Ex> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_index < self.model_size {
            if self.exclusion_criterion.is_state_excluded(self.next_index) {
                let result = Some(self.next_index);
                self.next_index += 1;
                return result;
            }
            self.next_index += 1;
        }
        None
    }
}

pub struct NoExclusion {}

impl NoExclusion {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExclusionCriterion for NoExclusion {
    type Iterator<'a> = std::iter::Empty<usize>;

    fn iter_states(&self) -> Self::Iterator<'_> {
        std::iter::empty()
    }

    fn is_state_excluded(&self, index: usize) -> bool {
        let _ = index;
        false
    }

    fn is_action_excluded(&self, state_index: usize, action_index: usize) -> bool {
        let _ = (state_index, action_index);
        false
    }
}

pub struct ExclusionList<'a> {
    excluded_states: &'a [usize],
}

impl<'a> ExclusionList<'a> {
    pub fn new(excluded_states: &'a [usize]) -> Self {
        Self { excluded_states }
    }
}

impl<'a> ExclusionCriterion for ExclusionList<'a> {
    type Iterator<'b>
        = std::iter::Cloned<std::slice::Iter<'b, usize>>
    where
        Self: 'b;

    fn iter_states(&self) -> Self::Iterator<'_> {
        self.excluded_states.iter().cloned()
    }

    fn is_state_excluded(&self, index: usize) -> bool {
        self.excluded_states.contains(&index)
    }

    fn is_action_excluded(&self, state_index: usize, action_index: usize) -> bool {
        let _ = (state_index, action_index);
        false
    }
}
