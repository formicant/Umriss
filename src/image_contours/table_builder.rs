#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Relation {
    None,
    Alias(usize),
    Parent(usize),
    Child(usize),
    Sibling(usize),
}

pub struct TableItem {
    pub x: u32,
    pub y: u32,
    pub next: usize,
    pub relation: Relation,
}

impl std::fmt::Debug for TableItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:4}, {:4}) {:5} {:?}", self.x, self.y, self.next, self.relation)
    }
}

pub struct TableBuilder {
    table: Vec<TableItem>,
    current_contour: usize,
}

impl TableBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        let root = TableItem { x: width, y: height, next: 0, relation: Relation::None };
        Self { table: vec![root], current_contour: 0 }
    }
    
    pub fn add_with_contour(&mut self, x: u32, y: u32) -> usize {
        let index = self.table.len();
        self.table.push(TableItem { x, y, next: 0, relation: Relation::Parent(self.current_contour) });
        index
    }
    
    pub fn add_with_next(&mut self, x: u32, y: u32, next: usize) -> usize {
        let index = self.table.len();
        self.table.push(TableItem { x, y, next, relation: Relation::None });
        index
    }
    
    pub fn add_with_previous(&mut self, x: u32, y: u32, previous: usize) -> usize {
        let index = self.table.len();
        self.table.push(TableItem { x, y, next: 0, relation: Relation::None });
        self.table[previous].next = index;
        index
    }
    
    pub fn add_with_next_and_previous(&mut self, x: u32, y: u32, next: usize, previous: usize){
        let index = self.table.len();
        self.table.push(TableItem { x, y, next, relation: Relation::None });
        self.table[previous].next = index;
    }
    
    pub fn cross_contour(&mut self, head: usize) {
        let index = self.unalias(head);
        if self.current_contour == index {
            if let Relation::Parent(parent) = self.table[self.current_contour].relation {
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
            self.table[from_index].relation = Relation::Alias(to_index);
            if self.current_contour == from_index {
                self.current_contour = to_index;
            }
        }
    }
    
    pub fn into(mut self) -> Vec<TableItem> {
        debug_assert_eq!(self.current_contour, 0);
        self.parents_to_children();
        self.table
    }
    
    fn unalias(&self, alias: usize) -> usize {
        let mut index = alias;
        while let Relation::Alias(head) = self.table[index].relation {
            index = head;
        }
        index
    }
    
    fn parents_to_children(&mut self) {
        for index in 1..self.table.len() {
            if let Relation::Parent(p) = self.table[index].relation {
                let parent = self.unalias(p);
                if let Relation::Child(child) = self.table[parent].relation {
                    let next = self.table[index].next;
                    self.table[next].relation = Relation::Sibling(child);
                }
                self.table[parent].relation = Relation::Child(index);
            }
        }
    }
}