/// Coordinates of a contour point
/// and a link to the next point of the contour.
/// 
/// Miyatake’s `rd_code` and `w_link` fields are not stored.
#[derive(Debug, PartialEq, Eq)]
pub struct PointListItem {
    pub x: i32,
    pub y: i32,
    pub next: usize,
}

/// Constructs a contour point list by adding points.
#[derive(Debug)]
pub struct PointListBuilder {
    point_list: Vec<PointListItem>,
}

impl PointListBuilder {
    /// Creates an empty list.
    pub fn new() -> Self {
        Self { point_list: Vec::new() }
    }
    
    /// Adds a point that is not yet connected to anything.
    /// Returns its index.
    pub fn add(&mut self, x: i32, y: i32) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next: NONE });
        index
    }
    
    /// Adds a point connected to an existing next point.
    /// Returns the index of the new point.
    pub fn add_with_next(&mut self, x: i32, y: i32, next: usize) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next });
        index
    }
    
    /// Adds a point connected to an existing previous point.
    /// Returns the index of the new point.
    pub fn add_with_previous(&mut self, x: i32, y: i32, previous: usize) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next: NONE });
        self.point_list[previous].next = index;
        index
    }
    
    /// Adds a point connected to existing next and previous points.
    /// Does not return anything.
    pub fn add_with_next_and_previous(&mut self, x: i32, y: i32, next: usize, previous: usize){
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next });
        self.point_list[previous].next = index;
    }
    
    /// Returns the constructed list.
    pub fn into(self) -> Vec<PointListItem> {
        self.point_list
    }
}

/// A temporary index value for points that do not yet have a `next`.
/// A fully constructed point list should not contain `NONE`s.
const NONE: usize = usize::MAX;
