//! Animations change object properties over time.
//!
//! An [`Animation`] is driven by local time in seconds. It must be called with
//! non-decreasing times, starting with [`Animation::begin`]; start values are captured
//! there, so chained animations (a move after a move) compose naturally.

mod anim;
mod animate;
pub mod easing;
mod group;
mod morph;

pub use animate::{
    Animate, create, fade_in, fade_out, fill_to, move_by, move_from, move_to, opacity_to, rotate_by, rotate_to,
    scale_to, stroke_to, uncreate, write,
};
pub use easing::Easing;
pub use morph::{IntoRegion, Morph, TransformInto, morph, transform_into};
pub use anim::{Anim, all, lag, seq};
pub use group::{Group, wait};

use crate::world::arena::Id;
use crate::world::scene::Scene;

pub trait Animation {
    /// Total length in seconds.
    fn duration(&self) -> f32;

    /// Called once, right before the first `update`. Capture start values here.
    fn begin(&mut self, _scene: &mut Scene) {}

    /// Apply the state at local `time`, already clamped to `[0, duration]`.
    fn update(&mut self, scene: &mut Scene, time: f32);

    /// Change the easing, for animations that have one. Others ignore it.
    fn set_easing(&mut self, _easing: Easing) {}

    /// Objects this animation brings onto the screen (`create`, `fade_in`, `write`). The stage
    /// hides them from frame 0 until the animation starts, so they needn't be added hidden.
    fn introduces(&self, _out: &mut Vec<Id>) {}

    /// Objects that only serve as a description of something else and should never show up
    /// themselves (the target of `transform_into`). The stage hides them from frame 0.
    fn consumes(&self, _out: &mut Vec<Id>) {}
}

impl Animation for Box<dyn Animation> {
    fn duration(&self) -> f32 {
        (**self).duration()
    }
    fn begin(&mut self, scene: &mut Scene) {
        (**self).begin(scene)
    }
    fn update(&mut self, scene: &mut Scene, time: f32) {
        (**self).update(scene, time)
    }
    fn set_easing(&mut self, easing: Easing) {
        (**self).set_easing(easing)
    }
    fn introduces(&self, out: &mut Vec<Id>) {
        (**self).introduces(out)
    }
    fn consumes(&self, out: &mut Vec<Id>) {
        (**self).consumes(out)
    }
}

/// Linear interpolation between two values of the same type.
pub trait Lerp: Copy {
    fn lerp(self, to: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, to: f32, t: f32) -> f32 {
        self + (to - self) * t
    }
}

impl Lerp for crate::world::transform::Vec3 {
    fn lerp(self, to: Self, t: f32) -> Self {
        self + (to - self) * t
    }
}

impl Lerp for tiny_skia::Color {
    fn lerp(self, to: Self, t: f32) -> Self {
        tiny_skia::Color::from_rgba(
            self.red().lerp(to.red(), t).clamp(0.0, 1.0),
            self.green().lerp(to.green(), t).clamp(0.0, 1.0),
            self.blue().lerp(to.blue(), t).clamp(0.0, 1.0),
            self.alpha().lerp(to.alpha(), t).clamp(0.0, 1.0),
        )
        .unwrap_or(to)
    }
}
