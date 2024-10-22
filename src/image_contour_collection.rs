mod row_changes;
mod run_changes;
mod feature_automaton;
mod table_builder;
mod contours;

use std::{iter, mem, collections::VecDeque};
use image::GrayImage;
use row_changes::RowChangeIter;
use run_changes::RunChangeIter;
use feature_automaton::{FeatureKind, Feature, FeatureAutomaton};
use table_builder::{TableItem, TableBuilder};
use contours::{ChildContourIter, DescendantContourIter};

pub struct ImageContourCollection {
    pub table: Vec<TableItem>,
}

impl ImageContourCollection {
    pub fn new(image: &GrayImage) -> Self {
        let (width, height) = image.dimensions();
        
        let mut feature_automaton = FeatureAutomaton::new();
        let mut table_builder = TableBuilder::new(width, height);
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
                        let new_index = table_builder.add_with_new_contour(x, y);
                        queue.push_back((new_index, new_index));
                        queue.push_back((new_index, new_index));
                    },
                    FeatureKind::Vertical => {
                        debug_assert!(queue.len() >= 1);
                        let (index, head) = queue.pop_front().unwrap();
                        queue.push_back((index, head));
                        table_builder.cross_contour(head);
                    },
                    FeatureKind::LeftShelf => {
                        debug_assert!(queue.len() >= 1);
                        let (to_index, head) = queue.pop_front().unwrap();
                        let new_index = table_builder.add_with_next(x, y, to_index);
                        table_builder.cross_contour(head);
                        queue.push_back((new_index, head));
                    },
                    FeatureKind::RightShelf => {
                        debug_assert!(queue.len() >= 1);
                        let (from_index, head) = queue.pop_front().unwrap();
                        let new_index = table_builder.add_with_previous(x, y, from_index);
                        table_builder.cross_contour(head);
                        queue.push_back((new_index, head));
                    },
                    FeatureKind::InnerFoot => {
                        debug_assert!(queue.len() >= 2);
                        let (from_index, from_head) = queue.pop_front().unwrap();
                        let (to_index, to_head) = queue.pop_front().unwrap();
                        table_builder.add_with_next_and_previous(x, y, to_index, from_index);
                        table_builder.combine_contours(to_head, from_head);
                    },
                    FeatureKind::OuterFoot => {
                        debug_assert!(queue.len() >= 2);
                        let (to_index, to_head) = queue.pop_front().unwrap();
                        let (from_index, from_head) = queue.pop_front().unwrap();
                        table_builder.add_with_next_and_previous(x, y, to_index, from_index);
                        table_builder.combine_contours(from_head, to_head);
                    },
                    FeatureKind::None => { }
                }
            }
        }
        debug_assert!(queue.is_empty());
        
        ImageContourCollection { table: table_builder.into() }
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        let root = &self.table[0];
        (root.x, root.y)
    }
    
    pub fn outermost_contours<'a>(&'a self) -> ChildContourIter<'a> {
        ChildContourIter::new(&self.table, 0, true)
    }
    
    pub fn non_hole_contours<'a>(&'a self) -> DescendantContourIter<'a> {
        DescendantContourIter::new(&self.table, 0, true, Some(false))
    }
    
    pub fn all_contours<'a>(&'a self) -> DescendantContourIter<'a> {
        DescendantContourIter::new(&self.table, 0, true, None)
    }
}
