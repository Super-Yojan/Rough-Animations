use super::{Anim, Animation, Lerp};
use crate::api::{IntoColor, Num, Pos};
use crate::animation::easing::Easing;
use crate::world::arena::Id;
use crate::world::scene::Scene;
use crate::world::transform::Vec3;
use tiny_skia::Color;

/// A value to tween toward: absolute, or relative to the value at `begin`.
#[derive(Clone, Copy, Debug)]
enum Target<T> {
    Abs(T),
    Rel(T),
}

/// A resolved tween: start and end captured at `begin`.
#[derive(Clone, Copy)]
struct Span<T> {
    from: T,
    to: T,
}

impl<T: Lerp> Span<T> {
    fn at(&self, t: f32) -> T {
        self.from.lerp(self.to, t)
    }
}

/// Tweens any combination of an object's properties over a duration.
///
/// Usually built with the helper functions ([`move_to`], [`fade_in`], ...) and refined with
/// [`Animate::duration`] / [`Animate::ease`]. Calls can be chained to animate several
/// properties at once: `move_to(id, p).opacity(0.5)`.
pub struct Animate {
    id: Id,
    duration: f32,
    easing: Easing,
    show_on_begin: bool,

    position: Option<Target<Vec3>>,
    rotation: Option<Target<Vec3>>,
    scale: Option<Target<Vec3>>,
    opacity: Option<Target<f32>>,
    /// Start opacity override (fade-in starts from 0 regardless of the current value).
    opacity_from: Option<f32>,
    /// Start this far from the object's position and settle onto it.
    position_start_offset: Option<Vec3>,
    reveal: Option<f32>,
    reveal_from: Option<f32>,
    stroke_width: Option<Target<f32>>,
    fill: Option<Color>,
    stroke: Option<Color>,

    // Resolved in `begin`.
    r_position: Option<Span<Vec3>>,
    r_rotation: Option<Span<Vec3>>,
    r_scale: Option<Span<Vec3>>,
    r_opacity: Option<Span<f32>>,
    r_reveal: Option<Span<f32>>,
    r_stroke_width: Option<Span<f32>>,
    r_fill: Option<Span<Color>>,
    r_stroke: Option<Span<Color>>,
}

impl Animate {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            duration: 1.0,
            easing: Easing::default(),
            show_on_begin: false,
            position: None,
            rotation: None,
            scale: None,
            opacity: None,
            opacity_from: None,
            position_start_offset: None,
            reveal: None,
            reveal_from: None,
            stroke_width: None,
            fill: None,
            stroke: None,
            r_position: None,
            r_rotation: None,
            r_scale: None,
            r_opacity: None,
            r_reveal: None,
            r_stroke_width: None,
            r_fill: None,
            r_stroke: None,
        }
    }

    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = seconds.max(0.0);
        self
    }

    pub fn ease(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn position(mut self, to: impl Into<Vec3>) -> Self {
        self.position = Some(Target::Abs(to.into()));
        self
    }

    pub fn position_by(mut self, delta: impl Into<Vec3>) -> Self {
        self.position = Some(Target::Rel(delta.into()));
        self
    }

    /// Euler angles in radians.
    pub fn rotation(mut self, to: impl Into<Vec3>) -> Self {
        self.rotation = Some(Target::Abs(to.into()));
        self
    }

    pub fn rotation_by(mut self, delta: impl Into<Vec3>) -> Self {
        self.rotation = Some(Target::Rel(delta.into()));
        self
    }

    pub fn scale(mut self, to: impl Into<Vec3>) -> Self {
        self.scale = Some(Target::Abs(to.into()));
        self
    }

    pub fn opacity(mut self, to: f32) -> Self {
        self.opacity = Some(Target::Abs(to.clamp(0.0, 1.0)));
        self
    }

    /// How much of the shape is drawn, 0.0 to 1.0.
    pub fn reveal(mut self, to: f32) -> Self {
        self.reveal = Some(to.clamp(0.0, 1.0));
        self
    }

    pub fn stroke_width(mut self, to: f32) -> Self {
        self.stroke_width = Some(Target::Abs(to));
        self
    }

    /// Fill colour. If the object has no fill yet, it fades in from fully transparent.
    pub fn fill(mut self, to: Color) -> Self {
        self.fill = Some(to);
        self
    }

    pub fn stroke(mut self, to: Color) -> Self {
        self.stroke = Some(to);
        self
    }
}

fn resolve<T: Lerp + std::ops::Add<Output = T>>(from: T, target: Target<T>) -> Span<T> {
    match target {
        Target::Abs(to) => Span { from, to },
        Target::Rel(d) => Span { from, to: from + d },
    }
}

impl Animation for Animate {
    fn duration(&self) -> f32 {
        self.duration
    }

    fn set_easing(&mut self, easing: Easing) {
        self.easing = easing;
    }

    fn introduces(&self, out: &mut Vec<Id>) {
        if self.show_on_begin {
            out.push(self.id);
        }
    }

    fn begin(&mut self, scene: &mut Scene) {
        let Some(o) = scene.get_mut(self.id) else {
            return;
        };
        if self.show_on_begin {
            o.visible = true;
        }
        let (t, s) = (o.transform, o.style);

        self.r_position = self.position.map(|x| resolve(t.position, x));
        if let Some(offset) = self.position_start_offset {
            self.r_position = Some(Span {
                from: t.position + offset,
                to: t.position,
            });
        }
        self.r_rotation = self.rotation.map(|x| resolve(t.rotation, x));
        self.r_scale = self.scale.map(|x| match x {
            Target::Abs(to) => Span { from: t.scale, to },
            Target::Rel(d) => Span {
                from: t.scale,
                to: t.scale + d,
            },
        });
        self.r_opacity = self.opacity.map(|x| {
            let mut span = resolve(s.opacity, x);
            if let Some(from) = self.opacity_from {
                span.from = from;
            }
            span
        });
        self.r_reveal = self.reveal.map(|to| Span {
            from: self.reveal_from.unwrap_or(s.reveal),
            to,
        });
        self.r_stroke_width = self.stroke_width.map(|x| resolve(s.stroke_width, x));
        self.r_fill = self.fill.map(|to| {
            let from = s.fill.unwrap_or_else(|| {
                let mut c = to;
                c.set_alpha(0.0);
                c
            });
            Span { from, to }
        });
        self.r_stroke = self.stroke.map(|to| Span { from: s.stroke, to });
    }

    fn update(&mut self, scene: &mut Scene, time: f32) {
        let Some(o) = scene.get_mut(self.id) else {
            return;
        };
        let raw = if self.duration > 0.0 {
            time / self.duration
        } else {
            1.0
        };
        let t = self.easing.apply(raw);

        if let Some(s) = self.r_position {
            o.transform.position = s.at(t);
        }
        if let Some(s) = self.r_rotation {
            o.transform.rotation = s.at(t);
        }
        if let Some(s) = self.r_scale {
            o.transform.scale = s.at(t);
        }
        if let Some(s) = self.r_opacity {
            o.style.opacity = s.at(t).clamp(0.0, 1.0);
        }
        if let Some(s) = self.r_reveal {
            o.style.reveal = s.at(t).clamp(0.0, 1.0);
        }
        if let Some(s) = self.r_stroke_width {
            o.style.stroke_width = s.at(t);
        }
        if let Some(s) = self.r_fill {
            o.style.fill = Some(s.at(t));
        }
        if let Some(s) = self.r_stroke {
            o.style.stroke = s.at(t);
        }
    }
}

/// Move to an absolute position.
pub fn move_to(id: Id, to: impl Pos) -> Anim {
    Anim::new(Animate::new(id).position(to.pos()))
}

/// Start `offset` away from where the object is now and settle onto its current position:
/// `move_from(id, (0, 80, 0))` drops it in from 80 units above. Combine with `fade_in` and
/// `.with(..)` for an entrance.
pub fn move_from(id: Id, offset: impl Pos) -> Anim {
    let mut a = Animate::new(id);
    a.position_start_offset = Some(offset.pos());
    Anim::new(a)
}

/// Move by an offset from wherever the object is when the animation begins.
pub fn move_by(id: Id, delta: impl Pos) -> Anim {
    Anim::new(Animate::new(id).position_by(delta.pos()))
}

/// Rotate to absolute Euler angles (radians; see [`deg`](crate::api::deg)).
pub fn rotate_to(id: Id, to: impl Pos) -> Anim {
    Anim::new(Animate::new(id).rotation(to.pos()))
}

/// Rotate by Euler angles (radians) relative to the current rotation.
pub fn rotate_by(id: Id, delta: impl Pos) -> Anim {
    Anim::new(Animate::new(id).rotation_by(delta.pos()))
}

/// Scale to `(sx, sy, sz)`, e.g. `scale_to(id, (2, 2, 2))`.
pub fn scale_to(id: Id, to: impl Pos) -> Anim {
    Anim::new(Animate::new(id).scale(to.pos()))
}

/// Fade from transparent to fully opaque. The object is hidden until the fade starts.
pub fn fade_in(id: Id) -> Anim {
    let mut a = Animate::new(id).opacity(1.0);
    a.opacity_from = Some(0.0);
    a.show_on_begin = true;
    Anim::new(a)
}

/// Fade to fully transparent. The object stays in the scene (invisible).
pub fn fade_out(id: Id) -> Anim {
    Anim::new(Animate::new(id).opacity(0.0))
}

pub fn opacity_to(id: Id, to: impl Num) -> Anim {
    Anim::new(Animate::new(id).opacity(to.f32()))
}

pub fn fill_to(id: Id, to: impl IntoColor) -> Anim {
    Anim::new(Animate::new(id).fill(to.into_color()))
}

pub fn stroke_to(id: Id, to: impl IntoColor) -> Anim {
    Anim::new(Animate::new(id).stroke(to.into_color()))
}

/// Draw the object onto the screen like a pen would: the outline is traced, then the fill
/// fades in. The object is hidden until the animation starts.
pub fn create(id: Id) -> Anim {
    let mut a = Animate::new(id).reveal(1.0);
    a.reveal_from = Some(0.0);
    a.show_on_begin = true;
    Anim::new(a)
}

/// The reverse of [`create`]: the fill fades out, then the outline is un-traced.
pub fn uncreate(id: Id) -> Anim {
    Anim::new(Animate::new(id).reveal(0.0))
}

/// Write text (or any shape) onto the screen: an alias for [`create`]. For
/// [`Text`](crate::primitives::Text) and [`Tex`](crate::primitives::Tex) the glyphs are inked
/// one after another.
pub fn write(id: Id) -> Anim {
    create(id)
}
