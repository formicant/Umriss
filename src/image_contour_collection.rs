mod row_changes;
mod row_pair_changes;
mod contour_collection_builder;
mod feature_automaton;
mod point_list_builder;
mod hierarchy_builder;
mod contours;
#[cfg(test)] mod tests;

use std::iter;
use image::GrayImage;
use contour_collection_builder::ContourCollectionBuilder;
use hierarchy_builder::HierarchyItem;
use point_list_builder::PointListItem;
use row_changes::RowChangeIter;
use row_pair_changes::RowPairChangeIter;
use contours::{ChildContourIter, DescendantContourIter};
pub use contours::Contour;

/// Finds contours in a binary image.
/// 
/// Implements a modified version of the Miyatake’s algorithm[^1].
/// 
/// The contour points correspond to the pixel corners, not the centers.
/// The upper-left corner of the pixel with coordinates (x, y) has coordinates (x, y),
/// and its lower-right corner has coordinates  (x + 1, y + 1).
/// 
/// The outer contours are 8-connected and clockwise.
/// The inner contours (holes) are 4-connected and anti-clockwise.
/// 
/// [^1]: Takafumi Miyatake, Hitoshi Matsushima, Masakazu Ejiri, 1997:
/// _Contour representation of binary images using run-type direction codes_.
pub struct ImageContourCollection {
    width: i32,
    height: i32,
    hierarchy: Vec<HierarchyItem>,
    pub point_list: Vec<PointListItem>,
}

impl ImageContourCollection {
    /// An alias for `new(image, inverted: false)`.
    /// Assumes black background and white foreground.
    pub fn white_on_black(image: &GrayImage) -> Self {
        Self::new(image, false)
    }
    
    /// An alias for `new(image, inverted: true)`.
    /// Assumes white background and black foreground.
    pub fn black_on_white(image: &GrayImage) -> Self {
        Self::new(image, true)
    }
    
    /// Creates a new instance of `ImageContourCollection`
    /// containing contours of the given `image`.
    /// 
    /// Lifetime of the instance is independent from the `image` lifetime.
    /// 
    /// Although `image` is an 8-bit grayscale,
    /// the algorithm considers all non-zero pixel values as white.
    /// So the image should be binarized beforehand.
    /// 
    /// If `inverted` is `true`, black pixels will be considered as foreground
    /// instead of white ones.
    pub fn new(image: &GrayImage, inverted: bool) -> Self {
        let (width, height) = image.dimensions();
        let mut builder = ContourCollectionBuilder::new(width as i32, height as i32);
        
        // Row changes are stored in two buffers of fixed capacity to avoid allocation
        let capacity = width as usize + 2;
        let mut top_changes = Vec::with_capacity(capacity);
        let mut bottom_changes = Vec::with_capacity(capacity);
        
        // Add padding rows to the top and the bottom of the image
        bottom_changes.extend(RowChangeIter::empty());
        let rows = image.rows()
            .map(|row| RowChangeIter::from(row, inverted))
            .chain(iter::once(RowChangeIter::empty()));
        
        // Scan the image row by row, top to bottom
        for (row_index, row_changes) in rows.enumerate() {
            // Swap the buffers so the old bottom row becomes the new top row.
            // Fill new bottom row with new row changes
            (top_changes, bottom_changes) = (bottom_changes, top_changes);
            bottom_changes.clear();
            bottom_changes.extend(row_changes);
            
            // Look for changes in two adjacent image rows
            let row_pair_changes = RowPairChangeIter::new(&top_changes, &bottom_changes);
            
            for change in row_pair_changes {
                // Update the point list and contour hierarchy
                builder.add_row_pair_change(row_index as i32, change);
            }
        }
        
        builder.into()
    }
    
    /// Gets width and height of the original image.
    pub fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }
    
    /// Iterates only the top-level contours without any descendants.
    pub fn outermost_contours<'a>(&'a self) -> ChildContourIter<'a> {
        ChildContourIter::new(&self.hierarchy, &self.point_list, 0, false)
    }
    
    /// Iterates all outer contours without their inner children (holes),
    /// but with outer grandchildren, great-great-grandchildren, and so on.
    pub fn outer_contours<'a>(&'a self) -> impl Iterator<Item = Contour<'a>> {
        self.all_contours().filter(|contour| contour.is_outer())
    }
    
    /// Iterates all the contours, outer or inner, depth-first.
    pub fn all_contours<'a>(&'a self) -> DescendantContourIter<'a> {
        DescendantContourIter::new(&self.hierarchy, &self.point_list, 0, false)
    }
}
