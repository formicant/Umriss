mod polygon;
mod orthopolygon;

use num_traits::{NumAssign, cast::NumCast};
pub use polygon::{Polygon, Polygonlike};
pub use orthopolygon::{Orthopolygon, Orthopolygonlike, PointPosition};

pub trait Number: Copy + Ord + NumAssign + NumCast { }
impl<T> Number for T where T: Copy + Ord + NumAssign + NumCast { }
