#[derive(Debug)]
pub enum Relation {
    None,
    Alias(usize),
    Parent(usize),
    Child(usize),
    Sibling(usize),
}

pub struct ContourPoint {
    pub x: u32,
    pub y: u32,
    pub next: usize,
    pub relation: Relation,
}

impl ContourPoint {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y, next: 0, relation: Relation::None }
    }
    
    pub fn with_next(x: u32, y: u32, next: usize) -> Self {
        Self { x, y, next, relation: Relation::None }
    }
}

impl std::fmt::Debug for ContourPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:4}, {:4}) {:5} {:?}", self.x, self.y, self.next, self.relation)
    }
}

#[derive(Copy, Clone)]
pub enum Feature {
    None,
    Head,
    Vertical,
    LeftShelf,
    RightShelf,
    InnerFoot,
    OuterFoot,
}
