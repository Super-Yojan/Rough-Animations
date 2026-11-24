use euclid::default::Point2D;

/// Isometric camera / projection state.
#[derive(Clone, Copy, Debug)]
pub struct IsoCamera {
    pub origin_x: f32,
    pub origin_y: f32,
    pub angle_rad: f32,
}

impl IsoCamera {
    pub fn new(origin_x: f32, origin_y: f32) -> Self {
        Self {
            origin_x,
            origin_y,
            angle_rad: 30.0_f32.to_radians(),
        }
    }

    /// Project a world-space point (x right, y up, z toward viewer) to screen space.
    pub fn project(&self, x: f32, y: f32, z: f32) -> Point2D<f32> {
        Point2D::new(
            self.origin_x + (x - z) * self.angle_rad.cos(),
            self.origin_y + (x + z) * self.angle_rad.sin() - y,
        )
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
    fn height_moves_up_the_screen() {
        let cam = IsoCamera::new(0.0, 0.0);
        assert!(cam.project(0.0, 10.0, 0.0).y < cam.project(0.0, 0.0, 0.0).y);
    }
}
