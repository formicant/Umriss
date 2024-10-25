use super::row_pair_changes::{RowPairChangeKind, RowPairChange};

/// Contour feature kinds:
/// ```
///               Head        Head
///               ●───┐       ●───────┐
///               │   │       │       │
/// LeftShelf ●───┘   ●───────┘       ├ Vertical
///           │       InnerFoot       │
///  Vertical ┤                   ┌───● RightShelf
///           │        Head       │ 
/// LeftShelf └───●   ┌───●       ●───┐ RightShelf
///               │   │   │           │
///               └───●   └───────────●
///           OuterFoot       OuterFoot
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    None,       // Miyatake’s RD-code notation:
    Head,       //  (1) or (9)
    Vertical,   //  (2) or (6)
    LeftShelf,  //  (3) or (4)
    RightShelf, //  (7) or (8)
    InnerFoot,  //  (10)
    OuterFoot,  //  (5)
}

/// Kind and x coordinate of a contour feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Feature {
    pub kind: FeatureKind,
    pub x: u32,
}

/// A final-state automaton that takes row pair changes as input
/// and returns contour features as output.
#[derive(Debug)]
pub struct FeatureAutomaton {
    state: usize,
    current_x: u32,
}

impl FeatureAutomaton {
    pub fn new() -> Self {
        Self { state: 0, current_x: 0 }
    }
    
    pub fn step(&mut self, change: RowPairChange) -> Feature {
        let (new_state, update_x, feature_kind) = match change.kind {
            RowPairChangeKind::Top => STEP_IF_TOP[self.state],
            RowPairChangeKind::Both => STEP_IF_BOTH[self.state],
            RowPairChangeKind::Bottom => STEP_IF_BOTTOM[self.state],
        };
        self.state = new_state;
        if update_x {
            self.current_x = change.x;
        }
        Feature { kind: feature_kind, x: self.current_x }
    }
}

// Automaton transition graph:
//          ╭────────────────↓RS───────────────╮
//          │        ╭───────╮             ╭───┴───╮
//          │ ╭─↑xOF─┤ 1│▒█│ ├────↕xLS────►│ 4│█ │ ├─↕IF─╮
//          │ │ ╭─↑─►│  │  │ ├─↓xLS╮ ╭─↑x─►│  │▒█│ │◄──╮ │
//        ╭─▼─▼─┴─╮  ╰───────╯  ╭──▼─┴──╮  ╰───┬───╯   │ │
// START─►│ 0│▒ │ ├─────↕V─────►│ 3│▒█│ │◄─↑IF─╯       │ │
// ◄─END──┤  │▒ │ │◄────↕V──────┤  │▒█│ │◄─↓xH─╮       │ │
//        ╰─▲─▲─┬─╯  ╭───────╮  ╰──▲─┬──╯  ╭───┴───╮   │ │
//          │ │ ╰↓x─►│ 2│  │ ├─↑LS─╯ ╰─↓──►│ 5│▒█│ ├↕xH╯ │
//          │ ╰─↓H───┤  │▒█│ ├─────↕LS────►│  │█ │ │◄────╯
//          │        ╰───────╯             ╰───┬───╯
//          ╰────────────────↑xRS──────────────╯
//    input:  ↑ — Top,  ↕ — Both,  ↓ — Bottom
// update_x:  x — true
//   output:  H — Head,     LS — LeftShelf,  IF — InnerFoot,
//            V — Vertical, RS — RightShelf, OF — OuterFoot

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
    (2, true,  FeatureKind::None),
    (3, true,  FeatureKind::LeftShelf),
    (0, false, FeatureKind::Head),
    (5, false, FeatureKind::None),
    (0, false, FeatureKind::RightShelf),
    (3, true,  FeatureKind::Head),
];
