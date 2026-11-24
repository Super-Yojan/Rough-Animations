//! Turning crisp glyph outlines into something that looks written by hand.
//!
//! Typeset glyphs are perfect: ruler-straight stems, identical letters, a dead-level
//! baseline. Handwriting is not, so each glyph is nudged by a smooth, deterministic
//! "wobble": a gentle ripple along its outline, its own small rotation, size and baseline
//! shift, and a slow wave along the line. Everything is derived from the glyph's position in
//! the run, so the result is the same every frame (nothing flickers) and every time.

use super::GlyphRun;
use crate::planar::{Point2, Region};

/// Longest edge, in ems, left in a handwritten outline. Straight stems and rules are cut
/// into pieces this long so they can bend.
const MAX_SEGMENT: f32 = 0.025;

/// Pen thickness added around every glyph, in ems, at strength 1. Typeset hairlines look
/// fragile as handwriting.
pub const PEN_WIDTH: f32 = 0.03;

/// Make `run` look handwritten. `strength` scales every effect (0 returns it unchanged, 1 is
/// the normal look, values up to about 2 are still readable).
pub fn handwrite(run: &GlyphRun, strength: f32) -> GlyphRun {
    if strength <= 0.0 {
        return run.clone();
    }
    let glyphs = run
        .glyphs
        .iter()
        .enumerate()
        .map(|(i, g)| warp_glyph(g, i, strength))
        .collect();
    GlyphRun::centered(glyphs)
}

fn warp_glyph(glyph: &Region, index: usize, strength: f32) -> Region {
    let Some((lo, hi)) = glyph.bounds() else {
        return glyph.clone();
    };
    let center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let mut rng = Rng::new(index as u64);

    // Each letter is slightly turned, scaled and shifted, as if drawn separately.
    let rot = (rng.unit() - 0.5) * 2.0 * 0.045 * strength; // about 2.6 degrees at most
    let scale = 1.0 + (rng.unit() - 0.5) * 0.08 * strength;
    let shift = [
        (rng.unit() - 0.5) * 0.02 * strength,
        (rng.unit() - 0.5) * 0.05 * strength,
    ];
    // A smooth ripple along the outline, plus a faint fine tremor.
    // The ripple's wavelength follows the glyph's size, so a short letter and a long fraction
    // bar both get one or two gentle waves; a fixed wavelength would make long bars squiggle.
    let extent = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(0.15);
    let (fu, fv) = (
        std::f32::consts::TAU * (0.8 + 0.8 * rng.unit()) / extent,
        std::f32::consts::TAU * (0.8 + 0.8 * rng.unit()) / extent,
    );
    let phase = [rng.phase(), rng.phase(), rng.phase(), rng.phase()];
    let (ripple, tremor) = (0.010 * strength, 0.003 * strength);
    let (sin_r, cos_r) = rot.sin_cos();

    let refined = refine(glyph, MAX_SEGMENT);
    refined.map_points(|[x, y]| {
        // Scale and rotate about the glyph's own centre.
        let (dx, dy) = ((x - center[0]) * scale, (y - center[1]) * scale);
        let (rx, ry) = (dx * cos_r - dy * sin_r, dx * sin_r + dy * cos_r);
        let (u, v) = (center[0] + rx, center[1] + ry);
        // Wobble the outline.
        let wx = ripple * (fu * v + phase[0]).sin() + tremor * (70.0 * v + phase[2]).sin();
        let wy = ripple * (fv * u + phase[1]).sin() + tremor * (70.0 * u + phase[3]).sin();
        // The line as a whole drifts gently up and down.
        let drift = 0.02 * strength * (2.0 * center[0] + 0.7).sin();
        [u + wx + shift[0], v + wy + shift[1] + drift]
    })
}

/// Cut every edge longer than `max_len` into equal pieces.
fn refine(region: &Region, max_len: f32) -> Region {
    Region::from_contours(
        region
            .contours
            .iter()
            .map(|c| {
                let mut out: Vec<Point2> = Vec::with_capacity(c.len());
                for k in 0..c.len() {
                    let (p, q) = (c[k], c[(k + 1) % c.len()]);
                    let len = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
                    let n = (len / max_len).ceil().max(1.0) as usize;
                    for j in 0..n {
                        let t = j as f32 / n as f32;
                        out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
                    }
                }
                out
            })
            .collect(),
    )
}

/// A tiny deterministic generator (splitmix64), so the same glyph position always gets the
/// same wobble.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0x1234_5678_9ABC_DEF1))
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    fn phase(&mut self) -> f32 {
        self.unit() * std::f32::consts::TAU
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{Align, Font};

    fn run(text: &str) -> GlyphRun {
        Font::default_font().layout(text, Align::Center, 1.0, 0.0)
    }

    #[test]
    fn strength_zero_changes_nothing() {
        let r = run("abc");
        assert_eq!(handwrite(&r, 0.0).glyphs, r.glyphs);
    }

    #[test]
    fn it_is_deterministic() {
        let r = run("hello world");
        assert_eq!(handwrite(&r, 1.0).glyphs, handwrite(&r, 1.0).glyphs);
    }

    #[test]
    fn it_keeps_the_glyphs_and_roughly_their_shape() {
        let r = run("Hand");
        let h = handwrite(&r, 1.0);
        assert_eq!(h.glyphs.len(), r.glyphs.len());
        // Still the same size to within a few percent, and nothing blew up.
        let ((w0, h0), (w1, h1)) = (r.size(), h.size());
        assert!((w1 / w0 - 1.0).abs() < 0.15 && (h1 / h0 - 1.0).abs() < 0.25, "{w0}x{h0} -> {w1}x{h1}");
        for g in &h.glyphs {
            assert!(g.contours.iter().flatten().all(|p| p[0].is_finite() && p[1].is_finite()));
        }
        // Areas barely change, so the letters stay as heavy as they were.
        let (a0, a1): (f32, f32) = (
            r.glyphs.iter().map(|g| g.area()).sum(),
            h.glyphs.iter().map(|g| g.area()).sum(),
        );
        assert!((a1 / a0 - 1.0).abs() < 0.15, "{a0} -> {a1}");
    }

    #[test]
    fn it_actually_distorts_and_adds_points_so_straight_edges_can_bend() {
        let r = run("I");
        let h = handwrite(&r, 1.0);
        assert_ne!(r.glyphs[0], h.glyphs[0]);
        let count = |run: &GlyphRun| run.glyphs[0].contours.iter().map(|c| c.len()).sum::<usize>();
        assert!(count(&h) > count(&r), "{} vs {}", count(&h), count(&r));
    }

    #[test]
    fn the_same_letter_is_written_differently_each_time() {
        let r = run("ooo");
        let h = handwrite(&r, 1.0);
        let shape = |g: &Region| -> Vec<[i32; 2]> {
            let (lo, _) = g.bounds().unwrap();
            // Compare relative to each glyph's own corner, so only the shape matters.
            g.contours[0].iter().map(|p| [((p[0] - lo[0]) * 1000.0) as i32, ((p[1] - lo[1]) * 1000.0) as i32]).collect()
        };
        assert_ne!(shape(&h.glyphs[0]), shape(&h.glyphs[1]));
        assert_ne!(shape(&h.glyphs[1]), shape(&h.glyphs[2]));
    }

    #[test]
    fn a_long_bar_bows_gently_instead_of_rippling() {
        // A 3 em long, thin bar. After handwriting its edge should bend a little, with only a
        // few changes of direction in the shape the eye sees, and only faint fine tremor.
        let bar = Region::polygon([[-1.5, -0.02], [1.5, -0.02], [1.5, 0.02], [-1.5, 0.02]]);
        let h = handwrite(&GlyphRun { glyphs: vec![bar] }, 1.0);
        // The outline runs along the bottom edge left to right, then back along the top. The
        // whole letter is tilted a little, so take an edge by position in the outline rather
        // than by height.
        let pts = &h.glyphs[0].contours[0];
        let ys: Vec<f32> = pts[..pts.len() / 2 - 3].iter().map(|p| p[1]).collect();

        // Smooth over about 0.25 em to see the bowing without the fine tremor.
        let k = 5;
        let smooth: Vec<f32> = (k..ys.len() - k)
            .map(|i| ys[i - k..=i + k].iter().sum::<f32>() / (2 * k + 1) as f32)
            .collect();
        // Count turning points, ignoring reversals smaller than a visible amount.
        let min_swing = 0.0015;
        let (mut flips, mut dir, mut extreme) = (0, 0.0f32, smooth[0]);
        for &y in &smooth {
            if dir >= 0.0 && y > extreme {
                extreme = y;
                dir = 1.0;
            } else if dir <= 0.0 && y < extreme {
                extreme = y;
                dir = -1.0;
            } else if (dir > 0.0 && extreme - y > min_swing) || (dir < 0.0 && y - extreme > min_swing) {
                flips += 1;
                dir = -dir;
                extreme = y;
            }
        }
        assert!(flips <= 6, "the bar's overall shape changes direction {flips} times: a squiggle");

        // The fine tremor stays well under a pen-width.
        let tremor = ys[k..ys.len() - k]
            .iter()
            .zip(&smooth)
            .map(|(y, s)| (y - s).abs())
            .fold(0.0, f32::max);
        assert!(tremor < 0.008, "tremor {tremor} em");

        // And it does bend: it is no longer a ruler-straight rectangle.
        let spread = smooth.iter().cloned().fold(f32::MIN, f32::max)
            - smooth.iter().cloned().fold(f32::MAX, f32::min);
        assert!(spread > 0.004, "{spread}");
    }

    #[test]
    fn more_strength_means_more_distortion() {
        let r = run("wavy");
        let moved = |s: f32| {
            let h = handwrite(&r, s);
            h.glyphs.iter().zip(&r.glyphs).map(|(a, b)| {
                let (la, _) = a.bounds().unwrap();
                let (lb, _) = b.bounds().unwrap();
                (la[0] - lb[0]).abs() + (la[1] - lb[1]).abs()
            }).sum::<f32>()
        };
        assert!(moved(2.0) > moved(0.5));
    }
}
