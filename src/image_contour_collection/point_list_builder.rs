#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Relation {
    None,
    Alias(usize),
    Parent(usize),
    Child(usize),
    Sibling(usize),
}

pub struct PointListItem {
    pub x: u32,
    pub y: u32,
    pub next: usize,
    pub relation: Relation,
}

impl std::fmt::Debug for PointListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:4}, {:4}) {:5} {:?}", self.x, self.y, self.next, self.relation)
    }
}

pub struct PointListBuilder {
    point_list: Vec<PointListItem>,
    current_contour: usize,
}

impl PointListBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        let root = PointListItem { x: width, y: height, next: 0, relation: Relation::None };
        Self { point_list: vec![root], current_contour: 0 }
    }
    
    pub fn add_with_new_contour(&mut self, x: u32, y: u32) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next: 0, relation: Relation::Parent(self.current_contour) });
        index
    }
    
    pub fn add_with_next(&mut self, x: u32, y: u32, next: usize) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next, relation: Relation::None });
        index
    }
    
    pub fn add_with_previous(&mut self, x: u32, y: u32, previous: usize) -> usize {
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next: 0, relation: Relation::None });
        self.point_list[previous].next = index;
        index
    }
    
    pub fn add_with_next_and_previous(&mut self, x: u32, y: u32, next: usize, previous: usize){
        let index = self.point_list.len();
        self.point_list.push(PointListItem { x, y, next, relation: Relation::None });
        self.point_list[previous].next = index;
    }
    
    pub fn cross_contour(&mut self, head: usize) {
        let index = self.unalias(head);
        if self.current_contour == index {
            if let Relation::Parent(parent) = self.point_list[self.current_contour].relation {
                self.current_contour = self.unalias(parent);
            } else {
                panic!();
            }
        } else {
            self.current_contour = index;
        }
    }
    
    pub fn combine_contours(&mut self, from_head: usize, to_head: usize) {
        let mut from_index = self.unalias(from_head);
        let mut to_index = self.unalias(to_head);
        if from_index != to_index {
            if from_index < to_index {
                (from_index, to_index) = (to_index, from_index);
            }
            self.point_list[from_index].relation = Relation::Alias(to_index);
            if self.current_contour == from_index {
                self.current_contour = to_index;
            }
        }
    }
    
    pub fn into(mut self) -> Vec<PointListItem> {
        debug_assert_eq!(self.current_contour, 0);
        self.parents_to_children();
        self.point_list
    }
    
    fn unalias(&self, alias: usize) -> usize {
        let mut index = alias;
        while let Relation::Alias(head) = self.point_list[index].relation {
            index = head;
        }
        index
    }
    
    fn parents_to_children(&mut self) {
        for index in 1..self.point_list.len() {
            if let Relation::Parent(p) = self.point_list[index].relation {
                let parent = self.unalias(p);
                if let Relation::Child(child) = self.point_list[parent].relation {
                    let next = self.point_list[index].next;
                    self.point_list[next].relation = Relation::Sibling(child);
                }
                self.point_list[parent].relation = Relation::Child(index);
            }
        }
    }
}