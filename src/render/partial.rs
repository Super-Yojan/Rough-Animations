//! Partially drawn rough shapes: the "pen is still drawing" effect behind `create`.

use rough_tiny_skia::{SkiaDrawable, SkiaGenerator};
use roughr::core::{OpSetType, Options};
use tiny_skia::{Paint, Path, PathBuilder, PathSegment, PixmapMut, Stroke, StrokeDash, Transform};

/// Fraction of the reveal spent drawing outlines; the remainder fades the fill in.
pub const STROKE_PORTION: f32 = 0.75;

/// How far the fill has faded in at `reveal` (0 until the outline is done, then 0 to 1).
pub fn fill_progress(reveal: f32) -> f32 {
    ((reveal - STROKE_PORTION) / (1.0 - STROKE_PORTION)).clamp(0.0, 1.0)
}

/// Draw a rough shape `reveal` (0..1) of the way through its creation: the outline is traced
/// pen-style first, then the fill fades in. `make` builds the full drawable from the
/// (fill-adjusted) options, e.g. `|g| g.polygon(pts)`.
pub fn draw(
    pixmap: &mut PixmapMut,
    mut opts: Options,
    reveal: f32,
    make: impl FnOnce(&SkiaGenerator) -> SkiaDrawable<f32>,
) {
    let stroke_progress = (reveal / STROKE_PORTION).clamp(0.0, 1.0);
    let fill_progress = fill_progress(reveal);

    match &mut opts.fill {
        Some(c) if fill_progress > 0.0 => c.alpha *= fill_progress,
        fill => *fill = None,
    }

    let drawable = make(&SkiaGenerator::new(opts));

    // Fill first (it sits under the outline), then the partially traced strokes.
    let (strokes, others): (Vec<_>, Vec<_>) = drawable
        .sets
        .iter()
        .cloned()
        .partition(|s| s.op_set_type == OpSetType::Path);
    if !others.is_empty() {
        SkiaDrawable {
            shape: drawable.shape.clone(),
            options: drawable.options.clone(),
            sets: others,
        }
        .draw(pixmap);
    }

    let mut paint = Paint::default();
    let c = drawable
        .options
        .stroke
        .unwrap_or(palette::Srgba::new(1.0, 1.0, 1.0, 1.0));
    paint.set_color_rgba8(
        (c.red * 255.0).round() as u8,
        (c.green * 255.0).round() as u8,
        (c.blue * 255.0).round() as u8,
        (c.alpha * 255.0).round() as u8,
    );
    paint.anti_alias = true;
    let width = drawable.options.stroke_width.unwrap_or(1.0);

    let contours: Vec<(Path, f32)> = strokes.iter().flat_map(|s| contours(&s.ops)).collect();
    let total: f32 = contours.iter().map(|(_, len)| len).sum();
    let mut remaining = stroke_progress * total;
    for (path, len) in contours {
        if remaining <= 0.0 {
            break;
        }
        let mut stroke = Stroke {
            width,
            ..Stroke::default()
        };
        if remaining < len {
            // A dash as long as what's left (and a gap beyond the end) draws just the start.
            match StrokeDash::new(vec![remaining.max(0.01), len * 2.0 + 1.0], 0.0) {
                Some(d) => stroke.dash = Some(d),
                None => break,
            }
        }
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        remaining -= len;
    }
}

/// Split a path into its contours, each with its (approximate) length.
pub fn contours(path: &Path) -> Vec<(Path, f32)> {
    let mut out = Vec::new();
    let mut builder = PathBuilder::new();
    let mut len = 0.0;
    let mut cur = (0.0f32, 0.0f32);
    let mut start = cur;
    let mut open = false;

    let mut flush = |builder: &mut PathBuilder, len: &mut f32, open: &mut bool| {
        if *open {
            if let Some(p) = std::mem::replace(builder, PathBuilder::new()).finish() {
                out.push((p, *len));
            }
        }
        *len = 0.0;
        *open = false;
    };

    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                flush(&mut builder, &mut len, &mut open);
                builder.move_to(p.x, p.y);
                cur = (p.x, p.y);
                start = cur;
                open = true;
            }
            PathSegment::LineTo(p) => {
                len += dist(cur, (p.x, p.y));
                builder.line_to(p.x, p.y);
                cur = (p.x, p.y);
            }
            PathSegment::QuadTo(c, p) => {
                len += quad_len(cur, (c.x, c.y), (p.x, p.y));
                builder.quad_to(c.x, c.y, p.x, p.y);
                cur = (p.x, p.y);
            }
            PathSegment::CubicTo(c1, c2, p) => {
                len += cubic_len(cur, (c1.x, c1.y), (c2.x, c2.y), (p.x, p.y));
                builder.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y);
                cur = (p.x, p.y);
            }
            PathSegment::Close => {
                len += dist(cur, start);
                builder.close();
                cur = start;
            }
        }
    }
    flush(&mut builder, &mut len, &mut open);
    out
}

type P = (f32, f32);

fn dist(a: P, b: P) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

const STEPS: usize = 16;

fn quad_len(p0: P, p1: P, p2: P) -> f32 {
    flatten_len(|t| {
        let u = 1.0 - t;
        (
            u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
            u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
        )
    })
}

fn cubic_len(p0: P, p1: P, p2: P, p3: P) -> f32 {
    flatten_len(|t| {
        let u = 1.0 - t;
        let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        (
            a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
            a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
        )
    })
}

fn flatten_len(f: impl Fn(f32) -> P) -> f32 {
    let mut prev = f(0.0);
    let mut len = 0.0;
    for i in 1..=STEPS {
        let p = f(i as f32 / STEPS as f32);
        len += dist(prev, p);
        prev = p;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_contours_and_measures_lines() {
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, 0.0);
        pb.line_to(3.0, 4.0); // 5
        pb.move_to(10.0, 10.0);
        pb.line_to(10.0, 12.0); // 2
        let cs = contours(&pb.finish().unwrap());
        assert_eq!(cs.len(), 2);
        assert!((cs[0].1 - 5.0).abs() < 1e-5);
        assert!((cs[1].1 - 2.0).abs() < 1e-5);
    }

    #[test]
    fn curve_length_is_close_to_straight_line_when_degenerate() {
        // A cubic whose control points lie on the line is exactly that line's length.
        let l = cubic_len((0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0));
        assert!((l - 30.0).abs() < 1e-3);
    }
}
