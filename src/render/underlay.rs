//! Opaque "paper" fills under shapes, so shapes in front hide the lines behind them.

use euclid::default::Point2D;
use tiny_skia::{
    BlendMode, Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, PixmapMut, Stroke, Transform,
};

/// Fill `contours` (even-odd, so inner contours stay open) with a flat colour.
pub fn fill_contours(pixmap: &mut PixmapMut, contours: &[Vec<Point2D<f32>>], color: Color) {
    fill_contours_with(pixmap, contours, color, BlendMode::SourceOver);
}

/// Make the inside of `contours` more transparent by `amount` (0 to 1), erasing whatever was
/// drawn there. This is the hidden-line patch for a page with no colour to paint.
pub fn erase_contours(pixmap: &mut PixmapMut, contours: &[Vec<Point2D<f32>>], amount: f32) {
    let mut color = Color::BLACK;
    color.set_alpha(amount.clamp(0.0, 1.0));
    fill_contours_with(pixmap, contours, color, BlendMode::DestinationOut);
}

fn fill_contours_with(
    pixmap: &mut PixmapMut,
    contours: &[Vec<Point2D<f32>>],
    color: Color,
    blend: BlendMode,
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
    paint.blend_mode = blend;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    fn square() -> Vec<Vec<Point2D<f32>>> {
        vec![vec![
            Point2D::new(2.0, 2.0),
            Point2D::new(18.0, 2.0),
            Point2D::new(18.0, 18.0),
            Point2D::new(2.0, 18.0),
        ]]
    }

    fn opaque_pixmap() -> Pixmap {
        let mut pm = Pixmap::new(20, 20).unwrap();
        pm.fill(Color::from_rgba8(10, 20, 30, 255));
        pm
    }

    #[test]
    fn erasing_removes_what_was_there_in_proportion_to_the_amount() {
        for (amount, expected) in [(1.0, 0u8), (0.5, 128), (0.0, 255)] {
            let mut pm = opaque_pixmap();
            erase_contours(&mut pm.as_mut(), &square(), amount);
            let inside = pm.pixel(10, 10).unwrap().alpha();
            assert!((inside as i32 - expected as i32).abs() <= 2, "amount {amount}: alpha {inside}");
            assert_eq!(pm.pixel(0, 0).unwrap().alpha(), 255, "outside the shape nothing changes");
        }
    }

    #[test]
    fn erasing_leaves_holes_alone() {
        // An inner contour (a hole) is not erased: it stays as it was.
        let mut contours = square();
        contours.push(vec![
            Point2D::new(8.0, 8.0),
            Point2D::new(12.0, 8.0),
            Point2D::new(12.0, 12.0),
            Point2D::new(8.0, 12.0),
        ]);
        let mut pm = opaque_pixmap();
        erase_contours(&mut pm.as_mut(), &contours, 1.0);
        assert_eq!(pm.pixel(4, 4).unwrap().alpha(), 0, "the ring is erased");
        assert_eq!(pm.pixel(10, 10).unwrap().alpha(), 255, "the hole is untouched");
    }

    #[test]
    fn painting_still_covers_opaquely() {
        let mut pm = Pixmap::new(20, 20).unwrap();
        fill_contours(&mut pm.as_mut(), &square(), Color::from_rgba8(250, 249, 246, 255));
        let p = pm.pixel(10, 10).unwrap();
        assert_eq!((p.red(), p.alpha()), (250, 255));
    }
}
