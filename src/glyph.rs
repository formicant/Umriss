use std::iter;
use euclid::default::{Point2D, Size2D};
use crate::image_contour_collection::Contour;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GlyphContour {
    is_outer: bool,
    even_points: Vec<Point2D<i32>>,
}

impl GlyphContour {
    /// `true` for the outer contour, `false` for holes.
    pub fn is_outer(&self) -> bool { self.is_outer }
    
    /// Iterate points of the contour.
    /// 
    /// Points are iterated from the top of the contour,
    /// clockwise for the outer contour and anti-clockwise for holes.
    pub fn points<'a>(&'a self) -> impl Iterator<Item = Point2D<i32>> + 'a {
        let len = self.even_points.len();
        (0..len)
            .flat_map(move |even_index| {
                let odd_index = (even_index + 1) % len;
                let even_point = self.even_points[even_index];
                let odd_point = Point2D::new(self.even_points[odd_index].x, even_point.y);
                [even_point, odd_point]
            })
    }
    
    /// Returns a slice of even points of the glyph.
    /// 
    /// Even points are sufficient to represent a contour.
    /// Odd points can be derived from them unambiguously:
    /// an odd point inherits its y coordinate from the previous
    /// even point and x coordinate from the next even point.
    pub fn even_points(&self) -> &[Point2D<i32>] { &self.even_points }
}

/// A _glyph_ is a contour, possibly, with holes, that does not remember its location.
/// The upper-left corner of a glyph’s bounding box always has the coordinates (0, 0).
/// 
/// Implements `Eq` and `Hash`. Two glyphs of the same shape are considered equal.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Glyph {
    size: Size2D<i32>,
    contours: Vec<GlyphContour>,
}

impl Glyph {
    /// Takes a contour from an `ImageContourCollection`,
    /// creates a glyph from it (glyph’s outer contour) and its children (holes),
    /// and returns it along with the location of its upper-left corner.
    /// 
    /// The lifespan of the glyph is independent from the lifespan of the contour collection.
    pub fn from_contour(outer_contour: Contour) -> (Self, Point2D<i32>) {
        let mut x_min = u32::MAX;
        let mut x_max = 0;
        let mut y_min = u32::MAX;
        let mut y_max = 0;
        
        for (x, y) in outer_contour.even_points() {
            if x < x_min { x_min = x; }
            if x > x_max { x_max = x; }
            if y < y_min { y_min = y; }
            if y > y_max { y_max = y; }
        }
        let width = x_max - x_min + 1;
        let height = y_max - y_min + 1;
        
        let location = Point2D::new(x_min, y_min).to_i32();
        let size = Size2D::new(width, height).to_i32();
        
        let contours = iter::once(outer_contour)
            .chain(outer_contour.children())
            .enumerate()
            .map(|(i, contour)| {
                let even_points = contour.even_points()
                    .map(|(x, y)| Point2D::new((x - x_min) as i32, (y - y_min) as i32))
                    .collect();
                GlyphContour { is_outer: i == 0, even_points }
            })
            .collect();
        
        (Self { size, contours }, location)
    }
    
    /// Width and height of the glyph’s bounding box.
    pub fn size(&self) -> Size2D<i32> {
        self.size
    }
    
    /// Contours of the glyph. The outer contour goes first, then, the holes.
    pub fn contours(&self) -> &[GlyphContour] {
        &self.contours[..]
    }
    
    /// Returns a reference to the outer contour of the glyph.
    pub fn outer_contour(&self) -> &GlyphContour {
        &self.contours[0]
    }
    
    /// Returns a slice of all hole contours of the glyph.
    pub fn inner_contours(&self) -> &[GlyphContour] {
        &self.contours[1..]
    }
}
