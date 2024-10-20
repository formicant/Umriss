mod types;
mod runs;
mod state_machine;
mod hierarchy;

use std::{collections::VecDeque, iter, mem};
use image::{GrayImage, Luma};
use types::{Relation, ContourPoint, Feature};
use state_machine::StateMachine;
use hierarchy::Hierarchy;

struct QueueItem {
    point: usize,
    head: usize,
}



pub struct ImageContours {
    pub contour_points: Vec<ContourPoint>,
}

impl ImageContours {
    pub fn new<F>(image: &GrayImage, binarize: F) -> Self
        where  F: Fn(&Luma<u8>) -> bool {
        
        let (width, height) = image.dimensions();
        
        let root = ContourPoint { x: width, y: height, next: 0, relation: Relation::None };
        let mut contour_points = vec![root];
        
        let mut queue = VecDeque::<QueueItem>::new();
        let mut state_machine = StateMachine::new();
        let mut hierarchy = Hierarchy::new();
        
        let transition_max = width as usize + 2;
        let mut previous_transitions = Vec::with_capacity(transition_max);
        previous_transitions.push(u32::MAX);
        let mut current_transitions = Vec::with_capacity(transition_max);
        
        let last_row = GrayImage::new(width, 1);
        for (row_index, row) in image.rows().chain(last_row.rows()).enumerate() {
            let y = row_index as u32;
            
            find_transitions(&mut current_transitions, row, &binarize);
            let mut previous_transition_index = 0;
            let mut current_transition_index = 0;
            
            loop {
                let previous_x = previous_transitions[previous_transition_index];
                let current_x = current_transitions[current_transition_index];
                
                let (x, feature) = if current_x < previous_x {
                    current_transition_index += 1;
                    state_machine.step(std::cmp::Ordering::Less, current_x)
                } else if current_x > previous_x {
                    previous_transition_index += 1;
                    state_machine.step(std::cmp::Ordering::Greater, previous_x)
                } else if current_x != u32::MAX {
                    previous_transition_index += 1;
                    current_transition_index += 1;
                    state_machine.step(std::cmp::Ordering::Equal, current_x)
                } else {
                    break;
                };
                
                let new_point = contour_points.len();
                match feature {
                    Feature::Head => {
                        contour_points.push(ContourPoint::new(x, y));
                        queue.push_front(QueueItem { point: new_point, head: new_point });
                        queue.push_front(QueueItem { point: new_point, head: new_point });
                        hierarchy.add_contour(&mut contour_points, new_point);
                    },
                    Feature::Vertical => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point, head } = queue.pop_back().unwrap();
                        queue.push_front(QueueItem { point, head });
                        hierarchy.cross_contour(&contour_points, head);
                    },
                    Feature::LeftShelf => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point: to_point, head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::with_next(x, y, to_point));
                        queue.push_front(QueueItem { point: new_point, head });
                        hierarchy.cross_contour(&contour_points, head);
                    },
                    Feature::RightShelf => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point: from_point, head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::new(x, y));
                        contour_points[from_point].next = new_point;
                        queue.push_front(QueueItem { point: new_point, head });
                        hierarchy.cross_contour(&contour_points, head);
                    },
                    Feature::InnerFoot => {
                        debug_assert!(queue.len() >= 2);
                        let QueueItem { point: from_point, head: from_head } = queue.pop_back().unwrap();
                        let QueueItem { point: to_point, head: to_head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::with_next(x, y, to_point));
                        contour_points[from_point].next = new_point;
                        hierarchy.combine_contours(&mut contour_points, to_head, from_head);
                    },
                    Feature::OuterFoot => {
                        debug_assert!(queue.len() >= 2);
                        let QueueItem { point: to_point, head: to_head } = queue.pop_back().unwrap();
                        let QueueItem { point: from_point, head: from_head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint::with_next(x, y, to_point));
                        contour_points[from_point].next = new_point;
                        hierarchy.combine_contours(&mut contour_points, from_head, to_head);
                    },
                    Feature::None => { }
                }
            }
            
            mem::swap(&mut previous_transitions, &mut current_transitions);
        }
        
        debug_assert!(queue.is_empty());
        
        ImageContours { contour_points }
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        let root = &self.contour_points[0];
        (root.x, root.y)
    }
}

fn find_transitions<F>(transitions: &mut Vec<u32>, row: image::buffer::Pixels<'_, Luma<u8>>, binarize: F)
where  F: Fn(&Luma<u8>) -> bool {
    transitions.clear();
    let pixels = row.map(binarize).chain(iter::once(false));
    let mut previous_pixel = false;
    for (x, pixel) in pixels.enumerate() {
        if pixel != previous_pixel {
            transitions.push(x as u32);
        }
        previous_pixel = pixel;
    }
    transitions.push(u32::MAX);
}
