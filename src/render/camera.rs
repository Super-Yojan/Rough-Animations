use euclid::default::Point2D;

/// Isometric camera / projection state.
#[derive(Clone, Copy, Debug)]
pub struct IsoCamera {
    pub origin_x: f32,
    pub origin_y: f32,
    pub angle_rad: f32,
    /// Turn of the whole scene about the vertical axis, in radians. 0 is the classic view;
    /// positive turns the scene counter-clockwise as seen from above.
    pub yaw_rad: f32,
    /// Pixels per world unit, relative to the default (1.0). Zooming in makes everything
    /// bigger on screen; line widths and text sizes stay the same.
    pub scale: f32,
    /// Device pixels per layout pixel (1.0 normally, 2.0 for a retina canvas). Unlike
    /// `scale` it also grows line widths, hatch spacing, text and arrowheads, so a figure
    /// drawn at 2.0 is the same picture as at 1.0, just sharper.
    pub pixel_ratio: f32,
}

impl IsoCamera {
    pub fn new(origin_x: f32, origin_y: f32) -> Self {
        Self {
            origin_x,
            origin_y,
            angle_rad: 30.0_f32.to_radians(),
            yaw_rad: 0.0,
            scale: 1.0,
            pixel_ratio: 1.0,
        }
    }

    /// Project a world-space point (x right, y up, z toward viewer) to screen space.
    pub fn project(&self, x: f32, y: f32, z: f32) -> Point2D<f32> {
        let (s, c) = self.yaw_rad.sin_cos();
        let (x, z) = (x * c + z * s, -x * s + z * c);
        Point2D::new(
            self.origin_x + (x - z) * self.angle_rad.cos() * self.scale,
            self.origin_y + ((x + z) * self.angle_rad.sin() - y) * self.scale,
        )
    }

    /// A world-space vector pointing from the scene toward the viewer. At yaw 0 it is
    /// `(1, 1, 1)`. Two points that land on the same pixel differ by a multiple of it.
    pub fn toward_viewer(&self) -> [f32; 3] {
        let (s, c) = self.yaw_rad.sin_cos();
        [c - s, 1.0, s + c]
    }

    /// Whether a surface with outward normal `n` faces the camera.
    pub fn faces_viewer(&self, n: crate::world::transform::Vec3) -> bool {
        let v = self.toward_viewer();
        n.x * v[0] + n.y * v[1] + n.z * v[2] > 1e-6
    }

    /// The point on the floor (`y = 0`) that lands on screen pixel `(sx, sy)`: the inverse of
    /// [`project`](Self::project) for the floor plane.
    pub fn unproject_floor(&self, sx: f32, sy: f32) -> (f32, f32, f32) {
        let a = (sx - self.origin_x) / (self.angle_rad.cos() * self.scale); // x - z
        let b = (sy - self.origin_y) / (self.angle_rad.sin() * self.scale); // x + z
        let (x, z) = ((a + b) / 2.0, (b - a) / 2.0);
        let (s, c) = self.yaw_rad.sin_cos();
        (x * c - z * s, 0.0, x * s + z * c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_projects_to_camera_origin() {
        let cam = IsoCamera::new(400.0, 300.0);
        let p = cam.project(0.0, 0.0, 0.0);
        assert_eq!((p.x, p.y), (400.0, 300.0));
    }

    #[test]
    fn zoom_scales_distances_from_the_origin() {
        let mut cam = IsoCamera::new(100.0, 100.0);
        let p1 = cam.project(10.0, 20.0, 30.0);
        cam.scale = 2.0;
        let p2 = cam.project(10.0, 20.0, 30.0);
        assert!((p2.x - 100.0 - 2.0 * (p1.x - 100.0)).abs() < 1e-4);
        assert!((p2.y - 100.0 - 2.0 * (p1.y - 100.0)).abs() < 1e-4);
        // The origin doesn't move.
        let o = cam.project(0.0, 0.0, 0.0);
        assert_eq!((o.x, o.y), (100.0, 100.0));
    }

    #[test]
    fn unproject_floor_inverts_project() {
        for scale in [1.0, 1.6, 0.5] {
            let mut cam = IsoCamera::new(640.0, 432.0);
            cam.scale = scale;
            for (sx, sy) in [(640.0, 100.0), (200.0, 600.0), (1000.0, 50.0)] {
                let (x, y, z) = cam.unproject_floor(sx, sy);
                assert_eq!(y, 0.0);
                let p = cam.project(x, y, z);
                assert!((p.x - sx).abs() < 1e-3 && (p.y - sy).abs() < 1e-3, "{scale}: {p:?}");
            }
        }
    }

    #[test]
    fn yaw_zero_matches_the_classic_view_and_a_quarter_turn_swaps_axes() {
        let mut cam = IsoCamera::new(0.0, 0.0);
        let a = cam.project(10.0, 5.0, 0.0);
        cam.yaw_rad = std::f32::consts::FRAC_PI_2;
        // A quarter turn carries the +x axis to where -z was: the point (10, 5, 0) now lands
        // where (0, 5, -10) did at yaw 0.
        let mut flat = IsoCamera::new(0.0, 0.0);
        flat.yaw_rad = 0.0;
        let b = flat.project(0.0, 5.0, -10.0);
        let c = cam.project(10.0, 5.0, 0.0);
        assert!((b.x - c.x).abs() < 1e-3 && (b.y - c.y).abs() < 1e-3, "{b:?} {c:?} {a:?}");
    }

    #[test]
    fn unproject_floor_inverts_project_at_any_yaw() {
        let mut cam = IsoCamera::new(300.0, 200.0);
        for yaw in [0.0, 0.7, -1.9, 3.0] {
            cam.yaw_rad = yaw;
            let (x, y, z) = cam.unproject_floor(420.0, 350.0);
            let p = cam.project(x, y, z);
            assert!((p.x - 420.0).abs() < 1e-2 && (p.y - 350.0).abs() < 1e-2, "{yaw}: {p:?}");
        }
    }

    #[test]
    fn the_viewer_vector_is_one_one_one_at_yaw_zero() {
        assert_eq!(IsoCamera::new(0.0, 0.0).toward_viewer(), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn height_moves_up_the_screen() {
        let cam = IsoCamera::new(0.0, 0.0);
        assert!(cam.project(0.0, 10.0, 0.0).y < cam.project(0.0, 0.0, 0.0).y);
    }
}
