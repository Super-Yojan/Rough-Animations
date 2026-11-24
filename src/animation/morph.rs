use super::{Anim, Animate, Animation};
use crate::world::object::Object;
use crate::planar::{Region, RegionMorph};
use crate::world::arena::Id;
use crate::animation::easing::Easing;
use crate::world::scene::Scene;

/// Morphs an object's shape into a target [`Region`]. Works on objects whose geometry is built
/// from a region: [`Shape2D`](crate::primitives::Shape2D) and [`Prism`](crate::primitives::Prism).
///
/// Position, colour and the rest are untouched; combine with other animations in a
/// [`all`](super::all) group, or use [`transform_into`] to take on another object's
/// look entirely.
pub struct Morph {
    id: Id,
    target: Region,
    duration: f32,
    easing: Easing,
    plan: Option<RegionMorph>,
}

impl Morph {
    pub fn new(id: Id, target: Region) -> Self {
        Self {
            id,
            target,
            duration: 1.0,
            easing: Easing::default(),
            plan: None,
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
}

impl Animation for Morph {
    fn duration(&self) -> f32 {
        self.duration
    }

    fn set_easing(&mut self, easing: Easing) {
        self.easing = easing;
    }

    fn begin(&mut self, scene: &mut Scene) {
        let Some(o) = scene.get(self.id) else {
            return;
        };
        let from = o
            .geometry
            .region()
            .expect("morph: this object's geometry has no region (use Shape2D or Prism)");
        self.plan = Some(RegionMorph::new(from, &self.target));
    }

    fn update(&mut self, scene: &mut Scene, time: f32) {
        let (Some(plan), Some(o)) = (&self.plan, scene.get_mut(self.id)) else {
            return;
        };
        let raw = if self.duration > 0.0 { time / self.duration } else { 1.0 };
        let t = self.easing.apply(raw);
        if let Some(r) = o.geometry.region_mut() {
            *r = plan.at(t);
        }
    }
}

/// Anything that has an outline to morph into: a [`Region`], or a flat shape such as
/// `star(5, 75, 32)` (it is only used for its outline and is not added to the scene).
pub trait IntoRegion {
    fn into_region(self) -> Region;
}

impl IntoRegion for Region {
    fn into_region(self) -> Region {
        self
    }
}

impl IntoRegion for &Region {
    fn into_region(self) -> Region {
        self.clone()
    }
}

impl IntoRegion for Object {
    fn into_region(self) -> Region {
        self.geometry
            .region()
            .expect("morph target must be a flat shape (circle, rect, star, ...)")
            .clone()
    }
}

/// Morph `id`'s shape into `target`: `morph(id, star(5, 75, 32))`. Works on flat shapes and
/// prisms.
pub fn morph(id: Id, target: impl IntoRegion) -> Anim {
    Anim::new(Morph::new(id, target.into_region()))
}

/// Turn object `from` into object `to`, like manim's `Transform`: its shape morphs into
/// `to`'s shape, and its position, rotation, scale, colours, stroke width and opacity tween to
/// `to`'s. `to` itself is left alone, so add it with `.hidden()` unless you want both on screen.
///
/// Both objects must have a region (`Shape2D` or `Prism`).
pub struct TransformInto {
    from: Id,
    to: Id,
    duration: f32,
    easing: Easing,
    morph: Option<Morph>,
    props: Option<Animate>,
}

impl TransformInto {
    pub fn new(from: Id, to: Id) -> Self {
        Self {
            from,
            to,
            duration: 1.0,
            easing: Easing::default(),
            morph: None,
            props: None,
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
}

impl Animation for TransformInto {
    fn duration(&self) -> f32 {
        self.duration
    }

    fn set_easing(&mut self, easing: Easing) {
        self.easing = easing;
    }

    fn consumes(&self, out: &mut Vec<Id>) {
        out.push(self.to);
    }

    fn begin(&mut self, scene: &mut Scene) {
        let Some(target) = scene.get(self.to) else {
            return;
        };
        let region = target
            .geometry
            .region()
            .expect("transform_into: the target's geometry has no region (use Shape2D or Prism)")
            .clone();
        let (t, s) = (target.transform, target.style);

        let mut props = Animate::new(self.from)
            .position(t.position)
            .rotation(t.rotation)
            .scale(t.scale)
            .opacity(s.opacity)
            .stroke(s.stroke)
            .stroke_width(s.stroke_width)
            .duration(self.duration)
            .ease(self.easing);
        if let Some(fill) = s.fill {
            props = props.fill(fill);
        }
        props.begin(scene);

        let mut morph = Morph::new(self.from, region).duration(self.duration).ease(self.easing);
        morph.begin(scene);
        self.props = Some(props);
        self.morph = Some(morph);
    }

    fn update(&mut self, scene: &mut Scene, time: f32) {
        if let (Some(m), Some(p)) = (&mut self.morph, &mut self.props) {
            m.update(scene, time);
            p.update(scene, time);
        }
    }
}

/// See [`TransformInto`]. The target `to` is hidden for you.
pub fn transform_into(from: Id, to: Id) -> Anim {
    Anim::new(TransformInto::new(from, to))
}
