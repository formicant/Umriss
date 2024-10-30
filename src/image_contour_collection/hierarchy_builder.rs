use std::num::NonZeroUsize;
use std::cmp::{min, max};

#[derive(Debug, PartialEq, Eq)]
pub struct HierarchyItem {
    pub head_point: usize,
    pub parent: usize,
    pub next_sibling: Option<NonZeroUsize>,
    pub first_child: Option<NonZeroUsize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    None,
    Alias(usize),
    Parent(usize),
}

#[derive(Debug)]
struct Head {
    index: usize,
    relation: Relation,
}

#[derive(Debug)]
pub struct HierarchyBuilder {
    heads: Vec<Head>,
    contour_to_the_left: usize,
}

impl HierarchyBuilder {
    pub fn new() -> Self {
        let root = Head { index: 0, relation: Relation::None };
        Self { heads: vec![root], contour_to_the_left: 0 }
    }
    
    pub fn add_contour(&mut self, head_index: usize) -> (usize, usize) {
        let new_contour = self.heads.len();
        self.heads.push(Head { index: head_index, relation: Relation::Parent(self.contour_to_the_left) });
        let contour_to_the_right = self.contour_to_the_left;
        (new_contour, contour_to_the_right)
    }
    
    pub fn cross_contour(&mut self, contour_to_the_right: usize) {
        self.contour_to_the_left = contour_to_the_right;
    }
    
    pub fn merge_contours(&mut self, contour_to_the_right: usize) {
        let to_the_left = self.unalias(self.contour_to_the_left);
        let to_the_right = self.unalias(contour_to_the_right);
        
        if to_the_left != to_the_right {
            let from = max(to_the_left, to_the_right);
            let to = min(to_the_left, to_the_right);
            self.heads[from].relation = Relation::Alias(to);
        }
    }
    
    pub fn into(mut self) -> Vec<HierarchyItem> {
        assert_eq!(self.unalias(self.contour_to_the_left), 0, "Some contours left unclosed");
        
        let root = HierarchyItem { head_point: NONE, parent: NONE, next_sibling: None, first_child: None };
        let mut hierarchy = vec![root];
        
        for head in 1..self.heads.len() {
            let Head { index: head_point, relation } = self.heads[head];
            if let Relation::Parent(parent_alias) = relation {
                let new_item = hierarchy.len();
                self.heads[head].index = new_item;
                
                let parent = self.heads[self.unalias(parent_alias)].index;
                let next_sibling = hierarchy[parent].first_child;
                hierarchy[parent].first_child = NonZeroUsize::new(new_item);
                
                hierarchy.push(HierarchyItem { head_point, parent, next_sibling, first_child: None });
            }
        }
        hierarchy
    }
     
    fn unalias(&self, alias: usize) -> usize {
        let mut index = alias;
        while let Relation::Alias(head) = self.heads[index].relation {
            index = head;
        }
        index
    }
}

pub const NONE: usize = usize::MAX;
