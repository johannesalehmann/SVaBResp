use super::super::{OrderedStateRegion, StateRegion};

pub struct InvertedStateRegion<R: StateRegion> {
    base: R,
}

impl<R: StateRegion> InvertedStateRegion<R> {
    pub fn new(base: R) -> Self {
        Self { base }
    }
}
impl<'a, R: StateRegion + 'a> InvertedStateRegion<R>
where
    &'a R: OrderedStateRegion,
{
    fn iter(&'a self) -> <&'a Self as IntoIterator>::IntoIter {
        (&self).into_iter()
    }
}

impl<'a, R: StateRegion> StateRegion for InvertedStateRegion<R> {
    fn get_size(&self) -> usize {
        self.base.get_size()
    }

    fn contains(&self, index: usize) -> bool {
        !self.base.contains(index)
    }
}

impl<'a, R: StateRegion + 'a> IntoIterator for &'a InvertedStateRegion<R>
where
    &'a R: OrderedStateRegion,
{
    type Item = usize;
    type IntoIter = InvertedStateIterator<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        InvertedStateIterator::new(&self)
    }
}

pub struct InvertedStateIterator<'a, R: StateRegion>
where
    &'a R: OrderedStateRegion,
{
    base_iter: <&'a R as IntoIterator>::IntoIter,
    counter: usize,
    next_base_value: Option<usize>,
    size: usize,
}

impl<'a, R: StateRegion> InvertedStateIterator<'a, R>
where
    &'a R: OrderedStateRegion,
{
    fn new(region: &'a InvertedStateRegion<R>) -> Self {
        let mut base_iter = (&region.base).into_iter();
        let next_base_value = base_iter.next();
        Self {
            base_iter,
            counter: 0,
            next_base_value,
            size: region.get_size(),
        }
    }
}

impl<'a, R: StateRegion> Iterator for InvertedStateIterator<'a, R>
where
    &'a R: OrderedStateRegion,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next_value) = self.next_base_value
            && next_value == self.counter
        {
            self.counter += 1;
            self.next_base_value = self.base_iter.next();
        }

        if self.next_base_value.is_some() {
            let result = Some(self.counter);
            self.counter += 1;
            result
        } else {
            if self.counter < self.size {
                let result = Some(self.counter);
                self.counter += 1;
                result
            } else {
                None
            }
        }
    }
}

impl<'a, R: StateRegion> OrderedStateRegion for InvertedStateIterator<'a, R> where
    &'a R: OrderedStateRegion
{
}

#[cfg(test)]
mod tests {
    use crate::regions::MutableStateRegion;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_iter() {
        let mut base = crate::regions::VectorStateRegion::create(10);
        base.add_state(0);
        base.add_state(1);
        base.add_state(4);
        base.add_state(6);
        base.add_state(7);

        let inverted = base.sorted().inverted();

        let iter_result = inverted.into_iter().collect::<Vec<_>>();
        assert_eq!(iter_result, vec![2, 3, 5, 8, 9]);
    }
}
