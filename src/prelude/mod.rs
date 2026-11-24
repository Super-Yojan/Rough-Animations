//! `use rough_animations::prelude::*;` is all a script needs.

// Stage, look, errors, numbers and colours.
pub use crate::api::{
    Error, Group, IntoColor, IntoTag, IntoTheme, Num, Pos, Result, Side, Sketch, Stage, Theme, deg,
};

// Shapes: cuboid, cube, cylinder, cone, sphere, circle, ellipse, rect, ngon, star, polygon,
// prism, line, arrow, text, label, title, math, latex.
pub use crate::api::shapes::*;

// Animations.
pub use crate::animation::{
    Anim, Easing, all, clear, create, create_all, fade_in, fade_out, fill_to, lag, morph, move_by, move_from, move_to,
    opacity_to, rotate_by, rotate_to, scale_to, seq, stroke_to, transform_into, uncreate, wait,
    write, write_all,
};

// Handles and a few types you name explicitly.
pub use crate::primitives::Plane;
pub use crate::text::{Align, Font};
pub use crate::world::arena::Id;
pub use crate::world::object::Object;
