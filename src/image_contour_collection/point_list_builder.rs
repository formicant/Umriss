#[derive(Debug, PartialEq, Eq)]
pub struct PointListItem {
    pub x: u32,
    pub y: u32,
    pub next: usize,
}

#[derive(Debug)]
pub struct PointListBuilder {
    point_list: Vec<PointListItem>,
}

impl PointListBuilder {
    pub fn new() -> Self {
        Self { point_list: Vec::new() }
    }
    
    pub fn add(&mut self, x: u32, y: u32) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next: NONE });
        index
    }
    
    pub fn add_with_next(&mut self, x: u32, y: u32, next: usize) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next });
        index
    }
    
    pub fn add_with_previous(&mut self, x: u32, y: u32, previous: usize) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next: NONE });
        self.point_list[previous].next = index;
        index
    }
    
    pub fn add_with_next_and_previous(&mut self, x: u32, y: u32, next: usize, previous: usize){
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next });
        self.point_list[previous].next = index;
    }
    
    pub fn into(self) -> Vec<PointListItem> {
        self.point_list
    }
}

const NONE: usize = usize::MAX;
