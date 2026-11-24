use euclid::default::Vector3D;

/// 3D vector used for positions, directions, rotations (radians) and scales.
pub type Vec3 = Vector3D<f32>;

/// A 3x3 matrix, `m[row][col]`, used for rotations.
pub type Mat3 = [[f32; 3]; 3];

/// The rotation for Euler angles in radians, in this library's convention: turn about X first,
/// then Y, then Z (so the matrix is `Rz * Ry * Rx`, all right-handed).
pub fn rotation_matrix(euler: Vec3) -> Mat3 {
    let (sa, ca) = euler.x.sin_cos();
    let (sb, cb) = euler.y.sin_cos();
    let (sc, cc) = euler.z.sin_cos();
    [
        [cb * cc, sa * sb * cc - ca * sc, ca * sb * cc + sa * sc],
        [cb * sc, sa * sb * sc + ca * cc, ca * sb * sc - sa * cc],
        [-sb, sa * cb, ca * cb],
    ]
}

/// Euler angles (radians, same convention as [`rotation_matrix`]) that produce `m`. Where the
/// answer is not unique (looking straight up or down) the Z angle is taken as 0.
pub fn euler_from_matrix(m: &Mat3) -> Vec3 {
    let sb = (-m[2][0]).clamp(-1.0, 1.0);
    let b = sb.asin();
    if sb.abs() < 0.999_999 {
        Vec3::new(m[2][1].atan2(m[2][2]), b, m[1][0].atan2(m[0][0]))
    } else {
        // Gimbal lock: X and Z turn about the same line, so fold everything into X.
        Vec3::new((-m[1][2]).atan2(m[1][1]), b, 0.0)
    }
}

/// `a * b`: apply `b` first, then `a`.
pub fn mat_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [[0.0; 3]; 3];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    out
}

/// `m * v`.
pub fn mat_apply(m: &Mat3, v: Vec3) -> Vec3 {
    Vec3::new(
        m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
        m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
        m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
    )
}

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

    /// This transform's rotation as a 3x3 matrix, `m[row][col]`.
    pub fn rotation_matrix(&self) -> Mat3 {
        rotation_matrix(self.rotation)
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

    /// Deterministic pseudo-random angles in (-pi, pi).
    fn angles(n: usize) -> Vec<Vec3> {
        let mut seed = 12345u32;
        let mut next = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            ((seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5) * 2.0 * std::f32::consts::PI
        };
        (0..n).map(|_| Vec3::new(next(), next(), next())).collect()
    }

    #[test]
    fn the_matrix_agrees_with_rotate_dir() {
        let probe = [Vec3::new(1., 0., 0.), Vec3::new(0., 1., 0.), Vec3::new(0.3, -0.7, 0.5)];
        for euler in angles(60) {
            let t = Transform {
                rotation: euler,
                ..Transform::IDENTITY
            };
            let m = t.rotation_matrix();
            for v in probe {
                assert!(close(mat_apply(&m, v), t.rotate_dir(v)), "{euler:?} {v:?}");
            }
        }
    }

    #[test]
    fn euler_from_matrix_round_trips_as_a_rotation() {
        // The angles may differ, but they must produce the same rotation.
        for euler in angles(200) {
            let m = rotation_matrix(euler);
            let back = rotation_matrix(euler_from_matrix(&m));
            for r in 0..3 {
                for c in 0..3 {
                    assert!((m[r][c] - back[r][c]).abs() < 1e-4, "{euler:?}: {m:?} vs {back:?}");
                }
            }
        }
    }

    #[test]
    fn euler_from_matrix_survives_looking_straight_up_or_down() {
        use std::f32::consts::FRAC_PI_2;
        for b in [FRAC_PI_2, -FRAC_PI_2] {
            for (a, c) in [(0.3, 0.9), (-1.0, 0.2), (0.0, 0.0)] {
                let m = rotation_matrix(Vec3::new(a, b, c));
                let back = rotation_matrix(euler_from_matrix(&m));
                for r in 0..3 {
                    for k in 0..3 {
                        assert!((m[r][k] - back[r][k]).abs() < 1e-3, "b={b} a={a} c={c}");
                    }
                }
            }
        }
    }

    #[test]
    fn matrix_products_compose_rotations_in_order() {
        use std::f32::consts::FRAC_PI_2;
        // Turning a quarter about X then a quarter about Y: +Z goes to +Y and then to +Y.
        let rx = rotation_matrix(Vec3::new(FRAC_PI_2, 0., 0.));
        let ry = rotation_matrix(Vec3::new(0., FRAC_PI_2, 0.));
        let both = mat_mul(&ry, &rx); // rx first
        assert!(close(mat_apply(&both, Vec3::new(0., 0., 1.)), Vec3::new(0., -1., 0.)));
        assert!(close(
            mat_apply(&both, Vec3::new(1., 0., 0.)),
            mat_apply(&ry, mat_apply(&rx, Vec3::new(1., 0., 0.)))
        ));
        // A product is a rotation too: Euler angles recovered from it reproduce it.
        let again = rotation_matrix(euler_from_matrix(&both));
        assert!(close(mat_apply(&again, Vec3::new(0.2, 0.5, 0.9)), mat_apply(&both, Vec3::new(0.2, 0.5, 0.9))));
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
