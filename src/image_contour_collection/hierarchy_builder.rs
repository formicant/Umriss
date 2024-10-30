use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct HierarchyItem {
    pub head_point_index: usize,
    pub parent: usize,
    pub next_sibling: Option<NonZeroUsize>,
    pub first_child: Option<NonZeroUsize>,
}

#[derive(Debug, Default)]
struct Head {
    point_index: usize,
    is_alias: bool,
    parent_contour: usize,
    hierarchy_index: usize,
}

#[derive(Debug)]
pub struct HierarchyBuilder {
    heads: Vec<Head>,
    contour_to_the_left: usize,
}

impl HierarchyBuilder {
    pub fn new() -> Self {
        let root = Default::default();
        Self { heads: vec![root], contour_to_the_left: 0 }
    }
    
    pub fn add_contour(&mut self, point_index: usize) -> (usize, usize) {
        let parent_contour = self.contour_to_the_left;
        let new_contour = self.heads.len();
        self.heads.push(Head { point_index, parent_contour, ..Default::default() });
        (new_contour, parent_contour)
    }
    
    pub fn cross_contour(&mut self, contour_to_the_right: usize) {
        self.contour_to_the_left = contour_to_the_right;
    }
    
    pub fn merge_contours(&mut self, contour_to_the_right: usize) {
        let left = self.unalias(self.contour_to_the_left);
        let right = self.unalias(contour_to_the_right);
        
        if left != right {
            let (from, to) = if left < right { (right, left) } else { (left, right) };
            let alias = &mut self.heads[from];
            alias.is_alias = true;
            alias.parent_contour = to;
        }
    }
    
    pub fn into(mut self) -> Vec<HierarchyItem> {
        assert_eq!(self.unalias(self.contour_to_the_left), 0, "Some contours left unclosed");
        
        let mut hierarchy = Vec::new();
        for head in self.heads.iter_mut().filter(|h| !h.is_alias) {
            head.hierarchy_index = hierarchy.len();
            hierarchy.push(HierarchyItem { head_point_index: head.point_index, ..Default::default() });
        }
        
        for index in (1..self.heads.len()).rev() {
            let head = &self.heads[index];
            if !head.is_alias {
                let current = head.hierarchy_index;
                let parent = self.heads[self.unalias(head.parent_contour)].hierarchy_index;
                
                hierarchy[current].parent = parent;
                hierarchy[current].next_sibling = hierarchy[parent].first_child;
                hierarchy[parent].first_child = NonZeroUsize::new(current);
            }
        }
        hierarchy
    }
     
    fn unalias(&self, alias: usize) -> usize {
        let mut index = alias;
        while let Head { point_index: _, is_alias: true, parent_contour, hierarchy_index: _ } = self.heads[index] {
            index = parent_contour;
        }
        index
    }
}
