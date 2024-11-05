mod polygon;
mod orthopolygon;

use std::fmt::Debug;
use euclid::num::{Floor, Ceil, Round};
use num_traits::{NumAssign, cast::NumCast};
pub use polygon::{Polygon, Polygonlike};
pub use orthopolygon::{Orthopolygon, Orthopolygonlike, PointPosition};

pub trait Number: Copy + PartialOrd + NumAssign + NumCast + Floor + Ceil + Round + Debug { }
impl<T> Number for T where T: Copy + PartialOrd + NumAssign + NumCast + Floor + Ceil + Round + Debug { }
