use std::f32::consts::{PI, TAU};

pub type Point2 = [f32; 2];

/// Number of segments used to approximate curved outlines (circles, ellipses).
pub const CURVE_SEGMENTS: usize = 48;

/// A filled planar area made of closed contours, filled with the even-odd rule: a contour
/// nested inside another one is a hole. Shapes and the results of boolean operations are both
/// `Region`s, so results can be combined again.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Region {
    pub contours: Vec<Vec<Point2>>,
}

impl Region {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_contours(contours: Vec<Vec<Point2>>) -> Self {
        Self { contours }
    }

    pub fn polygon(points: impl IntoIterator<Item = Point2>) -> Self {
        Self {
            contours: vec![points.into_iter().collect()],
        }
    }

    /// Axis-aligned rectangle centred on the origin.
    pub fn rect(width: f32, height: f32) -> Self {
        let (w, h) = (width / 2.0, height / 2.0);
        Self::polygon([[-w, -h], [w, -h], [w, h], [-w, h]])
    }

    pub fn circle(radius: f32) -> Self {
        Self::ellipse(radius, radius)
    }

    pub fn ellipse(rx: f32, ry: f32) -> Self {
        Self::polygon((0..CURVE_SEGMENTS).map(|i| {
            let a = TAU * i as f32 / CURVE_SEGMENTS as f32;
            [rx * a.cos(), ry * a.sin()]
        }))
    }

    /// Regular polygon with a vertex pointing up (+Y).
    pub fn regular_polygon(sides: usize, radius: f32) -> Self {
        let sides = sides.max(3);
        Self::polygon((0..sides).map(|i| {
            let a = PI / 2.0 + TAU * i as f32 / sides as f32;
            [radius * a.cos(), radius * a.sin()]
        }))
    }

    /// Star with `points` tips and a vertex pointing up.
    pub fn star(points: usize, outer: f32, inner: f32) -> Self {
        let points = points.max(3);
        Self::polygon((0..points * 2).map(|i| {
            let r = if i % 2 == 0 { outer } else { inner };
            let a = PI / 2.0 + PI * i as f32 / points as f32;
            [r * a.cos(), r * a.sin()]
        }))
    }

    pub fn is_empty(&self) -> bool {
        self.contours.iter().all(|c| c.len() < 3)
    }

    pub fn map_points(mut self, f: impl Fn(Point2) -> Point2) -> Self {
        for c in &mut self.contours {
            for p in c {
                *p = f(*p);
            }
        }
        self
    }

    pub fn translated(self, dx: f32, dy: f32) -> Self {
        self.map_points(|[x, y]| [x + dx, y + dy])
    }

    /// Rotate counter-clockwise by `radians` around the origin.
    pub fn rotated(self, radians: f32) -> Self {
        let (s, c) = radians.sin_cos();
        self.map_points(|[x, y]| [x * c - y * s, x * s + y * c])
    }

    pub fn scaled(self, sx: f32, sy: f32) -> Self {
        self.map_points(|[x, y]| [x * sx, y * sy])
    }

    /// `(min, max)` corners, or `None` if empty.
    pub fn bounds(&self) -> Option<(Point2, Point2)> {
        let mut pts = self.contours.iter().flatten();
        let first = *pts.next()?;
        Some(pts.fold((first, first), |(lo, hi), p| {
            ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])])
        }))
    }

    /// Whether `p` is inside the region (even-odd).
    pub fn contains(&self, p: Point2) -> bool {
        self.contours.iter().filter(|c| contour_contains(c, p)).count() % 2 == 1
    }

    /// Area of the region (holes subtract).
    pub fn area(&self) -> f32 {
        (0..self.contours.len())
            .map(|i| {
                let a = signed_area(&self.contours[i]).abs();
                if self.contour_depth(i) % 2 == 0 { a } else { -a }
            })
            .sum()
    }

    /// How many other contours enclose contour `i` (0 = outer boundary, 1 = a hole, ...).
    pub fn contour_depth(&self, i: usize) -> usize {
        let Some(&probe) = self.contours[i].first() else {
            return 0;
        };
        self.contours
            .iter()
            .enumerate()
            .filter(|&(j, c)| j != i && contour_contains(c, probe))
            .count()
    }
}

/// Shoelace area: positive for counter-clockwise contours.
pub fn signed_area(c: &[Point2]) -> f32 {
    let n = c.len();
    (0..n)
        .map(|i| {
            let (a, b) = (c[i], c[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f32>()
        / 2.0
}

fn contour_contains(c: &[Point2], p: Point2) -> bool {
    let n = c.len();
    let mut inside = false;
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let (a, b) = (c[i], c[j]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_area_and_bounds() {
        let r = Region::rect(4.0, 2.0);
        assert!((r.area() - 8.0).abs() < 1e-5);
        assert_eq!(r.bounds(), Some(([-2.0, -1.0], [2.0, 1.0])));
    }

    #[test]
    fn circle_area_is_close_to_pi_r_squared() {
        let a = Region::circle(10.0).area();
        assert!((a - PI * 100.0).abs() / (PI * 100.0) < 0.01, "{a}");
    }

    #[test]
    fn transforms() {
        let r = Region::rect(2.0, 2.0).translated(5.0, 0.0).scaled(2.0, 1.0);
        assert_eq!(r.bounds(), Some(([8.0, -1.0], [12.0, 1.0])));
        let q = Region::polygon([[1.0, 0.0], [2.0, 0.0], [2.0, 1.0]]).rotated(PI / 2.0);
        let [x, y] = q.contours[0][0];
        assert!(x.abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn contains_is_even_odd() {
        let ring = Region::from_contours(vec![
            Region::rect(10.0, 10.0).contours.remove(0),
            Region::rect(4.0, 4.0).contours.remove(0),
        ]);
        assert!(ring.contains([4.0, 0.0]));
        assert!(!ring.contains([0.0, 0.0]));
        assert!(!ring.contains([8.0, 0.0]));
        assert!((ring.area() - (100.0 - 16.0)).abs() < 1e-4);
    }

    #[test]
    fn regular_polygon_points_up() {
        let r = Region::regular_polygon(4, 1.0);
        let [x, y] = r.contours[0][0];
        assert!(x.abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
    }
}
