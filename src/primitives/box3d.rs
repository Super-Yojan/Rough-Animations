use crate::world::bounds::Aabb;
use crate::primitives::{DrawCtx, Geometry};
use crate::world::style::{Style, shade};
use crate::world::transform::{Transform, Vec3};

/// Axis-aligned (in local space) box, centred on the object's origin.
///
/// `size` is the extent along local X, Y (up) and Z. Use [`Box3D::corner_to_center`]
/// if you think in terms of a minimum corner.
#[derive(Clone, Copy, Debug)]
pub struct Box3D {
    pub size: Vec3,
}

impl Box3D {
    pub fn new(size_x: f32, size_y: f32, size_z: f32) -> Self {
        Self {
            size: Vec3::new(size_x, size_y, size_z),
        }
    }

    /// Centre position for a box of `size` whose minimum corner is `corner`.
    pub fn corner_to_center(corner: impl Into<Vec3>, size: impl Into<Vec3>) -> Vec3 {
        corner.into() + size.into() * 0.5
    }
}

/// (local outward normal, four corner signs in local half-extent units)
const FACES: [([f32; 3], [[f32; 3]; 4]); 6] = [
    (
        [0.0, 1.0, 0.0],
        [[-1., 1., -1.], [1., 1., -1.], [1., 1., 1.], [-1., 1., 1.]],
    ),
    (
        [0.0, -1.0, 0.0],
        [
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., -1., 1.],
            [-1., -1., 1.],
        ],
    ),
    (
        [0.0, 0.0, 1.0],
        [[-1., -1., 1.], [-1., 1., 1.], [1., 1., 1.], [1., -1., 1.]],
    ),
    (
        [0.0, 0.0, -1.0],
        [
            [-1., -1., -1.],
            [-1., 1., -1.],
            [1., 1., -1.],
            [1., -1., -1.],
        ],
    ),
    (
        [1.0, 0.0, 0.0],
        [[1., -1., 1.], [1., 1., 1.], [1., 1., -1.], [1., -1., -1.]],
    ),
    (
        [-1.0, 0.0, 0.0],
        [
            [-1., -1., 1.],
            [-1., 1., 1.],
            [-1., 1., -1.],
            [-1., -1., -1.],
        ],
    ),
];

impl Geometry for Box3D {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        let h = self.size * 0.5;
        Some(Aabb::of_local_box(t, -h, h))
    }

    fn draw(&self, ctx: &mut DrawCtx, transform: &Transform, style: &Style) {
        let half = self.size * 0.5;
        for (normal, corners) in FACES {
            let n = transform.rotate_dir(Vec3::new(normal[0], normal[1], normal[2]));
            // The isometric camera looks along (1, 1, 1) from the viewer toward the scene:
            // a face is visible when its normal points toward the viewer. Back faces are culled.
            if n.x + n.y + n.z <= 1e-6 {
                continue;
            }
            let pts: Vec<_> = corners
                .iter()
                .map(|c| {
                    ctx.project(transform.apply(Vec3::new(
                        c[0] * half.x,
                        c[1] * half.y,
                        c[2] * half.z,
                    )))
                })
                .collect();

            // Directional shading: +Y brightest, +Z base, +X darkest (matches the original look
            // for axis-aligned boxes), blended smoothly as the box rotates.
            let factor = 0.78 * n.x * n.x + 1.15 * n.y * n.y + 1.0 * n.z * n.z;
            let fill = style.fill.map(|c| shade(c, factor));
            ctx.rough_polygon(&pts, fill, style);
        }
    }
}
