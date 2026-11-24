use crate::world::transform::{Transform, Vec3};

/// Axis-aligned bounding box in world space, used to decide which object is in front.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

/// Boxes closer than this along an axis count as touching.
const EPS: f32 = 1e-3;

impl Aabb {
    pub fn point(p: Vec3) -> Self {
        Self { min: p, max: p }
    }

    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut it = points.into_iter();
        let first = it.next()?;
        Some(it.fold(Self::point(first), |b, p| Self {
            min: Vec3::new(b.min.x.min(p.x), b.min.y.min(p.y), b.min.z.min(p.z)),
            max: Vec3::new(b.max.x.max(p.x), b.max.y.max(p.y), b.max.z.max(p.z)),
        }))
    }

    /// Bounds of the local-space box `lo..hi` after applying `t`.
    pub fn of_local_box(t: &Transform, lo: Vec3, hi: Vec3) -> Self {
        let corners = [
            Vec3::new(lo.x, lo.y, lo.z),
            Vec3::new(hi.x, lo.y, lo.z),
            Vec3::new(lo.x, hi.y, lo.z),
            Vec3::new(hi.x, hi.y, lo.z),
            Vec3::new(lo.x, lo.y, hi.z),
            Vec3::new(hi.x, lo.y, hi.z),
            Vec3::new(lo.x, hi.y, hi.z),
            Vec3::new(hi.x, hi.y, hi.z),
        ];
        Self::from_points(corners.map(|c| t.apply(c))).expect("eight corners")
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn corners(&self) -> [Vec3; 8] {
        let (lo, hi) = (self.min, self.max);
        [
            Vec3::new(lo.x, lo.y, lo.z),
            Vec3::new(hi.x, lo.y, lo.z),
            Vec3::new(lo.x, hi.y, lo.z),
            Vec3::new(hi.x, hi.y, lo.z),
            Vec3::new(lo.x, lo.y, hi.z),
            Vec3::new(hi.x, lo.y, hi.z),
            Vec3::new(lo.x, hi.y, hi.z),
            Vec3::new(hi.x, hi.y, hi.z),
        ]
    }

    /// Whether `self` lies entirely on the far side of `other` along some axis.
    ///
    /// The isometric camera looks along (1, 1, 1) from the +x, +y, +z side, and two points
    /// that land on the same pixel differ by `t * (1, 1, 1)`. If `self` is entirely below
    /// `other` in x, y or z, then `t >= 0` for every such pair, so `other` is the nearer one
    /// wherever they overlap on screen.
    pub fn behind(&self, other: &Aabb) -> bool {
        self.max.x <= other.min.x + EPS
            || self.max.y <= other.min.y + EPS
            || self.max.z <= other.min.z + EPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(lo: [f32; 3], hi: [f32; 3]) -> Aabb {
        Aabb {
            min: Vec3::new(lo[0], lo[1], lo[2]),
            max: Vec3::new(hi[0], hi[1], hi[2]),
        }
    }

    #[test]
    fn separated_along_each_axis_is_behind() {
        let a = b([0., 0., 0.], [10., 10., 10.]);
        assert!(a.behind(&b([10., 0., 0.], [20., 10., 10.])), "x");
        assert!(a.behind(&b([0., 10., 0.], [10., 20., 10.])), "y (above = nearer)");
        assert!(a.behind(&b([0., 0., 10.], [10., 10., 20.])), "z");
        assert!(!b([10., 0., 0.], [20., 10., 10.]).behind(&a));
    }

    #[test]
    fn overlapping_boxes_are_not_behind_each_other() {
        let a = b([0., 0., 0.], [10., 10., 10.]);
        let c = b([5., 5., 5.], [15., 15., 15.]);
        assert!(!a.behind(&c) && !c.behind(&a));
    }

    #[test]
    fn box_resting_on_a_slab_is_in_front_of_it() {
        let slab = b([0., 0., 0.], [100., 20., 100.]);
        let on_top = b([10., 20., 10.], [40., 60., 40.]);
        assert!(slab.behind(&on_top));
        assert!(!on_top.behind(&slab));
    }

    #[test]
    fn local_box_follows_the_transform() {
        let mut t = Transform::from_position(Vec3::new(10., 0., 0.));
        t.rotation.y = std::f32::consts::FRAC_PI_2;
        let bb = Aabb::of_local_box(&t, Vec3::new(-1., -1., -5.), Vec3::new(1., 1., 5.));
        // A quarter turn about Y swaps the X and Z extents.
        assert!((bb.max.x - 15.0).abs() < 1e-4 && (bb.min.x - 5.0).abs() < 1e-4);
        assert!((bb.max.z - 1.0).abs() < 1e-4);
    }
}
