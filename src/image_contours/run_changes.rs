use std::iter;

use super::row_changes::END;

#[derive(Copy, Clone)]
pub enum RunChangeKind { Top, Both, Bottom }

#[derive(Copy, Clone)]
pub struct RunChange {
    pub kind: RunChangeKind,
    pub x: u32,
}

pub struct RunChanges<'a> {
    run_top: &'a[u32],
    run_bottom: &'a[u32],
    top_index: usize,
    bottom_index: usize,
}

impl<'a> RunChanges<'a> {
    pub fn new(run_top: &'a[u32], run_bottom: &'a[u32]) -> Self {
        Self { run_top, run_bottom, top_index: 0, bottom_index: 0 }
    }
    
    pub fn empty() -> iter::Once<u32> {
        iter::once(END)
    }
}

impl<'a> Iterator for RunChanges<'a> {
    type Item = RunChange;

    fn next(&mut self) -> Option<Self::Item> {
        let top_x = self.run_top[self.top_index];
        let bottom_x = self.run_bottom[self.bottom_index];
        
        if top_x < bottom_x  {
            self.top_index += 1;
            Some(RunChange { kind: RunChangeKind::Top, x: top_x })
        } else if top_x > bottom_x {
            self.bottom_index += 1;
            Some(RunChange{ kind: RunChangeKind::Bottom, x: bottom_x })
        } else if top_x != END {
            self.top_index += 1;
            self.bottom_index += 1;
            Some(RunChange{ kind: RunChangeKind::Both, x: top_x })
        } else {
            None
        }
    }
}
