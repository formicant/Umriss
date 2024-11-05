use euclid::default::Point2D;
use crate::more_itertools::MoreIterTools;
use super::Number;
use super::polygon::Polygonlike;

/// Position of a point relative to an orthopolygon.
pub enum PointPosition {
    Outside,
    Vertex,
    Edge,
    Inside,
}

/// Implements a polygon with all edges parallel to the coordinate axes.
/// Each even edge is parallel to the x axis.
/// Each odd edge is parallel to the y axis.
/// 
/// Implements `Eq` and `Hash`.
/// Warning! Not all same-shaped polygons are equal.
/// Two `Orthopolygon`s are considered equal only if the have identical vertex lists,
/// i.e. they have the same shape and also start from the same point and have the same direction.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Orthopolygon<N: Number> {
    even_vertices: Vec<Point2D<N>>,
}

impl<N: Number> Orthopolygon<N> {
    /// Creates an orthopolygon from an iterator of even vertices.
    pub fn new(even_vertices: impl Iterator<Item = Point2D<N>>) -> Self {
        Self { even_vertices: even_vertices.collect() }
    }
    
    /// Creates an orthopolygon from a `Vec` of even vertices.
    /// Takes the ownership of the `Vec`.
    pub fn from(even_vertices: Vec<Point2D<N>>) -> Self {
        Self { even_vertices }
    }
}

/// Represents a polygon with all edges parallel to the coordinate axes.
/// Each even edge is parallel to the x axis.
/// Each odd edge is parallel to the y axis.
pub trait Orthopolygonlike<N: Number>: Polygonlike<N> {
    /// Iterates even vertices of the orthopolygon.
    /// 
    /// Even vertices are sufficient to represent an orthopolygon.
    /// Odd vertices can be derived from them unambiguously:
    /// an odd vertex inherits its y coordinate from the previous
    /// even vertex and x coordinate from the next even vertex.
    fn even_vertices(&self) -> impl Iterator<Item = Point2D<N>>;
    
    /// Creates a new orthopolygon of the same shape with, possibly,
    /// other type of points (`Point2D<NDest>`).
    fn to_orthopolygon<NDest: Number>(&self) -> Orthopolygon<NDest> {
        let even_vertices = self.even_vertices()
            .map(|v| v.cast())
            .collect();
        Orthopolygon { even_vertices }
    }
    
    /// Uses ray casting algorithm to determine if the point is
    /// inside the orthopolygon, outside, on an edge, or at a vertex.
    fn get_point_position(&self, point: Point2D<N>) -> PointPosition {
        let Point2D { x, y, .. } = point;
        let mut intersections = 0;
        
        for (Point2D { x: x0, y: y0, .. }, Point2D { x: x1, y: y1, .. }) in self.even_vertices().circular_pairs() {
            let is_x_between = (x0 <= x && x < x1) || (x1 <= x && x < x0);
            if y0 < y && is_x_between {
                intersections += 1;
            }
            if (y0 == y && (x == x0 || x == x1)) || (y == y1 && x == x1) {
                return PointPosition::Vertex;
            }
            let is_y_between = (y0 <= y && y < y1) || (y1 <= y && y < y0);
            if  (y == y0 && is_x_between) || (x == x1 && is_y_between) {
                return PointPosition::Edge;
            }
        }
        if intersections % 2 == 0 { PointPosition::Outside } else { PointPosition::Inside }
    }
}

impl<N: Number> Orthopolygonlike<N> for Orthopolygon<N> {
    fn even_vertices(&self) -> impl Iterator<Item = Point2D<N>> {
        self.even_vertices.iter().cloned()
    }
}

impl<N: Number> Polygonlike<N> for Orthopolygon<N> {
    fn vertices(&self) -> impl Iterator<Item = Point2D<N>> {
        self.even_vertices()
            .circular_pairs()
            .flat_map(|(p0, p1)| [p0, Point2D::new(p1.x, p0.y)])
    }
}
