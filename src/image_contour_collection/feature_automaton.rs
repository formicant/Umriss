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

/// Contour feature.
/// 
/// `kind` — see `FeatureKind` (the picture above).
/// 
/// `x` is the x coordinate of the _representative point_ of the feature
/// (marked with black circles in the picture above).
/// 
/// When the features are combined in the correct order,
/// their representative points become even points of the contour.
/// Odd points can be derived from them unambiguously.
/// `Vertical` features, however, should be ignored during this process
/// because they have no representative points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Feature {
    pub kind: FeatureKind,
    pub x: i32,
}

/// A final-state automaton that takes row pair changes as input
/// and returns contour features as output.
#[derive(Debug)]
pub struct FeatureAutomaton {
    /// The states are the same as in Miyatake’s article,
    /// but 0-based numeration is used instead of 1-based.
    state: usize,
    
    /// Stores x coordinate of the representative point of the feature.
    feature_x: i32,
}

impl FeatureAutomaton {
    pub fn new() -> Self {
        Self { state: 0, feature_x: 0 }
    }
    
    pub fn step(&mut self, change: RowPairChange) -> Feature {
        let (new_state, update_x, feature_kind) = match change.kind {
            RowPairChangeKind::Top => STEP_IF_TOP[self.state],
            RowPairChangeKind::Bottom => STEP_IF_BOTTOM[self.state],
            RowPairChangeKind::Both => STEP_IF_BOTH[self.state],
        };
        self.state = new_state;
        if update_x {
            self.feature_x = change.x;
        }
        Feature { kind: feature_kind, x: self.feature_x }
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
//   pixels:  │ │ — background, │█│ — foreground, │▒│ — any
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
const STEP_IF_BOTTOM: [(usize, bool, FeatureKind); 6] = [
    (2, true,  FeatureKind::None),
    (3, true,  FeatureKind::LeftShelf),
    (0, false, FeatureKind::Head),
    (5, false, FeatureKind::None),
    (0, false, FeatureKind::RightShelf),
    (3, true,  FeatureKind::Head),
];
const STEP_IF_BOTH: [(usize, bool, FeatureKind); 6] = [
    (3, false, FeatureKind::Vertical),
    (4, true,  FeatureKind::LeftShelf),
    (5, false, FeatureKind::LeftShelf),
    (0, false, FeatureKind::Vertical),
    (5, false, FeatureKind::InnerFoot),
    (4, true,  FeatureKind::Head),
];
