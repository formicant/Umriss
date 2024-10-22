use super::row_changes::END;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RunChangeKind { Top, Both, Bottom }

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RunChange {
    pub kind: RunChangeKind,
    pub x: u32,
}

pub struct RunChangeIter<'a> {
    run_top: &'a[u32],
    run_bottom: &'a[u32],
    top_index: usize,
    bottom_index: usize,
}

impl<'a> RunChangeIter<'a> {
    pub fn new(run_top: &'a[u32], run_bottom: &'a[u32]) -> Self {
        Self { run_top, run_bottom, top_index: 0, bottom_index: 0 }
    }
}

impl<'a> Iterator for RunChangeIter<'a> {
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


// ---------

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use super::*;

    #[test_case(&[END], &[END], vec![])]
    #[test_case(&[0, END], &[END], vec![RunChange{ kind: RunChangeKind::Top, x: 0 }])]
    #[test_case(&[END], &[0, END], vec![RunChange{ kind: RunChangeKind::Bottom, x: 0 }])]
    #[test_case(&[0, END], &[0, END], vec![RunChange{ kind: RunChangeKind::Both, x: 0 }])]
    #[test_case(
        &[1, 38, 39, 41, END],
        &[1, 2, 39, 42, END],
        vec![
            RunChange{ kind: RunChangeKind::Both, x: 1 },
            RunChange{ kind: RunChangeKind::Bottom, x: 2 },
            RunChange{ kind: RunChangeKind::Top, x: 38 },
            RunChange{ kind: RunChangeKind::Both, x: 39 },
            RunChange{ kind: RunChangeKind::Top, x: 41 },
            RunChange{ kind: RunChangeKind::Bottom, x: 42 },
        ]
    )]
    fn pixel_row(run_top: &[u32], run_bottom: &[u32], expected: Vec<RunChange>) {
        let actual = RunChangeIter::new(run_top, run_bottom);
        assert!(actual.eq(expected));
    }
}
