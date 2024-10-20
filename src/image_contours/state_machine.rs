use std::cmp::Ordering;
use super::types::Feature;

pub struct StateMachine {
    state: usize,
    x: u32,
}

impl StateMachine {
    pub fn new() -> Self {
        Self { state: 0, x: 0 }
    }
    
    pub fn step(&mut self, ordering: Ordering, new_x: u32) -> (u32, Feature) {
        let (new_state, update_x, feature) = match ordering {
            Ordering::Less => STEP_IF_LESS[self.state],
            Ordering::Equal => STEP_IF_EQUAL[self.state],
            Ordering::Greater => STEP_IF_GREATER[self.state],
        };
        self.state = new_state;
        if update_x {
            self.x = new_x;
        }
        (self.x, feature)
    }
}

const STEP_IF_GREATER: [(usize, bool, Feature); 6] = [
    (1, false, Feature::None),
    (0, true,  Feature::OuterFoot),
    (3, false, Feature::LeftShelf),
    (4, true,  Feature::None),
    (3, false, Feature::InnerFoot),
    (0, true,  Feature::RightShelf),
];
const STEP_IF_EQUAL: [(usize, bool, Feature); 6] = [
    (3, false, Feature::Vertical),
    (4, true,  Feature::LeftShelf),
    (5, false, Feature::LeftShelf),
    (0, false, Feature::Vertical),
    (5, false, Feature::InnerFoot),
    (4, true,  Feature::Head),
];
const STEP_IF_LESS: [(usize, bool, Feature); 6] = [
    (2, true,  Feature::None ),
    (3, true,  Feature::LeftShelf ),
    (0, false, Feature::Head ),
    (5, false, Feature::None ),
    (0, false, Feature::RightShelf ),
    (3, true,  Feature::Head ),
];
