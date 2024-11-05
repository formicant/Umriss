use euclid::default::Point2D;
use crate::more_itertools::MoreIterTools;
use super::Number;

#[derive(Debug)]
pub struct Polygon<T: Number> {
    vertices: Vec<Point2D<T>>,
}

impl<T: Number> Polygon<T> {
    pub fn new(vertices: impl Iterator<Item = Point2D<T>>) -> Self {
        Self { vertices: vertices.collect() }
    }
}

pub trait Polygonlike<T: Number> {
    fn vertices(&self) -> impl Iterator<Item = Point2D<T>>;
    
    fn edges(&self) -> impl Iterator<Item = (Point2D<T>, Point2D<T>)> {
        self.vertices().circular_pairs()
    }
}

impl<T: Number> Polygonlike<T> for Polygon<T> {
    fn vertices(&self) -> impl Iterator<Item = Point2D<T>> {
        self.vertices.iter().cloned()
    }
}
