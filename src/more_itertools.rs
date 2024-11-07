use std::collections::VecDeque;
use itertools::{traits::HomogeneousTuple, Itertools};

pub trait MoreIterTools: Iterator {
    /// Works like `Itertools::circular_tuple_windows` but
    /// for `Copy` items only and
    /// does not require the iterator to be `ExactSizedIterator`.
    /// Si,pler version for pairs.
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
    
    /// Works like `Itertools::circular_tuple_windows` but
    /// for `Copy` items only and
    /// does not require the iterator to be `ExactSizedIterator`.
    fn circular_tuples<Tuple>(mut self) -> CircularTupleIter<Tuple, Self>
    where
        Tuple: HomogeneousTuple,
        Tuple::Item: Copy,
        Self: Iterator<Item = Tuple::Item> + Sized
    {
        let count = Tuple::num_items() - 1;
        let mut first_items: VecDeque<_> = self.by_ref().take(count).collect();
        let len = first_items.len();
        
        let mut previous_items = first_items.clone();
        if len > 0 {
            for _ in 0..(count - len) {
                let item = first_items.pop_front().unwrap();
                first_items.push_back(item);
                previous_items.push_back(item);
            }
        }
        CircularTupleIter { iter: self,  first_items, previous_items }
    }
}

impl<T> MoreIterTools for T where T: Iterator + ?Sized { }

pub struct CircularPairIter<T, Iter> {
    iter: Iter,
    first_item: Option<T>,
    previous_item: Option<T>,
}

impl<T, Iter> Iterator for CircularPairIter<T, Iter>
where
    T: Copy,
    Iter: Iterator<Item = T> + Sized,
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

pub struct CircularTupleIter<Tuple: HomogeneousTuple, Iter> {
    iter: Iter,
    first_items: VecDeque<Tuple::Item>,
    previous_items: VecDeque<Tuple::Item>,
}

impl<Tuple, Iter> Iterator for CircularTupleIter<Tuple, Iter>
where
    Tuple: HomogeneousTuple,
    Tuple::Item: Copy,
    Iter: Iterator<Item = Tuple::Item> + Sized,
{
    type Item = Tuple;
    
    fn next(&mut self) -> Option<Self::Item> {
        let Some(current) =
            self.iter.next().or_else(|| self.first_items.pop_front())
            else { return None; };
        
        self.previous_items.push_back(current);
        let tuple = self.previous_items.iter().cloned().collect_tuple();
        self.previous_items.pop_front();
        tuple
    }
}


// ---------

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use super::*;

    #[test_case(vec![] => Vec::<(i32, i32)>::new())]
    #[test_case(vec![0] => vec![(0, 0)])]
    #[test_case(vec![0, 1] => vec![(0, 1), (1, 0)])]
    #[test_case(vec![3, 2, 1, 3, 0] => vec![(3, 2), (2, 1), (1, 3), (3, 0), (0, 3)])]
    fn test_circular_pairs(items: Vec<i32>) -> Vec<(i32, i32)> {
        items.into_iter().circular_pairs().collect()
    }
    
    #[test_case(vec![] => Vec::<(i32,)>::new())]
    #[test_case(vec![0] => vec![(0,)])]
    #[test_case(vec![0, 1] => vec![(0,), (1,)])]
    #[test_case(vec![3, 2, 1, 3, 0] => vec![(3,), (2,), (1,), (3,), (0,)])]
    fn test_circular_1_tuples(items: Vec<i32>) -> Vec<(i32,)> {
        items.into_iter().circular_tuples().collect()
    }
    
    #[test_case(vec![] => Vec::<(i32, i32)>::new())]
    #[test_case(vec![0] => vec![(0, 0)])]
    #[test_case(vec![0, 1] => vec![(0, 1), (1, 0)])]
    #[test_case(vec![3, 2, 1, 3, 0] => vec![(3, 2), (2, 1), (1, 3), (3, 0), (0, 3)])]
    fn test_circular_2_tuples(items: Vec<i32>) -> Vec<(i32, i32)> {
        items.into_iter().circular_tuples().collect()
    }
    
    #[test_case(vec![] => Vec::<(i32, i32, i32)>::new())]
    #[test_case(vec![0] => vec![(0, 0, 0)])]
    #[test_case(vec![0, 1] => vec![(0, 1, 0), (1, 0, 1)])]
    #[test_case(vec![3, 2, 1, 3, 0] => vec![(3, 2, 1), (2, 1, 3), (1, 3, 0), (3, 0, 3), (0, 3, 2)])]
    fn test_circular_3_tuples(items: Vec<i32>) -> Vec<(i32, i32, i32)> {
        items.into_iter().circular_tuples().collect()
    }
    
    #[test_case(vec![] => Vec::<(i32, i32, i32, i32)>::new())]
    #[test_case(vec![0] => vec![(0, 0, 0, 0)])]
    #[test_case(vec![0, 1] => vec![(0, 1, 0, 1), (1, 0, 1, 0)])]
    #[test_case(vec![3, 2, 1, 3, 0] => vec![(3, 2, 1, 3), (2, 1, 3, 0), (1, 3, 0, 3), (3, 0, 3, 2), (0, 3, 2, 1)])]
    fn test_circular_4_tuples(items: Vec<i32>) -> Vec<(i32, i32, i32, i32)> {
        items.into_iter().circular_tuples().collect()
    }
}
