use euclid::default::Point2D;

/// Isometric camera / projection state.
#[derive(Clone, Copy, Debug)]
pub struct IsoCamera {
    pub origin_x: f32,
    pub origin_y: f32,
    pub angle_rad: f32,
    /// Pixels per world unit, relative to the default (1.0). Zooming in makes everything
    /// bigger on screen; line widths and text sizes stay the same.
    pub scale: f32,
}

impl IsoCamera {
    pub fn new(origin_x: f32, origin_y: f32) -> Self {
        Self {
            origin_x,
            origin_y,
            angle_rad: 30.0_f32.to_radians(),
            scale: 1.0,
        }
    }

    /// Project a world-space point (x right, y up, z toward viewer) to screen space.
    pub fn project(&self, x: f32, y: f32, z: f32) -> Point2D<f32> {
        Point2D::new(
            self.origin_x + (x - z) * self.angle_rad.cos() * self.scale,
            self.origin_y + ((x + z) * self.angle_rad.sin() - y) * self.scale,
        )
    }

    /// The point on the floor (`y = 0`) that lands on screen pixel `(sx, sy)`: the inverse of
    /// [`project`](Self::project) for the floor plane.
    pub fn unproject_floor(&self, sx: f32, sy: f32) -> (f32, f32, f32) {
        let a = (sx - self.origin_x) / (self.angle_rad.cos() * self.scale); // x - z
        let b = (sy - self.origin_y) / (self.angle_rad.sin() * self.scale); // x + z
        ((a + b) / 2.0, 0.0, (b - a) / 2.0)
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
    fn height_moves_up_the_screen() {
        let cam = IsoCamera::new(0.0, 0.0);
        assert!(cam.project(0.0, 10.0, 0.0).y < cam.project(0.0, 0.0, 0.0).y);
    }
}
