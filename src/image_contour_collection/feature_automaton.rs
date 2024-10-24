use super::run_changes::{RunChangeKind, RunChange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    None,
    Head,
    Vertical,
    LeftShelf,
    RightShelf,
    InnerFoot,
    OuterFoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Feature {
    pub kind: FeatureKind,
    pub x: u32,
}

#[derive(Debug)]
pub struct FeatureAutomaton {
    state: usize,
    current_x: u32,
}

impl FeatureAutomaton {
    pub fn new() -> Self {
        Self { state: 0, current_x: 0 }
    }
    
    pub fn step(&mut self, change: RunChange) -> Feature {
        let (new_state, update_x, feature_kind) = match change.kind {
            RunChangeKind::Top => STEP_IF_TOP[self.state],
            RunChangeKind::Both => STEP_IF_BOTH[self.state],
            RunChangeKind::Bottom => STEP_IF_BOTTOM[self.state],
        };
        self.state = new_state;
        if update_x {
            self.current_x = change.x;
        }
        Feature { kind: feature_kind, x: self.current_x }
    }
}

const STEP_IF_TOP: [(usize, bool, FeatureKind); 6] = [
    (1, false, FeatureKind::None),
    (0, true,  FeatureKind::OuterFoot),
    (3, false, FeatureKind::LeftShelf),
    (4, true,  FeatureKind::None),
    (3, false, FeatureKind::InnerFoot),
    (0, true,  FeatureKind::RightShelf),
];
const STEP_IF_BOTH: [(usize, bool, FeatureKind); 6] = [
    (3, false, FeatureKind::Vertical),
    (4, true,  FeatureKind::LeftShelf),
    (5, false, FeatureKind::LeftShelf),
    (0, false, FeatureKind::Vertical),
    (5, false, FeatureKind::InnerFoot),
    (4, true,  FeatureKind::Head),
];
const STEP_IF_BOTTOM: [(usize, bool, FeatureKind); 6] = [
    (2, true,  FeatureKind::None ),
    (3, true,  FeatureKind::LeftShelf ),
    (0, false, FeatureKind::Head ),
    (5, false, FeatureKind::None ),
    (0, false, FeatureKind::RightShelf ),
    (3, true,  FeatureKind::Head ),
];
