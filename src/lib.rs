#![doc = include_str!("../docs/cheatsheet.md")]
//!
//! ---
//!
//! Module layout: [`api`] (the `Stage` and shape/animation builders behind the prelude), [`world`]
//! (scene graph), [`primitives`] (solids, flat shapes, text), [`planar`] (2D regions and booleans),
//! [`text`] (fonts and LaTeX as outlines), [`animation`] (tweens, easing, morphing), [`render`]
//! (cameras, video export).

// Compile-check every Rust code block in the README as a doctest, so it can't drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

// The agent skill embeds the cheat sheet, so its code blocks are checked the same way.
#[cfg(doctest)]
#[doc = include_str!("../skills/rough-animations/SKILL.md")]
struct SkillDoctests;

pub mod animation;
pub mod api;
pub mod planar;
pub mod prelude;
pub mod primitives;
pub mod render;
pub mod text;
pub mod world;

#[cfg(feature = "spec")]
pub mod spec;
#[cfg(feature = "web")]
pub mod web;

pub use animation::{Animation, Easing};
pub use api::{Error, Result, Sketch, Stage};
pub use planar::Region;
pub use primitives::Geometry;
pub use render::camera::IsoCamera;
pub use render::export::{VideoFormat, VideoOptions};
pub use world::arena::Id;
pub use world::object::Object;
pub use world::scene::Scene;
pub use world::style::Style;
pub use world::transform::{Transform, Vec3};
