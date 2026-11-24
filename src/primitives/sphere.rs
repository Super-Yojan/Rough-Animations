use crate::world::bounds::Aabb;
use crate::primitives::common::{ring, shading};
use crate::primitives::{DrawCtx, Geometry};
use crate::world::style::{Style, shade};
use crate::world::transform::{Transform, Vec3};
use euclid::default::Point2D;
use std::f32::consts::TAU;

const SEGMENTS: usize = 48;

/// How much the isometric projection magnifies lengths (axes are drawn at full length, not
/// the true-isometric 0.816).
const ISO_SCALE: f32 = 1.224_744_9;

/// Sphere centred on its origin. Drawn as an outline circle plus a front-facing equator arc.
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Geometry for Sphere {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        // A sphere looks the same under rotation, so only position and scale matter.
        let r = Vec3::new(
            self.radius * t.scale.x.abs(),
            self.radius * t.scale.y.abs(),
            self.radius * t.scale.z.abs(),
        );
        Some(Aabb {
            min: t.position - r,
            max: t.position + r,
        })
    }

    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        let center = ctx.project(t.position);
        let mean_scale = (t.scale.x + t.scale.y + t.scale.z) / 3.0;
        let r = self.radius * mean_scale * ISO_SCALE;

        let outline: Vec<_> = (0..SEGMENTS)
            .map(|i| {
                let a = TAU * i as f32 / SEGMENTS as f32;
                Point2D::new(center.x + r * a.cos(), center.y + r * a.sin())
            })
            .collect();
        // A sphere has no flat faces; light it like the top-left-front, between box tones.
        let fill = style.fill.map(|c| shade(c, shading(Vec3::new(0.3, 0.7, 0.65).normalize())));
        ctx.rough_polygon(&outline, fill, style);

        // Equator, only the half facing the camera (the part with positive depth).
        let world: Vec<Vec3> = ring(self.radius, 0.0, SEGMENTS)
            .into_iter()
            .map(|p| t.apply(p))
            .collect();
        let front: Vec<bool> = world
            .iter()
            .map(|p| {
                let d = *p - t.position;
                d.x + d.y + d.z > 0.0
            })
            .collect();
        if let Some(start) = (0..SEGMENTS).find(|&i| front[i] && !front[(i + SEGMENTS - 1) % SEGMENTS])
        {
            let arc: Vec<_> = (0..SEGMENTS)
                .map(|k| (start + k) % SEGMENTS)
                .take_while(|&i| front[i])
                .map(|i| ctx.project(world[i]))
                .collect();
            ctx.rough_polyline(&arc, style);
        }
    }
}
