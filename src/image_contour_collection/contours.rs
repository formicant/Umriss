use super::point_list_builder::{Relation, PointListItem};

pub struct Contour<'a> {
    point_list: &'a[PointListItem],
    is_hole: bool,
    start_index: usize,
}

impl<'a> Contour<'a> {
    pub fn is_hole(&self) -> bool {
        self.is_hole
    }
    
    pub fn even_points(&self) -> EvenPointIter<'a> {
        EvenPointIter { point_list: self.point_list, start_index: self.start_index, current_index: Some(self.start_index) }
    }
}

pub struct EvenPointIter<'a> {
    point_list: &'a[PointListItem],
    start_index: usize,
    current_index: Option<usize>,
}

impl<'a> Iterator for EvenPointIter<'a> {
    type Item = (u32, u32);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|index| {
            let point = &self.point_list[index];
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
    point_list: &'a[PointListItem],
    is_hole: bool,
    current_index: Option<usize>,
}

impl<'a> ChildContourIter<'a> {
    pub fn new(point_list: &'a[PointListItem], parent_index: usize, is_parent_hole: bool) -> Self {
        let is_hole = !is_parent_hole;
        match point_list[parent_index].relation {
            Relation::Child(child) => Self { point_list, is_hole, current_index: Some(child) },
            _ => Self { point_list, is_hole, current_index: None }
        }
    }
}

impl<'a> Iterator for ChildContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|start_index| {
            let head = &self.point_list[start_index];
            let next = &self.point_list[head.next];
            self.current_index = match next.relation {
                Relation::Sibling(sibling) => Some(sibling),
                _ => None,
            };
            Contour { point_list: self.point_list, is_hole: self.is_hole, start_index }
        })
    }
}

pub struct DescendantContourIter<'a> {
    point_list: &'a[PointListItem],
    is_root_hole: bool,
    stack: Vec<usize>,
}

impl<'a> DescendantContourIter<'a> {
    pub fn new(point_list: &'a[PointListItem], root_index: usize, is_root_hole: bool) -> Self {
        match point_list[root_index].relation {
            Relation::Child(child) => Self { point_list, is_root_hole, stack: vec![child] },
            _ => Self { point_list, is_root_hole, stack: Vec::new() }
        }
    }
}

impl<'a> Iterator for DescendantContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|start_index| {
            let is_hole = self.is_root_hole ^ (self.stack.len() % 2 == 0);
            let head = &self.point_list[start_index];
            let next = &self.point_list[head.next];
            if let Relation::Sibling(sibling) = next.relation {
                self.stack.push(sibling);
            }
            if let Relation::Child(child) = head.relation {
                self.stack.push(child);
            }
            Contour { point_list: self.point_list, is_hole, start_index }
        })
    }
}
