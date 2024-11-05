use euclid::default::Point2D;
use num_traits::cast::NumCast;
use crate::more_itertools::MoreIterTools;

#[derive(Debug)]
pub struct Polygon<T: Copy + Ord + NumCast> {
    vertices: Vec<Point2D<T>>,
}

impl<T: Copy + Ord + NumCast> Polygon<T> {
    pub fn new(vertices: impl Iterator<Item = Point2D<T>>) -> Self {
        Self { vertices: vertices.collect() }
    }
}

pub trait Polygonlike<T: Copy + Ord + NumCast> {
    fn vertices(&self) -> impl Iterator<Item = Point2D<T>>;
    
    fn edges(&self) -> impl Iterator<Item = (Point2D<T>, Point2D<T>)> {
        self.vertices().circular_pairs()
    }
}

impl<T: Copy + Ord + NumCast> Polygonlike<T> for Polygon<T> {
    fn vertices(&self) -> impl Iterator<Item = Point2D<T>> {
        self.vertices.iter().cloned()
    }
}
