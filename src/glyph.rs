use std::iter;
use euclid::default::{Point2D, Size2D};
use crate::geometry::{Orthopolygonlike, Orthopolygon};
use crate::image_contour_collection::Contour;

/// A _glyph_ is a contour, possibly, with holes, that does not remember its location.
/// The upper-left corner of a glyph’s bounding box always has the coordinates (0, 0).
/// 
/// Implements `Eq` and `Hash`. Two glyphs of the same shape are considered equal.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Glyph {
    size: Size2D<i32>,
    contours: Vec<Orthopolygon>,
}

impl Glyph {
    /// Takes a contour from an `ImageContourCollection`,
    /// creates a glyph from it (glyph’s outer contour) and its children (holes),
    /// and returns it along with the location of its upper-left corner.
    /// 
    /// The lifespan of the glyph is independent from the lifespan of the contour collection.
    pub fn from_contour(outer_contour: Contour) -> (Self, Point2D<i32>) {
        let mut x_min = i32::MAX;
        let mut x_max = i32::MIN;
        let mut y_min = i32::MAX;
        let mut y_max = i32::MIN;
        
        for Point2D { x, y, .. } in outer_contour.even_vertices() {
            if x < x_min { x_min = x; }
            if x > x_max { x_max = x; }
            if y < y_min { y_min = y; }
            if y > y_max { y_max = y; }
        }
        let width = x_max - x_min + 1;
        let height = y_max - y_min + 1;
        
        let location = Point2D::new(x_min, y_min);
        let size = Size2D::new(width, height);
        
        let contours = iter::once(outer_contour)
            .chain(outer_contour.children())
            .map(|contour| {
                let even_vertices = contour.even_vertices()
                    .map(|Point2D { x, y, .. }| Point2D::new(x - x_min, y - y_min));
                Orthopolygon::new(even_vertices)
            })
            .collect();
        
        (Self { size, contours }, location)
    }
    
    /// Width and height of the glyph’s bounding box.
    pub fn size(&self) -> Size2D<i32> {
        self.size
    }
    
    /// Contours of the glyph. The outer contour goes first, then, the holes.
    pub fn contours(&self) -> &[Orthopolygon] {
        &self.contours[..]
    }
    
    /// Returns a reference to the outer contour of the glyph.
    pub fn outer_contour(&self) -> &Orthopolygon {
        &self.contours[0]
    }
    
    /// Returns a slice of all hole contours of the glyph.
    pub fn inner_contours(&self) -> &[Orthopolygon] {
        &self.contours[1..]
    }
}
