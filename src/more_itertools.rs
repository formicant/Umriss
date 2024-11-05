pub trait MoreIterTools: Iterator {
    /// Works like `Itertools::circular_tuple_windows` but
    /// for `Copy` items only and
    /// does not require the iterator to be `ExactSizedIterator`.
    fn circular_pairs<T>(mut self) -> CircularPairIter<T, Self>
    where
        T: Copy,
        Self: Iterator<Item = T> + Sized
    {
        match self.next() {
            Some(item) => CircularPairIter { iter: self, first_item: Some(item), previous_item: Some(item) },
            None => CircularPairIter { iter: self, first_item: None, previous_item: None }
        }
    }
}

impl<T> MoreIterTools for T where T: Iterator + ?Sized { }

pub struct CircularPairIter<T, I> {
    iter: I,
    first_item: Option<T>,
    previous_item: Option<T>,
}

impl<T, I> Iterator for CircularPairIter<T, I>
where
    T: Copy,
    I: Iterator<Item = T> + Sized,
{
    type Item = (T, T);
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(current) => {
                let previous = self.previous_item.unwrap();
                self.previous_item = Some(current);
                Some((previous, current))
            },
            None => self.previous_item.map(|previous| {
                let current = self.first_item.unwrap();
                self.previous_item = None;
                (previous, current)
            }),
        }
    }
}
