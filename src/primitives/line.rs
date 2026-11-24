use crate::world::bounds::Aabb;
use crate::primitives::{DrawCtx, Geometry};
use crate::world::style::Style;
use crate::world::transform::{Transform, Vec3};
use euclid::default::Point2D;

/// Straight segment between two local-space points. Stroke only.
#[derive(Clone, Copy, Debug)]
pub struct Line3D {
    pub from: Vec3,
    pub to: Vec3,
}

impl Line3D {
    pub fn new(from: impl Into<Vec3>, to: impl Into<Vec3>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl Geometry for Line3D {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        Aabb::from_points([t.apply(self.from), t.apply(self.to)])
    }

    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        let (a, b) = (ctx.project(t.apply(self.from)), ctx.project(t.apply(self.to)));
        ctx.rough_polyline(&[a, b], style);
    }

    fn depth(&self, t: &Transform) -> f32 {
        let m = t.apply((self.from + self.to) * 0.5);
        m.x + m.y + m.z
    }
}

/// A [`Line3D`] with an open arrowhead at `to`. The head is drawn flat on the screen so it
/// always reads clearly.
#[derive(Clone, Copy, Debug)]
pub struct Arrow3D {
    pub from: Vec3,
    pub to: Vec3,
    /// Length of the arrowhead strokes, in pixels.
    pub head: f32,
}

impl Arrow3D {
    pub fn new(from: impl Into<Vec3>, to: impl Into<Vec3>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            head: 16.0,
        }
    }

    pub fn head(mut self, pixels: f32) -> Self {
        self.head = pixels;
        self
    }
}

impl Geometry for Arrow3D {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        Aabb::from_points([t.apply(self.from), t.apply(self.to)])
    }

    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        let (a, b) = (ctx.project(t.apply(self.from)), ctx.project(t.apply(self.to)));
        ctx.rough_polyline(&[a, b], style);

        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-3 {
            return;
        }
        let (ux, uy) = (dx / len, dy / len);
        let head = self.head.min(len * 0.6);
        let wing = |sign: f32| {
            let (s, c) = (sign * 28.0_f32.to_radians()).sin_cos();
            // Rotate the reversed direction by ±28°.
            let (rx, ry) = (-ux * c + uy * s, -ux * s - uy * c);
            Point2D::new(b.x + rx * head, b.y + ry * head)
        };
        ctx.rough_polyline(&[wing(1.0), b, wing(-1.0)], style);
    }

    fn depth(&self, t: &Transform) -> f32 {
        let m = t.apply((self.from + self.to) * 0.5);
        m.x + m.y + m.z
    }
}
