use std::num::NonZeroUsize;
use super::hierarchy_builder::HierarchyItem;
use super::point_list_builder::PointListItem;

/// Represents a contour in a contour collection.
/// 
/// The lifetime is bounded by the lifetime of the collection.
/// 
/// Provides methods for iterating over the points
/// and navigating through the contour hierarchy.
#[derive(Debug, Clone, Copy)]
pub struct Contour<'a> {
    hierarchy: &'a[HierarchyItem],
    point_list: &'a[PointListItem],
    is_outer: bool,
    index: NonZeroUsize,
}

impl<'a> Contour<'a> {
    /// Returns `true` for outer contours
    /// and `false` for inner contours (holes).
    pub fn is_outer(&self) -> bool {
        self.is_outer
    }
    
    /// Iterates only even points of the contour.
    /// 
    /// Even points are sufficient to represent a contour.
    /// Odd points can be derived from them unambiguously:
    /// an odd point inherits its y coordinate from the previous
    /// even point and x coordinate from the next even point.
    /// 
    /// Points are iterated from the top of the contour,
    /// clockwise for outer contours and anti-clockwise for inner ones.
    pub fn even_points(&self) -> EvenPointIter<'a> {
        let start_index = self.hierarchy[self.index.get()].head_point;
        EvenPointIter { point_list: self.point_list, start_index, current_index: Some(start_index) }
    }
    
    /// Iterates the contour’s child contours.
    /// 
    /// All children of an outer contour are inner contours and vice versa.
    pub fn children(&self) -> ChildContourIter<'a> {
        ChildContourIter::new(self.hierarchy, self.point_list, self.index.get(), self.is_outer)
    }
    
    /// Iterates all the contour’s descendant contours
    /// i.e. children, grandchildren, etc, depth-first.
    pub fn all_descendants(&self) -> DescendantContourIter<'a> {
        DescendantContourIter::new(self.hierarchy, self.point_list, self.index.get(), self.is_outer)
    }
    
    /// Returns the contour’s parent contour
    /// or `None` for outermost contours.
    pub fn parent(&self) -> Option<Self> {
        let parent_index = self.hierarchy[self.index.get()].parent;
        NonZeroUsize::new(parent_index).map(|index| {
            let is_outer = !self.is_outer;
            Self { hierarchy: self.hierarchy, point_list: self.point_list, is_outer: is_outer, index }
        })
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
    is_outer: bool,
    current_index: Option<NonZeroUsize>,
}

impl<'a> ChildContourIter<'a> {
    pub fn new(hierarchy: &'a[HierarchyItem], point_list: &'a[PointListItem], parent_index: usize, is_parent_outer: bool) -> Self {
        let is_outer = !is_parent_outer;
        let current_index = hierarchy[parent_index].first_child;
        Self { hierarchy, point_list, is_outer, current_index }
    }
}

impl<'a> Iterator for ChildContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|index| {
            let current = &self.hierarchy[index.get()];
            self.current_index = current.next_sibling;
            Contour { hierarchy: self.hierarchy, point_list: self.point_list, is_outer: self.is_outer, index }
        })
    }
}

pub struct DescendantContourIter<'a> {
    hierarchy: &'a[HierarchyItem],
    point_list: &'a[PointListItem],
    root_index: usize,
    is_outer: bool,
    current_index: Option<NonZeroUsize>,
}

impl<'a> DescendantContourIter<'a> {
    pub fn new(hierarchy: &'a[HierarchyItem], point_list: &'a[PointListItem], root_index: usize, is_root_outer: bool) -> Self {
        let is_outer = !is_root_outer;
        let current_index = hierarchy[root_index].first_child;
        Self { hierarchy, point_list, root_index, is_outer: is_outer, current_index }
    }
}

impl<'a> Iterator for DescendantContourIter<'a> {
    type Item = Contour<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index.map(|index| {
            let is_outer = self.is_outer;
            let mut current = &self.hierarchy[index.get()];
            if let Some(child) = current.first_child {
                self.current_index = Some(child);
                self.is_outer = !self.is_outer;
            } else {
                loop {
                    if let Some(sibling) = current.next_sibling {
                        self.current_index = Some(sibling);
                        break;
                    } else if current.parent != self.root_index {
                        current = &self.hierarchy[current.parent];
                        self.is_outer = !self.is_outer;
                    } else {
                        self.current_index = None;
                        break;
                    }
                }
            }
            Contour { hierarchy: self.hierarchy, point_list: self.point_list, is_outer: is_outer, index }
        })
    }
}
