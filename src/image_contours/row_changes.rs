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
                let value = pixel[0] >= 128;
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
