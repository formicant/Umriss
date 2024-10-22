use super::table_builder::{Relation, TableItem};

pub struct Contour<'a> {
    table: &'a[TableItem],
    is_hole: bool,
    start_index: usize,
}

impl<'a> Contour<'a> {
    pub fn new(table: &'a[TableItem], is_hole: bool, start_index: usize) -> Self {
        Self { table, is_hole, start_index }
    }
    
    pub fn is_hole(&self) -> bool {
        self.is_hole
    }
    
    pub fn control_points(&self) -> ControlPointIter<'a> {
        ControlPointIter { table: self.table, start_index: self.start_index, current_index: Some(self.start_index) }
    }
}

pub struct ControlPointIter<'a> {
    table: &'a[TableItem],
    start_index: usize,
    current_index: Option<usize>,
}

impl<'a> Iterator for ControlPointIter<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_index {
            Some(index) => {
                let point = &self.table[index];
                self.current_index = if point.next != self.start_index {
                    Some(point.next)
                } else {
                    None
                };
                Some((point.x, point.y))
            },
            None => None,
        }
    }
}

pub struct SiblingContourIter<'a> {
    table: &'a[TableItem],
    is_hole: bool,
    current_index: Option<usize>,
}

impl<'a> SiblingContourIter<'a> {
    pub fn empty(table: &'a[TableItem]) -> Self {
        Self { table, is_hole: false, current_index: None }
    }
    
    pub fn new(table: &'a[TableItem], is_hole: bool, start_index: usize) -> Self {
        Self { table, is_hole, current_index: Some(start_index) }
    }
}

impl<'a> Iterator for SiblingContourIter<'a> {
    type Item = Contour<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_index {
            Some(index) => {
                let point = &self.table[index];
                let next_point = &self.table[point.next];
                self.current_index = if let Relation::Sibling(sibling) = next_point.relation {
                    Some(sibling)
                } else {
                    None
                };
                Some(Contour::new(self.table, self.is_hole, index))
            },
            None => None,
        }
    }
}
