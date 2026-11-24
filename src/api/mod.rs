//! The high-level, low-token API: `Stage`, shape constructors, colours by name, short
//! animation builders. Everything here is a thin layer over the lower-level modules.

pub mod color;
pub mod error;
pub mod group;
pub mod layout;
pub mod num;
pub mod shapes;
pub mod sketch;
pub mod stage;
pub mod theme;

pub use color::IntoColor;
pub use error::{Error, Result};
pub use group::Group;
pub use layout::{IntoTag, Side};
pub use num::{Num, Pos, deg};
pub use shapes::*;
pub use sketch::Sketch;
pub use stage::Stage;
pub use theme::{IntoTheme, Theme};
