use super::row_changes::END;

/// In which row of the pair a change has occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowPairChangeKind { Top, Bottom, Both }

/// Kind and x coordinate of a row pair change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowPairChange {
    pub kind: RowPairChangeKind,
    pub x: i32,
}

/// Iterates changes in either of the two adjacent image rows,
/// from left to right.
pub struct RowPairChangeIter<'a> {
    top_changes: &'a[i32],
    bottom_changes: &'a[i32],
    top_index: usize,
    bottom_index: usize,
}

impl<'a> RowPairChangeIter<'a> {
    /// Looks at the changes of two rows of the image
    /// and returns a row pair change iterator.
    pub fn new(top_changes: &'a[i32], bottom_changes: &'a[i32]) -> Self {
        Self { top_changes, bottom_changes, top_index: 0, bottom_index: 0 }
    }
}

impl<'a> Iterator for RowPairChangeIter<'a> {
    type Item = RowPairChange;

    fn next(&mut self) -> Option<Self::Item> {
        let top_x = self.top_changes[self.top_index];
        let bottom_x = self.bottom_changes[self.bottom_index];
        
        if top_x < bottom_x  {
            self.top_index += 1;
            Some(RowPairChange { kind: RowPairChangeKind::Top, x: top_x })
        } else if top_x > bottom_x {
            self.bottom_index += 1;
            Some(RowPairChange{ kind: RowPairChangeKind::Bottom, x: bottom_x })
        } else if top_x != END {
            self.top_index += 1;
            self.bottom_index += 1;
            Some(RowPairChange{ kind: RowPairChangeKind::Both, x: top_x })
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

    #[test_case(&[END],    &[END]    => Vec::<RowPairChange>::new())]
    #[test_case(&[0, END], &[END]    => vec![RowPairChange{ kind: RowPairChangeKind::Top,    x: 0 }])]
    #[test_case(&[END],    &[0, END] => vec![RowPairChange{ kind: RowPairChangeKind::Bottom, x: 0 }])]
    #[test_case(&[0, END], &[0, END] => vec![RowPairChange{ kind: RowPairChangeKind::Both,   x: 0 }])]
    #[test_case(
        &[1, 38, 39, 41, END],
        &[1, 2, 39, 42, END]
        => vec![
            RowPairChange{ kind: RowPairChangeKind::Both,   x: 1 },
            RowPairChange{ kind: RowPairChangeKind::Bottom, x: 2 },
            RowPairChange{ kind: RowPairChangeKind::Top,    x: 38 },
            RowPairChange{ kind: RowPairChangeKind::Both,   x: 39 },
            RowPairChange{ kind: RowPairChangeKind::Top,    x: 41 },
            RowPairChange{ kind: RowPairChangeKind::Bottom, x: 42 },
        ]
    )]
    fn test_pixel_row(run_top: &[i32], run_bottom: &[i32]) -> Vec<RowPairChange> {
        RowPairChangeIter::new(run_top, run_bottom).collect()
    }
}
