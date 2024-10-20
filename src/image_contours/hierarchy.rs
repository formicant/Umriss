use super::types::{Relation, ContourPoint};

pub struct Hierarchy {
    current_contour: usize,
}

impl Hierarchy {
    pub fn new() -> Self {
        Hierarchy { current_contour: 0 }
    }
    
    pub fn add_contour(&self, contour_points: &mut [ContourPoint], head: usize) {
        contour_points[head].relation = Relation::Parent(self.current_contour);
    }
    
    pub fn cross_contour(&mut self, contour_points: &[ContourPoint], head: usize) {
        let index = unalias(contour_points, head);
        if self.current_contour == index {
            if let Relation::Parent(parent) = contour_points[self.current_contour].relation {
                self.current_contour = unalias(contour_points, parent);
            } else {
                panic!();
            }
        } else {
            self.current_contour = index;
        }
    }
    
    pub fn combine_contours(&mut self, contour_points: &mut [ContourPoint], from_head: usize, to_head: usize) {
        let mut from_index = unalias(contour_points, from_head);
        let mut to_index = unalias(contour_points, to_head);
        if from_index != to_index {
            if from_index < to_index {
                (from_index, to_index) = (to_index, from_index);
            }
            contour_points[from_index].relation = Relation::Alias(to_index);
            if self.current_contour == from_index {
                self.current_contour = to_index;
            }
        }
    }
}

fn unalias(contour_points: &[ContourPoint], point: usize) -> usize {
    let mut index = point;
    while let Relation::Alias(head) = contour_points[index].relation {
        index = head;
    }
    index
}

