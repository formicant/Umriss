use image::{buffer::Pixels, Luma};

/// Marks the end of changes iteration.
/// Is greater than any coordinate.
pub const END: u32 = u32::MAX;

/// Iterates x coordinates in the row of pixels where changes occur, from left to right.
/// 
/// A _change_ occurs when the value of the pixel differs from the value of its neighbor.
/// The edge pixels that have no neighbors are compared to `edge_value` instead.
/// 
/// Binary pixel values assumed — all non-zero `Luma<u8>` values are considered the same.
/// 
/// Coordinates correspond to positions between the pixels:
/// - coordinate 0 is to the left of the 0th pixel,
/// - coordinate 1 is to the right of the 0th and to the left of the 1st pixel,
/// - . . .
/// - coordinate `width` is to the right of the last pixel of the row.
/// 
/// When there are no more changes, the iterator returns the `END` marker.
/// It is greater than any coordinate for the convenience of comparison.
pub struct RowChangeIter<'a> {
    row: Option<Pixels<'a, Luma<u8>>>,
    edge_value: bool,
    previous: bool,
    x: u32,
}

impl<'a> RowChangeIter<'a> {
    /// A row with no changes. Used as a padding row.
    pub fn empty() -> Self {
        Self { row: None, edge_value: false, previous: false, x: 0 }
    }
    
    /// Consumes a row of pixels and returns an iterator over its changes.
    pub fn from(row: Pixels<'a, Luma<u8>>, edge_value: bool) -> Self {
        Self { row: Some(row), edge_value, previous: edge_value, x: 0 }
    }
}

impl<'a> Iterator for RowChangeIter<'a> {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(row) = &mut self.row {
            // performance-critical
            while let Some(pixel) = row.next() {
                let x = self.x;
                let value = pixel[0] != 0;
                let differs = value != self.previous;
                self.previous = value;
                self.x += 1;
                if differs {
                    return Some(x);
                }
            }
            self.row = None;
            if self.previous != self.edge_value {
                return Some(self.x);
            }
        }
        if self.x != END {
            self.x = END;
            Some(END)
        } else {
            None
        }
    }
}


// ---------

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use ::image::GrayImage;
    use super::*;

    #[test]
    fn empty() {
        let actual = RowChangeIter::empty();
        let expected = vec![END];
        assert!(actual.eq(expected));
    }

    #[test_case(vec![1], false, vec![0, 1, END])]
    #[test_case(vec![1], true, vec![END])]
    #[test_case(vec![0, 0], false, vec![END])]
    #[test_case(vec![0, 0], true, vec![0, 2, END])]
    #[test_case(vec![1, 1], false, vec![0, 2, END])]
    #[test_case(vec![1, 1], true, vec![END])]
    #[test_case(vec![0, 1, 0], false, vec![1, 2, END])]
    #[test_case(vec![0, 1, 0], true, vec![0, 1, 2, 3, END])]
    #[test_case(vec![1, 1, 0, 0, 0, 1, 0, 1], false, vec![0, 2, 5, 6, 7, 8, END])]
    #[test_case(vec![1, 1, 0, 0, 0, 1, 0, 1], true, vec![2, 5, 6, 7, END])]
    fn pixel_row(row_pixels: Vec<u8>, edge_value: bool, expected: Vec<u32>) {
        let width = row_pixels.len() as u32;
        let image = GrayImage::from_vec(width, 1, row_pixels).unwrap();
        let row = image.rows().next().unwrap();
        let actual = RowChangeIter::from(row, edge_value);
        assert!(actual.eq(expected));
    }
}
