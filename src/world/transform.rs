use euclid::default::Vector3D;

/// 3D vector used for positions, directions, rotations (radians) and scales.
pub type Vec3 = Vector3D<f32>;

/// Position / rotation / scale of an object in world space.
///
/// Points are transformed as: scale, then rotate (X, then Y, then Z), then translate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    /// Euler angles in radians, applied in X, Y, Z order.
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: Vec3::new(0.0, 0.0, 0.0),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };

    pub fn from_position(position: impl Into<Vec3>) -> Self {
        Self {
            position: position.into(),
            ..Self::IDENTITY
        }
    }

    /// Rotate a direction (ignores scale and translation). Use for normals.
    pub fn rotate_dir(&self, d: Vec3) -> Vec3 {
        let (sx, cx) = self.rotation.x.sin_cos();
        let (sy, cy) = self.rotation.y.sin_cos();
        let (sz, cz) = self.rotation.z.sin_cos();
        // X
        let (y, z) = (d.y * cx - d.z * sx, d.y * sx + d.z * cx);
        let d = Vec3::new(d.x, y, z);
        // Y
        let (x, z) = (d.x * cy + d.z * sy, -d.x * sy + d.z * cy);
        let d = Vec3::new(x, d.y, z);
        // Z
        let (x, y) = (d.x * cz - d.y * sz, d.x * sz + d.y * cz);
        Vec3::new(x, y, d.z)
    }

    /// Transform a point from local space to world space.
    pub fn apply(&self, p: Vec3) -> Vec3 {
        let scaled = Vec3::new(p.x * self.scale.x, p.y * self.scale.y, p.z * self.scale.z);
        self.rotate_dir(scaled) + self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn identity_is_noop() {
        let p = Vec3::new(1.0, 2.0, 3.0);
        assert!(close(Transform::IDENTITY.apply(p), p));
    }

    #[test]
    fn scale_then_translate() {
        let mut t = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
        t.scale = Vec3::new(2.0, 2.0, 2.0);
        assert!(close(
            t.apply(Vec3::new(1.0, 1.0, 1.0)),
            Vec3::new(12.0, 2.0, 2.0)
        ));
    }

    #[test]
    fn rotate_y_quarter_turn_maps_x_to_minus_z() {
        let mut t = Transform::IDENTITY;
        t.rotation.y = FRAC_PI_2;
        assert!(close(
            t.apply(Vec3::new(1.0, 0.0, 0.0)),
            Vec3::new(0.0, 0.0, -1.0)
        ));
    }

    #[test]
    fn rotate_z_quarter_turn_maps_x_to_y() {
        let mut t = Transform::IDENTITY;
        t.rotation.z = FRAC_PI_2;
        assert!(close(
            t.apply(Vec3::new(1.0, 0.0, 0.0)),
            Vec3::new(0.0, 1.0, 0.0)
        ));
    }
}
