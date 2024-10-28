use std::collections::VecDeque;
use super::ImageContourCollection;
use super::row_pair_changes::RowPairChange;
use super::feature_automaton::{FeatureKind, Feature, FeatureAutomaton};
use super::hierarchy_builder::HierarchyBuilder;
use super::point_list_builder::PointListBuilder;

/// Builds the point list and contour hierarchy
/// from changes in image rows pairs.
pub struct ContourCollectionBuilder {
    // Original image dimensions
    width: u32,
    height: u32,
    
    // Helper builders
    point_list: PointListBuilder,
    hierarchy: HierarchyBuilder,
    feature_automaton: FeatureAutomaton,
    
    // The queue stores open ends of the contours until they are closed.
    // It takes less space than the `w_link` pointers suggested by Miyatake.
    // Each queue element is a tuple of the point index in the `point_list`
    // and the contour index in the `hierarchy`
    queue: VecDeque<(usize,usize)>,
}

impl ContourCollectionBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width, height,
            feature_automaton: FeatureAutomaton::new(),
            point_list: PointListBuilder::new(),
            hierarchy: HierarchyBuilder::new(),
            queue: VecDeque::new(),
        }
    }
    
    pub fn into(self) -> ImageContourCollection {
        assert!(self.queue.is_empty(), "Queue left non-empty");
        ImageContourCollection {
            width: self.width, height: self.height,
            hierarchy: self.hierarchy.into(),
            point_list: self.point_list.into()
        }
    }
    
    pub fn add_row_pair_change(&mut self, y: u32, change: RowPairChange) {
        // Detect next contour feature
        let Feature { kind, x } = self.feature_automaton.step(change);
        
        match kind {
            FeatureKind::Head => {
                // A `Head` starts a new contour.
                // Add its two open ends to the queue
                let new_index = self.point_list.add(x, y);
                let new_contour = self.hierarchy.add_contour(new_index);
                self.queue.push_back((new_index, new_contour));
                self.queue.push_back((new_index, new_contour));
            },
            FeatureKind::Vertical => {
                // A `Vertical` should not be added to the point list.
                // Take one open end and place back in the queue untouched.
                // We cross a contour boundary
                debug_assert!(self.queue.len() >= 1);
                let (index, contour) = self.queue.pop_front().unwrap();
                self.hierarchy.cross_contour(contour);
                self.queue.push_back((index, contour));
            },
            FeatureKind::LeftShelf => {
                // Connect a `Shelf` to the current open end.
                // Add its new open end to the queue.
                // We cross a contour boundary
                debug_assert!(self.queue.len() >= 1);
                let (to_index, contour) = self.queue.pop_front().unwrap();
                let new_index = self.point_list.add_with_next(x, y, to_index);
                self.hierarchy.cross_contour(contour);
                self.queue.push_back((new_index, contour));
            },
            FeatureKind::RightShelf => {
                // Same, but the connection order is reversed
                debug_assert!(self.queue.len() >= 1);
                let (from_index, contour) = self.queue.pop_front().unwrap();
                let new_index = self.point_list.add_with_previous(x, y, from_index);
                self.hierarchy.cross_contour(contour);
                self.queue.push_back((new_index, contour));
            },
            FeatureKind::InnerFoot => {
                // A `Foot` connects two open ends from the queue.
                // If they belonged to separate contours, they should be merged
                debug_assert!(self.queue.len() >= 2);
                let (from_index, from_contour) = self.queue.pop_front().unwrap();
                let (to_index, to_contour) = self.queue.pop_front().unwrap();
                self.point_list.add_with_next_and_previous(x, y, to_index, from_index);
                self.hierarchy.merge_contours(to_contour, from_contour);
            },
            FeatureKind::OuterFoot => {
                // Same, but the connection order is reversed
                debug_assert!(self.queue.len() >= 2);
                let (to_index, to_contour) = self.queue.pop_front().unwrap();
                let (from_index, from_contour) = self.queue.pop_front().unwrap();
                self.point_list.add_with_next_and_previous(x, y, to_index, from_index);
                self.hierarchy.merge_contours(from_contour, to_contour);
            },
            FeatureKind::None => { } // Ignore
        }
    }
}
