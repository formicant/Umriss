use super::table_builder::{Relation, TableItem};

pub struct Contour<'a> {
    table: &'a[TableItem],
    is_hole: bool,
    start_index: usize,
}

impl<'a> Contour<'a> {
    pub fn is_hole(&self) -> bool {
        self.is_hole
    }
    
    pub fn even_points(&self) -> EvenPointIter<'a> {
        EvenPointIter { table: self.table, start_index: self.start_index, current_index: Some(self.start_index) }
    }
}

pub struct EvenPointIter<'a> {
    table: &'a[TableItem],
    start_index: usize,
    current_index: Option<usize>,
}

impl<'a> Iterator for EvenPointIter<'a> {
    type Item = (u32, u32);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|index| {
            let point = &self.table[index];
            self.current_index = if point.next != self.start_index {
                Some(point.next)
            } else {
                None
            };
            (point.x, point.y)
        })
    }
}

pub struct ChildContourIter<'a> {
    table: &'a[TableItem],
    is_hole: bool,
    current_index: Option<usize>,
}

impl<'a> ChildContourIter<'a> {
    pub fn new(table: &'a[TableItem], parent_index: usize, is_parent_hole: bool) -> Self {
        let is_hole = !is_parent_hole;
        match table[parent_index].relation {
            Relation::Child(child) => Self { table, is_hole, current_index: Some(child) },
            _ => Self { table, is_hole, current_index: None }
        }
    }
}

impl<'a> Iterator for ChildContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|start_index| {
            let head = &self.table[start_index];
            let next = &self.table[head.next];
            self.current_index = match next.relation {
                Relation::Sibling(sibling) => Some(sibling),
                _ => None,
            };
            Contour { table: self.table, is_hole: self.is_hole, start_index }
        })
    }
}

pub struct DescendantContourIter<'a> {
    table: &'a[TableItem],
    is_root_hole: bool,
    hole_filter: Option<bool>,
    stack: Vec<usize>,
}

impl<'a> DescendantContourIter<'a> {
    pub fn new(table: &'a[TableItem], root_index: usize, is_root_hole: bool, hole_filter: Option<bool>) -> Self {
        match table[root_index].relation {
            Relation::Child(child) => Self { table, is_root_hole, hole_filter, stack: vec![child] },
            _ => Self { table, is_root_hole, hole_filter, stack: Vec::new() }
        }
    }
}

impl<'a> Iterator for DescendantContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|start_index| {
            let is_hole = self.is_root_hole ^ (self.stack.len() % 2 == 0);
            let head = &self.table[start_index];
            let next = &self.table[head.next];
            if let Relation::Sibling(sibling) = next.relation {
                self.stack.push(sibling);
            }
            if let Relation::Child(child) = head.relation {
                self.stack.push(child);
            }
            Contour { table: self.table, is_hole, start_index }
        })
    }
}
