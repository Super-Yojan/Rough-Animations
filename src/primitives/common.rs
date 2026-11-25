//! Helpers shared by the 3D primitives.

use crate::world::transform::Vec3;
use euclid::default::Point2D;
use std::f32::consts::TAU;

/// Directional shading for a surface with unit normal `n`: +Y brightest, +Z base, +X darkest.
/// Axis-aligned normals give exactly 1.15 / 1.0 / 0.78.
pub fn shading(n: Vec3) -> f32 {
    0.78 * n.x * n.x + 1.15 * n.y * n.y + 1.0 * n.z * n.z
}

/// Points on a circle of `radius` in the local XZ plane at height `y`.
pub fn ring(radius: f32, y: f32, segments: usize) -> Vec<Vec3> {
    (0..segments)
        .map(|i| {
            let a = TAU * i as f32 / segments as f32;
            Vec3::new(radius * a.cos(), y, radius * a.sin())
        })
        .collect()
}

/// Convex hull (counter-clockwise) of 2D points, by Andrew's monotone chain.
pub fn convex_hull(points: &[Point2D<f32>]) -> Vec<Point2D<f32>> {
    let mut pts: Vec<Point2D<f32>> = points.to_vec();
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup_by(|a, b| (a.x - b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4);
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: Point2D<f32>, a: Point2D<f32>, b: Point2D<f32>| {
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
    };
    let mut hull: Vec<Point2D<f32>> = Vec::new();
    for pass in 0..2 {
        let start = hull.len();
        let iter: Box<dyn Iterator<Item = &Point2D<f32>>> = if pass == 0 {
            Box::new(pts.iter())
        } else {
            Box::new(pts.iter().rev())
        };
        for &p in iter {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
            {
                hull.pop();
            }
            hull.push(p);
        }
        hull.pop();
    }
    hull
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hull_of_square_with_interior_points() {
        let p = |x, y| Point2D::new(x, y);
        let hull = convex_hull(&[p(0., 0.), p(4., 0.), p(4., 4.), p(0., 4.), p(2., 2.), p(1., 3.)]);
        assert_eq!(hull.len(), 4);
    }

    #[test]
    fn axis_normals_match_the_box_shading() {
        assert_eq!(shading(Vec3::new(0., 1., 0.)), 1.15);
        assert_eq!(shading(Vec3::new(0., 0., 1.)), 1.0);
        assert_eq!(shading(Vec3::new(1., 0., 0.)), 0.78);
    }

    #[test]
    fn visibility_matches_the_three_front_faces() {
        let cam = crate::render::camera::IsoCamera::new(0.0, 0.0);
        assert!(cam.faces_viewer(Vec3::new(0., 1., 0.)));
        assert!(cam.faces_viewer(Vec3::new(1., 0., 0.)));
        assert!(cam.faces_viewer(Vec3::new(0., 0., 1.)));
        assert!(!cam.faces_viewer(Vec3::new(0., -1., 0.)));
        // Turned a half circle, the back faces show instead.
        let mut back = cam;
        back.yaw_rad = std::f32::consts::PI;
        assert!(back.faces_viewer(Vec3::new(-1., 0., 0.)));
        assert!(back.faces_viewer(Vec3::new(0., 0., -1.)));
        assert!(!back.faces_viewer(Vec3::new(1., 0., 0.)));
    }
}
