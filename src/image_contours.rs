mod types;
mod row_changes;
mod run_changes;
mod feature_automaton;
mod hierarchy;

use std::{iter, mem, collections::VecDeque};
use image::GrayImage;
use types::{ContourPoint, Relation};
use row_changes::RowChanges;
use run_changes::RunChanges;
use feature_automaton::{FeatureKind, FeatureAutomaton};
use hierarchy::Hierarchy;

#[derive(Copy, Clone, PartialEq)]
struct QueueItem {
    point: usize,
    head: usize,
}

pub struct ImageContours {
    pub contour_points: Vec<ContourPoint>,
}

impl ImageContours {
    pub fn new(image: &GrayImage) -> Self {
        let (width, height) = image.dimensions();
        let root = ContourPoint::new(width, height);
        let mut contour_points = vec![root];
        
        let mut queue = VecDeque::new();
        let mut feature_detector = FeatureAutomaton::new();
        let mut hierarchy = Hierarchy::new();
        
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
                let new_point = contour_points.len();
                let feature = feature_detector.step(change);
                let x = feature.x;
                match feature.kind {
                    FeatureKind::Head => {
                        contour_points.push(ContourPoint::new(x, y));
                        queue.push_front(QueueItem { point: new_point, head: new_point });
                        queue.push_front(QueueItem { point: new_point, head: new_point });
                        hierarchy.add_contour(&mut contour_points, new_point);
                    },
                    FeatureKind::Vertical => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point, head } = queue.pop_back().unwrap();
                        queue.push_front(QueueItem { point, head });
                        hierarchy.cross_contour(&contour_points, head);
                    },
                    FeatureKind::LeftShelf => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point: to_point, head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::with_next(x, y, to_point));
                        queue.push_front(QueueItem { point: new_point, head });
                        hierarchy.cross_contour(&contour_points, head);
                    },
                    FeatureKind::RightShelf => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point: from_point, head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::new(x, y));
                        contour_points[from_point].next = new_point;
                        queue.push_front(QueueItem { point: new_point, head });
                        hierarchy.cross_contour(&contour_points, head);
                    },
                    FeatureKind::InnerFoot => {
                        debug_assert!(queue.len() >= 2);
                        let QueueItem { point: from_point, head: from_head } = queue.pop_back().unwrap();
                        let QueueItem { point: to_point, head: to_head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::with_next(x, y, to_point));
                        contour_points[from_point].next = new_point;
                        hierarchy.combine_contours(&mut contour_points, to_head, from_head);
                    },
                    FeatureKind::OuterFoot => {
                        debug_assert!(queue.len() >= 2);
                        let QueueItem { point: to_point, head: to_head } = queue.pop_back().unwrap();
                        let QueueItem { point: from_point, head: from_head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::with_next(x, y, to_point));
                        contour_points[from_point].next = new_point;
                        hierarchy.combine_contours(&mut contour_points, from_head, to_head);
                    },
                    FeatureKind::None => { }
                }
            }
        }
        debug_assert!(queue.is_empty());
        
        Hierarchy::parents_to_children(&mut contour_points);
        ImageContours { contour_points }
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        let root = &self.contour_points[0];
        (root.x, root.y)
    }
    
    pub fn outermost_contours<'a>(&'a self) -> SiblingContours<'a> {
        if let Relation::Child(first_child) = self.contour_points[0].relation {
            SiblingContours { contour_points: &self.contour_points, current_index: Some(first_child) }
        } else {
            SiblingContours { contour_points: &self.contour_points, current_index: None }
        }
    }
}

pub struct SiblingContours<'a> {
    contour_points: &'a[ContourPoint],
    current_index: Option<usize>,
}

impl<'a> Iterator for SiblingContours<'a> {
    type Item = Contour<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_index {
            Some(index) => {
                let point = &self.contour_points[index];
                let next_point = &self.contour_points[point.next];
                self.current_index = if let Relation::Sibling(sibling) = next_point.relation {
                    Some(sibling)
                } else {
                    None
                };
                Some(Contour { contour_points: self.contour_points, start_index: index })
            },
            None => None,
        }
    }
}

pub struct Contour<'a> {
    contour_points: &'a[ContourPoint],
    start_index: usize,
}

impl<'a> Contour<'a> {
    pub fn control_points(&self) -> ControlPoints<'a> {
        ControlPoints { contour_points: self.contour_points, start_index: self.start_index, current_index: Some(self.start_index) }
    }
}

pub struct ControlPoints<'a> {
    contour_points: &'a[ContourPoint],
    start_index: usize,
    current_index: Option<usize>,
}

impl<'a> Iterator for ControlPoints<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_index {
            Some(index) => {
                let point = &self.contour_points[index];
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

