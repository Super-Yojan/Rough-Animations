//! Planar (2D) geometry: [`Region`]s and boolean operations on them.

mod boolean;
mod morph;
mod region;

pub use morph::RegionMorph;
pub use region::{Point2, Region, signed_area};
