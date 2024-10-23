use std::num::NonZeroUsize;
use super::hierarchy_builder::HierarchyItem;
use super::point_list_builder::PointListItem;

pub struct Contour<'a> {
    hierarchy: &'a[HierarchyItem],
    point_list: &'a[PointListItem],
    is_hole: bool,
    index: NonZeroUsize,
}

impl<'a> Contour<'a> {
    pub fn is_hole(&self) -> bool {
        self.is_hole
    }
    
    pub fn even_points(&self) -> EvenPointIter<'a> {
        let start_index = self.hierarchy[self.index.get()].head_point;
        EvenPointIter { point_list: self.point_list, start_index, current_index: Some(start_index) }
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
    hierarchy: &'a[HierarchyItem],
    point_list: &'a[PointListItem],
    is_hole: bool,
    current_index: Option<NonZeroUsize>,
}

impl<'a> ChildContourIter<'a> {
    pub fn new(hierarchy: &'a[HierarchyItem], point_list: &'a[PointListItem], parent_index: usize, is_parent_hole: bool) -> Self {
        let is_hole = !is_parent_hole;
        let current_index = hierarchy[parent_index].first_child;
        Self { hierarchy, point_list, is_hole, current_index }
    }
}

impl<'a> Iterator for ChildContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|index| {
            let current = &self.hierarchy[index.get()];
            self.current_index = current.next_sibling;
            Contour { hierarchy: self.hierarchy, point_list: self.point_list, is_hole: self.is_hole, index }
        })
    }
}

pub struct DescendantContourIter<'a> {
    hierarchy: &'a[HierarchyItem],
    point_list: &'a[PointListItem],
    root_index: usize,
    is_hole: bool,
    current_index: Option<NonZeroUsize>,
}

impl<'a> DescendantContourIter<'a> {
    pub fn new(hierarchy: &'a[HierarchyItem], point_list: &'a[PointListItem], root_index: usize, is_root_hole: bool) -> Self {
        let is_hole = !is_root_hole;
        let current_index = hierarchy[root_index].first_child;
        Self { hierarchy, point_list, root_index, is_hole, current_index }
    }
}

impl<'a> Iterator for DescendantContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|index| {
            let is_hole = self.is_hole;
            let mut current = &self.hierarchy[index.get()];
            if let Some(child) = current.first_child {
                self.current_index = Some(child);
                self.is_hole = !self.is_hole;
            } else {
                loop {
                    if let Some(sibling) = current.next_sibling {
                        self.current_index = Some(sibling);
                        break;
                    } else if current.parent != self.root_index {
                        current = &self.hierarchy[current.parent];
                        self.is_hole = !self.is_hole;
                    } else {
                        self.current_index = None;
                        break;
                    }
                }
            }
            Contour { hierarchy: self.hierarchy, point_list: self.point_list, is_hole, index }
        })
    }
}
