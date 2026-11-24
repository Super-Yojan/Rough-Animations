use crate::world::bounds::Aabb;
use crate::planar::Region;
use crate::primitives::{DrawCtx, Geometry};
use crate::world::style::Style;
use crate::world::transform::{Transform, Vec3};

/// Which plane a 2D shape lies in, in the object's local space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Plane {
    /// The floor: shape `(u, v)` maps to `(x, 0, -z)`, so `+v` points away from the viewer.
    /// Drawn behind everything standing on it.
    #[default]
    Ground,
    /// A wall facing the right-hand side of the screen: `(u, v)` maps to `(u, v, 0)`.
    Front,
    /// A wall facing the left-hand side of the screen: `(u, v)` maps to `(0, v, -u)`.
    Side,
}

impl Plane {
    pub fn to_3d(self, [u, v]: [f32; 2]) -> Vec3 {
        match self {
            Plane::Ground => Vec3::new(u, 0.0, -v),
            Plane::Front => Vec3::new(u, v, 0.0),
            Plane::Side => Vec3::new(0.0, v, -u),
        }
    }
}

/// A flat 2D shape (circle, rectangle, polygon, or the result of a boolean operation)
/// lying in a [`Plane`]. Build the [`Region`] with `Region::circle(..)` and friends, combine
/// regions with `|`, `&`, `-`, `^`, then wrap the result: `Shape2D::new(region)`.
#[derive(Clone, Debug)]
pub struct Shape2D {
    pub region: Region,
    pub plane: Plane,
}

impl Shape2D {
    pub fn new(region: Region) -> Self {
        Self {
            region,
            plane: Plane::default(),
        }
    }

    pub fn circle(radius: f32) -> Self {
        Self::new(Region::circle(radius))
    }

    pub fn ellipse(rx: f32, ry: f32) -> Self {
        Self::new(Region::ellipse(rx, ry))
    }

    pub fn rect(width: f32, height: f32) -> Self {
        Self::new(Region::rect(width, height))
    }

    pub fn regular_polygon(sides: usize, radius: f32) -> Self {
        Self::new(Region::regular_polygon(sides, radius))
    }

    pub fn star(points: usize, outer: f32, inner: f32) -> Self {
        Self::new(Region::star(points, outer, inner))
    }

    /// Choose the plane the shape lies in.
    pub fn on(mut self, plane: Plane) -> Self {
        self.plane = plane;
        self
    }
}

impl From<Region> for Shape2D {
    fn from(region: Region) -> Self {
        Shape2D::new(region)
    }
}

impl Geometry for Shape2D {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        let (lo, hi) = self.region.bounds()?;
        let corners = [[lo[0], lo[1]], [hi[0], lo[1]], [lo[0], hi[1]], [hi[0], hi[1]]];
        Aabb::from_points(corners.map(|c| t.apply(self.plane.to_3d(c))))
    }

    fn region(&self) -> Option<&Region> {
        Some(&self.region)
    }

    fn set_plane(&mut self, plane: Plane) -> bool {
        self.plane = plane;
        true
    }

    fn region_mut(&mut self) -> Option<&mut Region> {
        Some(&mut self.region)
    }

    fn draw(&self, ctx: &mut DrawCtx, transform: &Transform, style: &Style) {
        let contours: Vec<Vec<_>> = self
            .region
            .contours
            .iter()
            .map(|c| {
                c.iter()
                    .map(|&p| ctx.project(transform.apply(self.plane.to_3d(p))))
                    .collect()
            })
            .collect();
        ctx.rough_path(&contours, style.fill, style);
    }
}
