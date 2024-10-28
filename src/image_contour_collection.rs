mod row_changes;
mod row_pair_changes;
mod feature_automaton;
mod point_list_builder;
mod hierarchy_builder;
mod contours;
#[cfg(test)] mod tests;

use std::{iter, collections::VecDeque};
use hierarchy_builder::{HierarchyBuilder, HierarchyItem};
use image::GrayImage;
use row_changes::RowChangeIter;
use row_pair_changes::RowPairChangeIter;
use feature_automaton::{FeatureKind, Feature, FeatureAutomaton};
use point_list_builder::{PointListItem, PointListBuilder};
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
    width: u32,
    height: u32,
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
        
        // Helpers
        let mut feature_automaton = FeatureAutomaton::new();
        let mut point_list_builder = PointListBuilder::new();
        let mut hierarchy_builder = HierarchyBuilder::new();
        
        // The queue stores open ends of the contours until they are closed.
        // It takes less space than the `w_link` pointers suggested by Miyatake.
        // Each queue element is a tuple of the point index in the `point_list`
        // and the contour index in the `hierarchy`
        let mut queue = VecDeque::new();
        
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
            
            let y = row_index as u32;
            for change in RowPairChangeIter::new(&top_changes, &bottom_changes) {
                // Detect next contour feature
                let Feature { kind, x } = feature_automaton.step(change);
                match kind {
                    FeatureKind::Head => {
                        // A `Head` starts a new contour.
                        // Add its two open ends to the queue
                        let new_index = point_list_builder.add(x, y);
                        let new_contour = hierarchy_builder.add_contour(new_index);
                        queue.push_back((new_index, new_contour));
                        queue.push_back((new_index, new_contour));
                    },
                    FeatureKind::Vertical => {
                        // A `Vertical` is not added to the point list.
                        // Take one open end and place back in the queue untouched.
                        // We cross a contour boundary
                        debug_assert!(queue.len() >= 1);
                        let (index, contour) = queue.pop_front().unwrap();
                        hierarchy_builder.cross_contour(contour);
                        queue.push_back((index, contour));
                    },
                    FeatureKind::LeftShelf => {
                        // Connect a `Shelf` to the current open end.
                        // Add its new open end to the queue.
                        // We cross a contour boundary
                        debug_assert!(queue.len() >= 1);
                        let (to_index, contour) = queue.pop_front().unwrap();
                        let new_index = point_list_builder.add_with_next(x, y, to_index);
                        hierarchy_builder.cross_contour(contour);
                        queue.push_back((new_index, contour));
                    },
                    FeatureKind::RightShelf => {
                        // Same, but the connection order is reversed
                        debug_assert!(queue.len() >= 1);
                        let (from_index, contour) = queue.pop_front().unwrap();
                        let new_index = point_list_builder.add_with_previous(x, y, from_index);
                        hierarchy_builder.cross_contour(contour);
                        queue.push_back((new_index, contour));
                    },
                    FeatureKind::InnerFoot => {
                        // A `Foot` connects two open ends from the queue.
                        // If they belonged to separate contours, they should be merged
                        debug_assert!(queue.len() >= 2);
                        let (from_index, from_contour) = queue.pop_front().unwrap();
                        let (to_index, to_contour) = queue.pop_front().unwrap();
                        point_list_builder.add_with_next_and_previous(x, y, to_index, from_index);
                        hierarchy_builder.merge_contours(to_contour, from_contour);
                    },
                    FeatureKind::OuterFoot => {
                        // Same, but the connection order is reversed
                        debug_assert!(queue.len() >= 2);
                        let (to_index, to_contour) = queue.pop_front().unwrap();
                        let (from_index, from_contour) = queue.pop_front().unwrap();
                        point_list_builder.add_with_next_and_previous(x, y, to_index, from_index);
                        hierarchy_builder.merge_contours(from_contour, to_contour);
                    },
                    FeatureKind::None => { } // Ignore
                }
            }
        }
        assert!(queue.is_empty(), "Queue left non-empty");
        
        let point_list = point_list_builder.into();
        let hierarchy = hierarchy_builder.into();
        ImageContourCollection { width, height, hierarchy, point_list }
    }
    
    /// Gets width and height of the original image.
    pub fn dimensions(&self) -> (u32, u32) {
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
