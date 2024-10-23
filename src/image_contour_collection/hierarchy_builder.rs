use std::num::NonZeroUsize;

#[derive(Debug, PartialEq)]
pub struct HierarchyItem {
    pub head_point: usize,
    pub parent: usize,
    pub next_sibling: Option<NonZeroUsize>,
    pub first_child: Option<NonZeroUsize>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Relation {
    Alias(usize),
    Parent(usize),
}

#[derive(Debug, PartialEq)]
struct Head {
    index: usize,
    relation: Relation,
}

pub struct HierarchyBuilder {
    heads: Vec<Head>,
    current_contour: usize,
}

impl HierarchyBuilder {
    pub fn new() -> Self {
        Self { heads: Vec::new(), current_contour: NONE }
    }
    
    pub fn add_contour(&mut self, head_index: usize) -> usize {
        let new_contour = self.heads.len();
        self.heads.push(Head { index: head_index, relation: Relation::Parent(self.current_contour) });
        new_contour
    }
    
    pub fn cross_contour(&mut self, contour: usize) {
        let index = self.unalias(contour);
        if self.current_contour == index {
            if let Relation::Parent(parent) = self.heads[self.current_contour].relation {
                self.current_contour = self.unalias(parent);
            } else {
                panic!();
            }
        } else {
            self.current_contour = index;
        }
    }
    
    pub fn combine_contours(&mut self, from_contour: usize, to_contour: usize) {
        let mut from_index = self.unalias(from_contour);
        let mut to_index = self.unalias(to_contour);
        if from_index != to_index {
            if from_index < to_index {
                (from_index, to_index) = (to_index, from_index);
            }
            if self.current_contour == from_index {
                self.current_contour = to_index;
                if let Relation::Parent(parent) = self.heads[from_index].relation {
                    let parent_index = self.unalias(parent);
                    if parent_index == to_index {
                        if let Relation::Parent(grandparent) = self.heads[parent_index].relation {
                            let grandparent_index = self.unalias(grandparent);
                            self.current_contour = grandparent_index;
                        } else {
                            panic!();
                        }
                    }
                }
            }
            self.heads[from_index].relation = Relation::Alias(to_index);
        }
    }
    
    pub fn into(mut self) -> Vec<HierarchyItem> {
        debug_assert_eq!(self.current_contour, NONE);
        
        let root = HierarchyItem { head_point: NONE, parent: NONE, next_sibling: None, first_child: None };
        let mut hierarchy = vec![root];
        
        for head in 0..self.heads.len() {
            let Head { index: head_point, relation } = self.heads[head];
            if let Relation::Parent(parent_alias) = relation {
                let new_item = hierarchy.len();
                self.heads[head].index = new_item;
                
                let parent = if parent_alias == NONE { 0 } else {
                    let parent_head = self.unalias(parent_alias);
                    self.heads[parent_head].index
                };
                let next_sibling = hierarchy[parent].first_child;
                hierarchy[parent].first_child = NonZeroUsize::new(new_item);
                
                hierarchy.push(HierarchyItem { head_point, parent, next_sibling, first_child: None });
            }
        }
        hierarchy
    }
     
    fn unalias(&self, alias: usize) -> usize {
        if alias == NONE {
            return NONE;
        }
        let mut index = alias;
        while let Relation::Alias(head) = self.heads[index].relation {
            index = head;
        }
        index
    }
}

pub const NONE: usize = usize::MAX;
