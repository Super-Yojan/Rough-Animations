use crate::world::transform::Vec3;

/// Any number type, so `cube(80)`, `cube(80.0)` and `cube(n as f32)` all work.
pub trait Num: Copy {
    fn f32(self) -> f32;
}

macro_rules! impl_num {
    ($($t:ty),*) => { $(impl Num for $t { fn f32(self) -> f32 { self as f32 } })* };
}
impl_num!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// Degrees to radians, for rotations: `rotate_by(id, (0, deg(90), 0))`.
pub fn deg(degrees: impl Num) -> f32 {
    degrees.f32().to_radians()
}

/// A 3D position or vector: `(x, y, z)`, `[x, y, z]` of any number type, or a `Vec3`.
/// World axes: x to the lower right, z to the lower left, y up.
pub trait Pos {
    fn pos(self) -> Vec3;
}

impl<A: Num, B: Num, C: Num> Pos for (A, B, C) {
    fn pos(self) -> Vec3 {
        Vec3::new(self.0.f32(), self.1.f32(), self.2.f32())
    }
}

impl<T: Num> Pos for [T; 3] {
    fn pos(self) -> Vec3 {
        Vec3::new(self[0].f32(), self[1].f32(), self[2].f32())
    }
}

impl Pos for Vec3 {
    fn pos(self) -> Vec3 {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_convert_to_radians() {
        assert!((deg(180) - std::f32::consts::PI).abs() < 1e-6);
        assert!((deg(90.0) - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn positions_from_tuples_arrays_and_vectors() {
        assert_eq!((1, 2, 3).pos(), Vec3::new(1., 2., 3.));
        assert_eq!((1, 2.5, 3u8).pos(), Vec3::new(1., 2.5, 3.));
        assert_eq!([1.5, 2.5, 3.5].pos(), Vec3::new(1.5, 2.5, 3.5));
        assert_eq!([1, 2, 3].pos(), Vec3::new(1., 2., 3.));
        assert_eq!(Vec3::new(4., 5., 6.).pos(), Vec3::new(4., 5., 6.));
    }
}
