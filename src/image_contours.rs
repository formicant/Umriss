use std::{collections::VecDeque, fmt::Debug, iter, mem};
use image::{GrayImage, Luma};

#[derive(Debug)]
pub enum Relation {
    None,
    Alias(usize),
    Parent(usize),
    Child(usize),
    Sibling(usize),
}

pub struct ContourPoint {
    x: u32,
    y: u32,
    next: usize,
    relation: Relation,
}

impl std::fmt::Debug for ContourPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:4}, {:4}) {:5} {:?}", self.x, self.y, self.next, self.relation)
    }
}

struct QueueItem {
    point: usize,
    head: usize,
}

enum Feature {
    None,
    Head,
    Vertical,
    LeftShelf,
    RightShelf,
    InnerFoot,
    OuterFoot,
}

struct Move {
    state: usize,
    update_x: bool,
    feature: Feature,
}

const MOVE_IF_GREATER: [Move; 6] = [
    Move { state: 1, update_x: false, feature: Feature::None },
    Move { state: 0, update_x: true,  feature: Feature::OuterFoot },
    Move { state: 3, update_x: false, feature: Feature::LeftShelf },
    Move { state: 4, update_x: true,  feature: Feature::None },
    Move { state: 3, update_x: false, feature: Feature::InnerFoot },
    Move { state: 0, update_x: true,  feature: Feature::RightShelf },    
];
const MOVE_IF_EQUAL: [Move; 6] = [
    Move { state: 3, update_x: false, feature: Feature::Vertical },
    Move { state: 4, update_x: true,  feature: Feature::LeftShelf },
    Move { state: 5, update_x: false, feature: Feature::LeftShelf },
    Move { state: 0, update_x: false, feature: Feature::Vertical },
    Move { state: 5, update_x: false, feature: Feature::InnerFoot },
    Move { state: 4, update_x: true,  feature: Feature::Head },
];
const MOVE_IF_LESS: [Move; 6] = [
    Move { state: 2, update_x: true,  feature: Feature::None },
    Move { state: 3, update_x: true,  feature: Feature::LeftShelf },
    Move { state: 0, update_x: false, feature: Feature::Head },
    Move { state: 5, update_x: false, feature: Feature::None },
    Move { state: 0, update_x: false, feature: Feature::RightShelf },
    Move { state: 3, update_x: true,  feature: Feature::Head },
];


pub struct ImageContours {
    pub contour_points: Vec<ContourPoint>,
}

impl ImageContours {
    pub fn new(image: &GrayImage) -> Self {
        let (width, height) = image.dimensions();
        
        let root = ContourPoint { x: width, y: height, next: 0, relation: Relation::None };
        let mut contour_points = vec![root];
        
        let mut queue = VecDeque::<QueueItem>::new();
        let mut current_contour = 0;
        let mut state = 0;
        
        let transition_max = width as usize + 2;
        let mut previous_transitions = Vec::with_capacity(transition_max);
        previous_transitions.push(u32::MAX);
        let mut current_transitions = Vec::with_capacity(transition_max);
        
        let last_row = GrayImage::new(width, 1);
        for (row_index, row) in image.rows().chain(last_row.rows()).enumerate() {
            let y = row_index as u32;
            
            find_transitions(&mut current_transitions, row);
            let mut previous_transition_index = 0;
            let mut current_transition_index = 0;
            
            let mut x = 0;
            loop {
                let previous_x = previous_transitions[previous_transition_index];
                let current_x = current_transitions[current_transition_index];
                
                let (new_x, Move { state: new_state, update_x, feature }) = if current_x < previous_x {
                    current_transition_index += 1;
                    (current_x, &MOVE_IF_LESS[state])
                } else if current_x > previous_x {
                    previous_transition_index += 1;
                    (previous_x, &MOVE_IF_GREATER[state])
                } else if current_x != u32::MAX {
                    previous_transition_index += 1;
                    current_transition_index += 1;
                    (current_x, &MOVE_IF_EQUAL[state])
                } else {
                    break;
                };
                state = *new_state;
                if *update_x {
                    x = new_x;
                }
                
                let new_point = contour_points.len();
                match feature {
                    Feature::Head => {
                        contour_points.push(ContourPoint { x, y, next: 0, relation: Relation::Parent(current_contour) });
                        queue.push_front(QueueItem { point: new_point, head: new_point });
                        queue.push_front(QueueItem { point: new_point, head: new_point });
                    },
                    Feature::Vertical => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point, head } = queue.pop_back().unwrap();
                        queue.push_front(QueueItem { point, head });
                        current_contour = cross_contour(&contour_points, current_contour, head);
                    },
                    Feature::LeftShelf => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point: to_point, head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint { x, y, next: to_point, relation: Relation::None });
                        queue.push_front(QueueItem { point: new_point, head });
                        current_contour = cross_contour(&contour_points, current_contour, head);
                    },
                    Feature::RightShelf => {
                        debug_assert!(!queue.is_empty());
                        let QueueItem { point: from_point, head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint { x, y, next: 0, relation: Relation::None });
                        contour_points[from_point].next = new_point;
                        queue.push_front(QueueItem { point: new_point, head });
                        current_contour = cross_contour(&contour_points, current_contour, head);
                    },
                    Feature::InnerFoot => {
                        debug_assert!(queue.len() >= 2);
                        let QueueItem { point: from_point, head: from_head } = queue.pop_back().unwrap();
                        let QueueItem { point: to_point, head: to_head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint { x, y, next: to_point, relation: Relation::None });
                        contour_points[from_point].next = new_point;
                        current_contour = combine_contours(&mut contour_points, current_contour, to_head, from_head);
                    },
                    Feature::OuterFoot => {
                        debug_assert!(queue.len() >= 2);
                        let QueueItem { point: to_point, head: to_head } = queue.pop_back().unwrap();
                        let QueueItem { point: from_point, head: from_head } = queue.pop_back().unwrap();
                        contour_points.push(ContourPoint { x, y, next: to_point, relation: Relation::None });
                        contour_points[from_point].next = new_point;
                        current_contour = combine_contours(&mut contour_points, current_contour, from_head, to_head);
                    },
                    Feature::None => { }
                }
            }
            
            mem::swap(&mut previous_transitions, &mut current_transitions);
        }
        
        ImageContours { contour_points }
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        let root = &self.contour_points[0];
        (root.x, root.y)
    }
}

fn find_transitions(transitions: &mut Vec<u32>, row: image::buffer::Pixels<'_, Luma<u8>>) {
    transitions.clear();
    let pixels = row.map(|pixel| pixel[0] >= 128).chain(iter::once(false));
    let mut previous_pixel = false;
    for (x, pixel) in pixels.enumerate() {
        if pixel != previous_pixel {
            transitions.push(x as u32);
        }
        previous_pixel = pixel;
    }
    transitions.push(u32::MAX);
}

fn unalias(contour_points: &[ContourPoint], point: usize) -> usize {
    let mut index = point;
    while let Relation::Alias(head) = contour_points[index].relation {
        index = head;
    }
    index
}

fn cross_contour(contour_points: &[ContourPoint], current_contour: usize, head: usize) -> usize {
    let index = unalias(contour_points, head);
    if current_contour == index {
        if let Relation::Parent(parent) = contour_points[current_contour].relation {
            return unalias(contour_points, parent);
        } else {
            panic!();
        }
    } else {
        index
    }
}

fn combine_contours(contour_points: &mut [ContourPoint], current_contour: usize, from_head: usize, to_head: usize) -> usize {
    let mut from_index = unalias(contour_points, from_head);
    let mut to_index = unalias(contour_points, to_head);
    if from_index != to_index {
        if from_index < to_index {
            (from_index, to_index) = (to_index, from_index);
        }
        contour_points[from_index].relation = Relation::Alias(to_index);
        if current_contour == from_index {
            return to_index;
        }
    }
    current_contour
}
