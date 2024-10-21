use image::{buffer::Pixels, Luma};

pub const END: u32 = u32::MAX;

pub struct RowChanges<'a> {
    row: Option<Pixels<'a, Luma<u8>>>,
    previous: bool,
    x: u32,
}

impl<'a> RowChanges<'a> {
    pub fn empty() -> Self {
        Self { row: None, previous: false, x: 0 }
    }
    
    pub fn from(row: Pixels<'a, Luma<u8>>) -> Self {
        Self { row: Some(row), previous: false, x: 0 }
    }
}

impl<'a> Iterator for RowChanges<'a> {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(row) = &mut self.row {
            while let Some(pixel) = row.next() {
                let x = self.x;
                self.x += 1;
                let value = pixel[0] != 0;
                let differs = value != self.previous;
                self.previous = value;
                if differs {
                    return Some(x);
                }
            }
            self.row = None;
            if self.previous {
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
        let actual = RowChanges::empty();
        let expected = vec![END];
        assert!(actual.eq(expected));
    }

    #[test_case(vec![1], vec![0, 1, END])]
    #[test_case(vec![0, 0], vec![END])]
    #[test_case(vec![1, 1], vec![0, 2, END])]
    #[test_case(vec![0, 1, 0], vec![1, 2, END])]
    #[test_case(vec![1, 1, 0, 0, 0, 1, 0, 1], vec![0, 2, 5, 6, 7, 8, END])]
    fn pixel_row(row_pixels: Vec<u8>, expected: Vec<u32>) {
        let width = row_pixels.len() as u32;
        let image = GrayImage::from_vec(width, 1, row_pixels).unwrap();
        let row = image.rows().next().unwrap();
        let actual = RowChanges::from(row);
        assert!(actual.eq(expected));
    }
}
