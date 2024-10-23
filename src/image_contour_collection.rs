mod row_changes;
mod run_changes;
mod feature_automaton;
mod point_list_builder;
mod hierarchy_builder;
mod contours;

use std::{iter, mem, collections::VecDeque};
use hierarchy_builder::{HierarchyBuilder, HierarchyItem};
use image::GrayImage;
use row_changes::RowChangeIter;
use run_changes::RunChangeIter;
use feature_automaton::{FeatureKind, Feature, FeatureAutomaton};
use point_list_builder::{PointListItem, PointListBuilder};
use contours::{Contour, ChildContourIter, DescendantContourIter};

pub struct ImageContourCollection {
    width: u32,
    height: u32,
    hierarchy: Vec<HierarchyItem>,
    pub point_list: Vec<PointListItem>,
}

impl ImageContourCollection {
    pub fn new(image: &GrayImage) -> Self {
        let (width, height) = image.dimensions();
        
        let mut feature_automaton = FeatureAutomaton::new();
        let mut point_list_builder = PointListBuilder::new();
        let mut hierarchy_builder = HierarchyBuilder::new();
        let mut queue = VecDeque::new();
        
        let run_capacity = width as usize + 2;
        let mut run_top = Vec::with_capacity(run_capacity);
        let mut run_bottom = Vec::with_capacity(run_capacity);
        run_bottom.extend(RowChangeIter::empty());
        
        let rows = image.rows()
            .map(RowChangeIter::from)
            .chain(iter::once(RowChangeIter::empty()));
        
        for (row_index, row_changes) in rows.enumerate() {
            mem::swap(&mut run_top, &mut run_bottom);
            run_bottom.clear();
            run_bottom.extend(row_changes);

            let y = row_index as u32;
            for change in RunChangeIter::new(&run_top, &run_bottom) {
                let Feature { kind, x } = feature_automaton.step(change);
                match kind {
                    FeatureKind::Head => {
                        let new_index = point_list_builder.add(x, y);
                        let new_contour = hierarchy_builder.add_contour(new_index);
                        queue.push_back((new_index, new_contour));
                        queue.push_back((new_index, new_contour));
                    },
                    FeatureKind::Vertical => {
                        debug_assert!(queue.len() >= 1);
                        let (index, contour) = queue.pop_front().unwrap();
                        hierarchy_builder.cross_contour(contour);
                        queue.push_back((index, contour));
                    },
                    FeatureKind::LeftShelf => {
                        debug_assert!(queue.len() >= 1);
                        let (to_index, contour) = queue.pop_front().unwrap();
                        let new_index = point_list_builder.add_with_next(x, y, to_index);
                        hierarchy_builder.cross_contour(contour);
                        queue.push_back((new_index, contour));
                    },
                    FeatureKind::RightShelf => {
                        debug_assert!(queue.len() >= 1);
                        let (from_index, contour) = queue.pop_front().unwrap();
                        let new_index = point_list_builder.add_with_previous(x, y, from_index);
                        hierarchy_builder.cross_contour(contour);
                        queue.push_back((new_index, contour));
                    },
                    FeatureKind::InnerFoot => {
                        debug_assert!(queue.len() >= 2);
                        let (from_index, from_contour) = queue.pop_front().unwrap();
                        let (to_index, to_contour) = queue.pop_front().unwrap();
                        point_list_builder.add_with_next_and_previous(x, y, to_index, from_index);
                        hierarchy_builder.combine_contours(to_contour, from_contour);
                    },
                    FeatureKind::OuterFoot => {
                        debug_assert!(queue.len() >= 2);
                        let (to_index, to_contour) = queue.pop_front().unwrap();
                        let (from_index, from_contour) = queue.pop_front().unwrap();
                        point_list_builder.add_with_next_and_previous(x, y, to_index, from_index);
                        hierarchy_builder.combine_contours(from_contour, to_contour);
                    },
                    FeatureKind::None => { }
                }
            }
        }
        debug_assert!(queue.is_empty());
        
        let point_list = point_list_builder.into();
        let hierarchy = hierarchy_builder.into();
        ImageContourCollection { width, height, hierarchy, point_list }
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    pub fn outermost_contours<'a>(&'a self) -> ChildContourIter<'a> {
        ChildContourIter::new(&self.hierarchy, &self.point_list, 0, true)
    }
    
    pub fn non_hole_contours<'a>(&'a self) -> impl Iterator<Item = Contour<'a>> {
        self.all_contours().filter(|contour| !contour.is_hole())
    }
    
    pub fn all_contours<'a>(&'a self) -> DescendantContourIter<'a> {
        DescendantContourIter::new(&self.hierarchy, &self.point_list, 0, true)
    }
}


// ---------

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use test_case::test_case;
    use super::*;
    use super::hierarchy_builder::NONE;
    
    #[test_case(1, 1, vec![0], vec![], vec![root(None)])]
    #[test_case(
        1, 1, vec![1],
        vec![PointListItem { x: 0, y: 0, next: 1 }, PointListItem { x: 1, y: 1, next: 0 }],
        vec![root(NonZeroUsize::new(1)), hier(0, 0, None, None)]
    )]
    #[test_case(
        3, 4,
        vec![
            1, 1, 1,
            1, 0, 1,
            1, 0, 0,
            0, 0, 1,
        ],
        vec![
            PointListItem { x: 0, y: 0, next: 2 },
            PointListItem { x: 2, y: 1, next: 3 },
            PointListItem { x: 3, y: 2, next: 1 },
            PointListItem { x: 1, y: 3, next: 0 },
            PointListItem { x: 2, y: 3, next: 5 },
            PointListItem { x: 3, y: 4, next: 4 },
        ],
        vec![
            root(NonZeroUsize::new(2)),
            hier(0, 0, None, None),
            hier(4, 0, NonZeroUsize::new(1), None),
        ]
    )]
    fn pixel_row(width: u32, height: u32, image_pixels: Vec<u8>,
        expected_point_list: Vec<PointListItem>, expected_hierarchy: Vec<HierarchyItem>
    ) {
        let image = GrayImage::from_vec(width, height, image_pixels).unwrap();
        let actual = ImageContourCollection::new(&image);
        assert_eq!(actual.dimensions(), (width, height));
        assert_eq!(actual.point_list, expected_point_list);
        assert_eq!(actual.hierarchy, expected_hierarchy);
    }
    
    const fn root(first_child: Option<NonZeroUsize>) -> HierarchyItem {
        HierarchyItem { head_point: NONE, parent: NONE, next_sibling: None, first_child }
    }
    
    const fn hier(head_point: usize, parent: usize, next_sibling: Option<NonZeroUsize>, first_child: Option<NonZeroUsize>) -> HierarchyItem {
        HierarchyItem { head_point, parent, next_sibling, first_child }
    }
}
