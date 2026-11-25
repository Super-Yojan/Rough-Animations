use crate::world::bounds::Aabb;
use crate::primitives::common::{convex_hull, ring, shading};
use crate::primitives::{DrawCtx, Geometry};
use crate::world::style::{Style, shade};
use crate::world::transform::{Transform, Vec3};

const SEGMENTS: usize = 40;

/// Draw a cone frustum along local Y (from `-height/2` to `+height/2`), centred on the origin.
/// A cylinder has equal radii; a cone has a top radius of 0.
fn draw_frustum(
    ctx: &mut DrawCtx,
    t: &Transform,
    style: &Style,
    r_bottom: f32,
    r_top: f32,
    height: f32,
) {
    let h = height / 2.0;
    let bottom: Vec<_> = ring(r_bottom, -h, SEGMENTS)
        .into_iter()
        .map(|p| ctx.project(t.apply(p)))
        .collect();
    let top: Vec<_> = ring(r_top, h, SEGMENTS)
        .into_iter()
        .map(|p| ctx.project(t.apply(p)))
        .collect();

    // The silhouette of a frustum is the convex hull of its two end circles.
    let all: Vec<_> = bottom.iter().chain(&top).copied().collect();
    let hull = convex_hull(&all);
    if hull.len() >= 3 {
        ctx.rough_polygon(&hull, style.fill.map(|c| shade(c, 0.95)), style);
    }

    // Caps that face the camera, with the same lighting as a box's top / bottom.
    for (pts, normal, radius) in [
        (&bottom, Vec3::new(0.0, -1.0, 0.0), r_bottom),
        (&top, Vec3::new(0.0, 1.0, 0.0), r_top),
    ] {
        let n = t.rotate_dir(normal);
        if radius > 1e-4 && ctx.camera.faces_viewer(n) {
            let fill = style.fill.map(|c| shade(c, shading(n)));
            ctx.rough_polygon(pts, fill, style);
        }
    }
}

/// Upright cylinder centred on its origin.
#[derive(Clone, Copy, Debug)]
pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
}

impl Cylinder {
    pub fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }
}

impl Geometry for Cylinder {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        let h = Vec3::new(self.radius, self.height / 2.0, self.radius);
        Some(Aabb::of_local_box(t, -h, h))
    }

    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        draw_frustum(ctx, t, style, self.radius, self.radius, self.height);
    }
}

/// Upright cone, base at the bottom, apex at the top, centred on its origin.
#[derive(Clone, Copy, Debug)]
pub struct Cone {
    pub radius: f32,
    pub height: f32,
}

impl Cone {
    pub fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }
}

impl Geometry for Cone {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        let h = Vec3::new(self.radius, self.height / 2.0, self.radius);
        Some(Aabb::of_local_box(t, -h, h))
    }

    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        draw_frustum(ctx, t, style, self.radius, 0.0, self.height);
    }
}
