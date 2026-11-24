//! The high-level, low-token API: `Stage`, shape constructors, colours by name, short
//! animation builders. Everything here is a thin layer over the lower-level modules.

pub mod color;
pub mod error;
pub mod num;
pub mod shapes;
pub mod sketch;
pub mod stage;

pub use color::IntoColor;
pub use error::{Error, Result};
pub use num::{Num, Pos, deg};
pub use shapes::*;
pub use sketch::Sketch;
pub use stage::Stage;
