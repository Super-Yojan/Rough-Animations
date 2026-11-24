//! Morphing one [`Region`] into another.
//!
//! Contours are paired up (largest first, nearest centre), each pair is given the same number
//! of points by subdividing its longer edges (original corners are kept, so frame 0 looks
//! exactly like the start shape), the points are rotated into the best alignment, and then
//! interpolated. A contour with no partner shrinks to, or grows from, its own centre.

use super::region::{Point2, Region, signed_area};

/// A prepared morph; evaluate it at any `t` in `[0, 1]`.
#[derive(Clone, Debug)]
pub struct RegionMorph {
    from: Region,
    to: Region,
    pairs: Vec<(Vec<Point2>, Vec<Point2>)>,
}

impl RegionMorph {
    pub fn new(from: &Region, to: &Region) -> Self {
        let (a, b) = (clean(from), clean(to));
        let mut pairs = Vec::new();

        // Outer boundaries morph into outer boundaries, holes into holes.
        for parity in [0, 1] {
            let ai: Vec<usize> = (0..a.contours.len())
                .filter(|&i| a.contour_depth(i) % 2 == parity)
                .collect();
            let mut bi: Vec<usize> = (0..b.contours.len())
                .filter(|&i| b.contour_depth(i) % 2 == parity)
                .collect();

            let mut order = ai;
            order.sort_by(|&x, &y| {
                signed_area(&a.contours[y]).abs().total_cmp(&signed_area(&a.contours[x]).abs())
            });
            for i in order {
                let ca = centroid(&a.contours[i]);
                let nearest = bi
                    .iter()
                    .enumerate()
                    .min_by(|(_, x), (_, y)| {
                        let dx = dist2(ca, centroid(&b.contours[**x]));
                        let dy = dist2(ca, centroid(&b.contours[**y]));
                        dx.total_cmp(&dy)
                    })
                    .map(|(slot, _)| slot);
                match nearest {
                    Some(slot) => {
                        let j = bi.remove(slot);
                        pairs.push(prepare(&a.contours[i], &b.contours[j]));
                    }
                    None => pairs.push(prepare(&a.contours[i], &[centroid(&a.contours[i])])),
                }
            }
            for j in bi {
                pairs.push(prepare(&[centroid(&b.contours[j])], &b.contours[j]));
            }
        }

        Self {
            from: from.clone(),
            to: to.clone(),
            pairs,
        }
    }

    /// The region `t` of the way through. `t <= 0` is exactly the start shape and `t >= 1`
    /// exactly the end shape.
    pub fn at(&self, t: f32) -> Region {
        if t <= 0.0 {
            return self.from.clone();
        }
        if t >= 1.0 {
            return self.to.clone();
        }
        Region::from_contours(
            self.pairs
                .iter()
                .map(|(a, b)| {
                    a.iter()
                        .zip(b)
                        .map(|(p, q)| [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t])
                        .collect()
                })
                .collect(),
        )
    }
}

impl Region {
    /// Prepare a morph from this region into `target`.
    pub fn morph_to(&self, target: &Region) -> RegionMorph {
        RegionMorph::new(self, target)
    }
}

/// Drop degenerate contours and make every contour counter-clockwise.
fn clean(r: &Region) -> Region {
    Region::from_contours(
        r.contours
            .iter()
            .filter(|c| c.len() >= 3)
            .map(|c| {
                let mut c = c.clone();
                if signed_area(&c) < 0.0 {
                    c.reverse();
                }
                c
            })
            .collect(),
    )
}

fn centroid(c: &[Point2]) -> Point2 {
    let n = c.len() as f32;
    let (x, y) = c.iter().fold((0.0, 0.0), |(x, y), p| (x + p[0], y + p[1]));
    [x / n, y / n]
}

fn dist2(a: Point2, b: Point2) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}

/// Give both contours the same point count and the best rotational alignment.
fn prepare(a: &[Point2], b: &[Point2]) -> (Vec<Point2>, Vec<Point2>) {
    let n = a.len().max(b.len()).max(3);
    let a = subdivide(a, n);
    let b = subdivide(b, n);
    let shift = (0..n)
        .min_by(|&s1, &s2| cost(&a, &b, s1).total_cmp(&cost(&a, &b, s2)))
        .unwrap_or(0);
    let b = (0..n).map(|i| b[(i + shift) % n]).collect();
    (a, b)
}

fn cost(a: &[Point2], b: &[Point2], shift: usize) -> f32 {
    let n = a.len();
    (0..n).map(|i| dist2(a[i], b[(i + shift) % n])).sum()
}

/// Grow a closed contour to exactly `n` points by repeatedly splitting its longest edge,
/// keeping every original vertex. A single point is repeated `n` times.
pub fn subdivide(c: &[Point2], n: usize) -> Vec<Point2> {
    if c.len() == 1 {
        return vec![c[0]; n];
    }
    let mut pts = c.to_vec();
    while pts.len() < n {
        let len = pts.len();
        let (i, _) = (0..len)
            .map(|i| (i, dist2(pts[i], pts[(i + 1) % len])))
            .max_by(|x, y| x.1.total_cmp(&y.1))
            .expect("non-empty contour");
        let (p, q) = (pts[i], pts[(i + 1) % len]);
        pts.insert(i + 1, [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0]);
    }
    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn endpoints_are_exact() {
        let (a, b) = (Region::rect(10., 10.), Region::star(5, 20., 8.));
        let m = a.morph_to(&b);
        assert_eq!(m.at(0.0), a);
        assert_eq!(m.at(1.0), b);
        assert_eq!(m.at(-1.0), a);
        assert_eq!(m.at(2.0), b);
    }

    #[test]
    fn subdivide_keeps_corners_and_outline() {
        let sq = Region::rect(10., 10.).contours.remove(0);
        let sub = subdivide(&sq, 32);
        assert_eq!(sub.len(), 32);
        for corner in &sq {
            assert!(sub.contains(corner));
        }
        assert!(close(signed_area(&sub), signed_area(&sq), 1e-3));
    }

    #[test]
    fn first_morphed_frame_starts_from_the_start_shape() {
        // Just after t=0 the polygon (with extra collinear points) still has the start's area.
        let (a, b) = (Region::rect(10., 10.), Region::circle(30.));
        let m = a.morph_to(&b);
        assert!(close(m.at(0.001).area(), a.area(), 1.0));
    }

    #[test]
    fn circle_to_bigger_circle_grows_smoothly() {
        let m = Region::circle(10.).morph_to(&Region::circle(20.));
        let areas: Vec<f32> = (0..=10).map(|i| m.at(i as f32 / 10.0).area()).collect();
        assert!(areas.windows(2).all(|w| w[0] < w[1]), "{areas:?}");
        // Midpoint radius is 15, so area is about pi * 15^2 (no twisting or collapsing).
        let mid = m.at(0.5).area();
        assert!(close(mid, std::f32::consts::PI * 225.0, 10.0), "{mid}");
    }

    #[test]
    fn alignment_avoids_twisting() {
        // A square rotated by 45 degrees morphing into an upright one should pass through
        // a shape no smaller than the smaller of the two (a twist through the centre collapses).
        let a = Region::rect(10., 10.).rotated(std::f32::consts::FRAC_PI_4);
        let b = Region::rect(10., 10.);
        let mid = a.morph_to(&b).at(0.5).area();
        assert!(mid > 0.8 * 100.0, "{mid}");
    }

    #[test]
    fn extra_contours_shrink_and_grow() {
        let two = &Region::circle(5.) | &Region::circle(5.).translated(30., 0.);
        let one = Region::circle(5.);
        // Two shapes into one: the second blob collapses to a point by the end.
        let m = two.morph_to(&one);
        assert_eq!(m.at(0.5).contours.len(), 2);
        // Growing from one to two: the second contour starts as a point and grows.
        let g = one.morph_to(&two);
        let mid = g.at(0.5);
        assert_eq!(mid.contours.len(), 2);
        assert!(mid.area() > one.area() && mid.area() < two.area());
    }

    #[test]
    fn holes_morph_into_holes() {
        let ring_a = &Region::circle(20.) - &Region::circle(8.);
        let ring_b = &Region::circle(30.) - &Region::circle(18.);
        let mid = ring_a.morph_to(&ring_b).at(0.5);
        assert_eq!(mid.contours.len(), 2);
        assert!(!mid.contains([0., 0.]), "hole should stay open");
        assert!(mid.contains([19., 0.]), "ring body at radius ~19 (between 12 and 24)");
    }

    #[test]
    fn empty_regions_are_handled() {
        let r = Region::circle(10.);
        let vanish = r.morph_to(&Region::empty());
        assert!(vanish.at(0.9).area() < r.area() * 0.1);
        let appear = Region::empty().morph_to(&r);
        assert!(appear.at(0.1).area() < r.area() * 0.1);
        assert!(appear.at(0.5).area() > 0.0);
    }
}
