mod row_changes;
mod run_changes;
mod table_builder;
mod feature_automaton;

use std::{iter, mem, collections::VecDeque};
use image::GrayImage;
use table_builder::{Relation, TableBuilder, TableItem};
use row_changes::RowChanges;
use run_changes::RunChanges;
use feature_automaton::{Feature, FeatureAutomaton, FeatureKind};

pub struct ImageContours {
    pub table: Vec<TableItem>,
}

impl ImageContours {
    pub fn new(image: &GrayImage) -> Self {
        let (width, height) = image.dimensions();
        
        let mut table_builder = TableBuilder::new(width, height);
        let mut queue = VecDeque::new();
        let mut feature_automaton = FeatureAutomaton::new();
        
        let run_capacity = width as usize + 2;
        let mut run_top = Vec::with_capacity(run_capacity);
        let mut run_bottom = Vec::with_capacity(run_capacity);
        run_bottom.extend(RowChanges::empty());
        
        let rows = image.rows()
            .map(|row| RowChanges::from(row))
            .chain(iter::once(RowChanges::empty()));
        
        for (row_index, row_changes) in rows.enumerate() {
            mem::swap(&mut run_top, &mut run_bottom);
            run_bottom.clear();
            run_bottom.extend(row_changes);

            let y = row_index as u32;
            for change in RunChanges::new(&run_top, &run_bottom) {
                let Feature { kind, x } = feature_automaton.step(change);
                match kind {
                    FeatureKind::Head => {
                        let new_index = table_builder.add_with_contour(x, y);
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
        
        ImageContours { table: table_builder.into() }
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        let root = &self.table[0];
        (root.x, root.y)
    }
    
    pub fn outermost_contours<'a>(&'a self) -> SiblingContours<'a> {
        if let Relation::Child(first_child) = self.table[0].relation {
            SiblingContours { table: &self.table, current_index: Some(first_child) }
        } else {
            SiblingContours { table: &self.table, current_index: None }
        }
    }
}

pub struct SiblingContours<'a> {
    table: &'a[TableItem],
    current_index: Option<usize>,
}

impl<'a> Iterator for SiblingContours<'a> {
    type Item = Contour<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_index {
            Some(index) => {
                let point = &self.table[index];
                let next_point = &self.table[point.next];
                self.current_index = if let Relation::Sibling(sibling) = next_point.relation {
                    Some(sibling)
                } else {
                    None
                };
                Some(Contour { table: self.table, start_index: index })
            },
            None => None,
        }
    }
}

pub struct Contour<'a> {
    table: &'a[TableItem],
    start_index: usize,
}

impl<'a> Contour<'a> {
    pub fn control_points(&self) -> ControlPoints<'a> {
        ControlPoints { table: self.table, start_index: self.start_index, current_index: Some(self.start_index) }
    }
}

pub struct ControlPoints<'a> {
    table: &'a[TableItem],
    start_index: usize,
    current_index: Option<usize>,
}

impl<'a> Iterator for ControlPoints<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_index {
            Some(index) => {
                let point = &self.table[index];
                self.current_index = if point.next != self.start_index {
                    Some(point.next)
                } else {
                    None
                };
                Some((point.x, point.y))
            },
            None => None,
        }
    }
}

