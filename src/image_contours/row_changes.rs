use image::{Luma, buffer::Pixels};

pub const END: u32 = u32::MAX;

pub struct RowChanges<'a> {
    row: Pixels<'a, Luma<u8>>,
    can_iterate: bool,
    previous: bool,
    x: u32,
}

impl<'a> RowChanges<'a> {
    pub fn from(row: Pixels<'a, Luma<u8>>) -> Self {
        Self { row, can_iterate: true, previous: false, x: 0 }
    }
}

impl<'a> Iterator for RowChanges<'a> {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.can_iterate {
            while let Some(pixel) = self.row.next() {
                let x = self.x;
                let value = pixel[0] >= 128;
                let differs = value != self.previous;
                self.previous = value;
                self.x += 1;
                if differs {
                    return Some(x);
                }
            }
            self.can_iterate = false;
            if self.previous {
                return Some(self.x);
            }
        }
        if self.x != END {
            self.x = END;
            return Some(END);
        }
        None
    }
}
