use euclid::default::Point2D;
use crate::more_itertools::MoreIterTools;
use super::Number;

#[derive(Debug)]
pub struct Polygon<N: Number> {
    vertices: Vec<Point2D<N>>,
}

impl<N: Number> Polygon<N> {
    pub fn new(vertices: impl Iterator<Item = Point2D<N>>) -> Self {
        Self { vertices: vertices.collect() }
    }
    
    pub fn from(vertices: Vec<Point2D<N>>) -> Self {
        Self { vertices }
    }
}

pub trait Polygonlike<N: Number> {
    fn vertices(&self) -> impl Iterator<Item = Point2D<N>>;
    
    fn edges(&self) -> impl Iterator<Item = (Point2D<N>, Point2D<N>)> {
        self.vertices().circular_pairs()
    }
}

impl<N: Number> Polygonlike<N> for Polygon<N> {
    fn vertices(&self) -> impl Iterator<Item = Point2D<N>> {
        self.vertices.iter().cloned()
    }
}
