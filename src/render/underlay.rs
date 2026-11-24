//! Opaque "paper" fills under shapes, so shapes in front hide the lines behind them.

use euclid::default::Point2D;
use tiny_skia::{Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// Fill `contours` (even-odd, so inner contours stay open) with a flat colour.
pub fn fill_contours(pixmap: &mut PixmapMut, contours: &[Vec<Point2D<f32>>], color: Color) {
    let mut pb = PathBuilder::new();
    for c in contours.iter().filter(|c| c.len() >= 3) {
        pb.move_to(c[0].x, c[0].y);
        for p in &c[1..] {
            pb.line_to(p.x, p.y);
        }
        pb.close();
    }
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    pixmap.fill_path(&path, &paint, FillRule::EvenOdd, Transform::identity(), None);
}

/// Stroke closed contours with a flat colour and round joins.
pub fn stroke_contours(
    pixmap: &mut PixmapMut,
    contours: &[Vec<Point2D<f32>>],
    color: Color,
    width: f32,
) {
    let mut pb = PathBuilder::new();
    for c in contours.iter().filter(|c| c.len() >= 3) {
        pb.move_to(c[0].x, c[0].y);
        for p in &c[1..] {
            pb.line_to(p.x, p.y);
        }
        pb.close();
    }
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
}
