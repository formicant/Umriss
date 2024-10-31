use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct HierarchyItem {
    pub head_point_index: usize,
    pub parent: usize,
    pub next_sibling: Option<NonZeroUsize>,
    pub first_child: Option<NonZeroUsize>,
}

#[derive(Debug, PartialEq, Eq, Default)]
struct Head {
    point_index: usize,
    is_alias: bool,
    parent: usize,
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
        self.heads.push(Head { point_index, parent: parent_contour, ..Default::default() });
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
            alias.parent = to;
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
                let parent = self.heads[self.unalias(head.parent)].hierarchy_index;
                
                hierarchy[current].parent = parent;
                hierarchy[current].next_sibling = hierarchy[parent].first_child;
                hierarchy[parent].first_child = NonZeroUsize::new(current);
            }
        }
        hierarchy
    }
    
    /// Returns the index of the main head of a contour by an aliased head index.
    /// TODO: Consider using disjoint-set forest balancing.
    fn unalias(&self, alias: usize) -> usize {
        let mut index = alias;
        while let Head { point_index: _, is_alias: true, parent, hierarchy_index: _ } = self.heads[index] {
            index = parent;
        }
        index
    }
}


// ---------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let builder = HierarchyBuilder::new();
        
        let expected_heads = vec![Default::default()];
        assert_eq!(builder.heads, expected_heads);
        
        let expected_hierarchy = vec![root(None)];
        assert_eq!(builder.into(), expected_hierarchy);
    }
    
    /// ```
    /// 0 ┌+1─┐0  ┌+2─────────┐0  ┌+3─────────┐0
    ///   │   │   │           │   │           │
    /// 0 │1  └2→1┘1  ┌+4─┐1  │0  │3  ┌+5─┐1  │0
    ///   │           │   │   │   │   │   │   │
    /// 0 │1  ┌+6─┐1  │4  │1  │0  │3  └───┘3  │0
    ///   │   │   │   │   │   │   │           │
    /// 0 │1  │6  └6→4┘4  │1  └3→1┘1      ┌───┘0
    ///   │   │           │               │
    /// 0 │1  └───────────┘1  ┌+7─────┐1  │0
    ///   │                   │       │   │
    /// 0 │1      ┌+8─────┐1  └───────┘1  │0
    ///   │       │       │               │
    /// 0 └──8→0──┘0      └───────────────┘0
    /// ```
    #[test]
    fn contour_depicted_above() {
        let expected_heads = vec![
            Default::default(),
            head(0,  false, 0),
            head(1,  true,  1),
            head(2,  true,  1),
            head(4,  false, 1),
            head(5,  false, 3),
            head(6,  true,  4),
            head(12, false, 1),
            head(13, true,  0),
        ];
        let expected_hierarchy = vec![
            root(NonZeroUsize::new(1)),
            hier(0,  0, None, NonZeroUsize::new(2)),
            hier(4,  1, NonZeroUsize::new(3), None),
            hier(5,  1, NonZeroUsize::new(4), None),
            hier(12, 1, None, None),
        ];
        
        let mut builder = HierarchyBuilder::new();
        
        assert_eq!(builder.add_contour(0), (1, 0));
        assert_eq!(builder.add_contour(1), (2, 0));
        assert_eq!(builder.add_contour(2), (3, 0));
        
        builder.cross_contour(1);
        builder.merge_contours(2);
        assert_eq!(builder.add_contour(4), (4, 1));
        builder.cross_contour(0);
        builder.cross_contour(3);
        assert_eq!(builder.add_contour(5), (5, 3));
        builder.cross_contour(0);
        
        builder.cross_contour(1);
        assert_eq!(builder.add_contour(6), (6, 1));
        builder.cross_contour(4);
        builder.cross_contour(1);
        builder.cross_contour(0);
        builder.cross_contour(3);
        builder.merge_contours(3);
        builder.cross_contour(0);
        
        builder.cross_contour(1);
        builder.cross_contour(6);
        builder.merge_contours(4);
        builder.cross_contour(1);
        builder.merge_contours(3);
        builder.cross_contour(0);
        
        builder.cross_contour(1);
        builder.merge_contours(1);
        assert_eq!(builder.add_contour(12), (7, 1));
        builder.cross_contour(0);
        
        builder.cross_contour(1);
        assert_eq!(builder.add_contour(13), (8, 1));
        builder.merge_contours(1);
        builder.cross_contour(0);
        
        builder.merge_contours(8);
        builder.merge_contours(0);
        
        assert_eq!(builder.heads, expected_heads);
        assert_eq!(builder.into(), expected_hierarchy);
    }
    
    const fn head(point_index: usize, is_alias: bool, parent: usize) -> Head {
        Head { point_index, is_alias, parent, hierarchy_index: 0 }
    }
    
    const fn root(first_child: Option<NonZeroUsize>) -> HierarchyItem {
        HierarchyItem { head_point_index: 0, parent: 0, next_sibling: None, first_child }
    }

    const fn hier(head_point: usize, parent: usize, next_sibling: Option<NonZeroUsize>, first_child: Option<NonZeroUsize>) -> HierarchyItem {
        HierarchyItem { head_point_index: head_point, parent, next_sibling, first_child }
    }
}
