//! Vector output: the same drawing calls the raster path makes, written as SVG elements.
//!
//! Partially revealed shapes (mid-`create`) are written whole, so SVG is meant for finished
//! frames: posters, static fallbacks, print.

use euclid::default::Point2D;
use rough_tiny_skia::SkiaDrawable;
use roughr::core::OpSetType;
use std::fmt::Write;
use tiny_skia::{Color, Path, PathSegment};

fn n(v: f32) -> String {
    // Two decimals is plenty for pixels, and keeps files small.
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// `d` attribute for a tiny-skia path.
pub fn path_d(path: &Path) -> String {
    let mut d = String::new();
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => write!(d, "M{} {}", n(p.x), n(p.y)),
            PathSegment::LineTo(p) => write!(d, "L{} {}", n(p.x), n(p.y)),
            PathSegment::QuadTo(c, p) => write!(d, "Q{} {} {} {}", n(c.x), n(c.y), n(p.x), n(p.y)),
            PathSegment::CubicTo(a, b, p) => {
                write!(d, "C{} {} {} {} {} {}", n(a.x), n(a.y), n(b.x), n(b.y), n(p.x), n(p.y))
            }
            PathSegment::Close => write!(d, "Z"),
        }
        .ok();
    }
    d
}

pub fn contours_d(contours: &[Vec<Point2D<f32>>]) -> String {
    let mut d = String::new();
    for c in contours.iter().filter(|c| c.len() >= 3) {
        for (i, p) in c.iter().enumerate() {
            write!(d, "{}{} {}", if i == 0 { "M" } else { "L" }, n(p.x), n(p.y)).ok();
        }
        d.push('Z');
    }
    d
}

/// `#rrggbb` and the alpha, 0 to 1.
fn rgb(c: Color) -> (String, f32) {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    (format!("#{:02x}{:02x}{:02x}", b(c.red()), b(c.green()), b(c.blue())), c.alpha())
}

fn srgba(c: palette::Srgba) -> Color {
    let (r, g, b, a): (u8, u8, u8, u8) = c.into_format().into_components();
    Color::from_rgba8(r, g, b, a)
}

fn opacity(attr: &str, a: f32) -> String {
    if a >= 0.999 { String::new() } else { format!(" {attr}=\"{}\"", n(a)) }
}

pub fn fill(out: &mut String, d: &str, color: Color, even_odd: bool) {
    if d.is_empty() {
        return;
    }
    let (c, a) = rgb(color);
    let rule = if even_odd { " fill-rule=\"evenodd\"" } else { "" };
    writeln!(out, "<path d=\"{d}\" fill=\"{c}\"{}{rule}/>", opacity("fill-opacity", a)).ok();
}

pub fn stroke(out: &mut String, d: &str, color: Color, width: f32) {
    if d.is_empty() {
        return;
    }
    let (c, a) = rgb(color);
    writeln!(
        out,
        "<path d=\"{d}\" fill=\"none\" stroke=\"{c}\"{} stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
        opacity("stroke-opacity", a),
        n(width)
    )
    .ok();
}

/// Write every op set of a rough drawable: outlines, solid fills and hatching.
pub fn drawable(out: &mut String, dr: &SkiaDrawable<f32>) {
    let o = &dr.options;
    let stroke_width = o.stroke_width.unwrap_or(1.0);
    let fill_color = o.fill.map(srgba);
    for set in &dr.sets {
        let d = path_d(&set.ops);
        match set.op_set_type {
            OpSetType::Path => {
                if let Some(c) = o.stroke {
                    stroke(out, &d, srgba(c), stroke_width);
                }
            }
            OpSetType::FillPath => {
                let even_odd = matches!(dr.shape.as_str(), "curve" | "polygon" | "path");
                fill(out, &d, fill_color.unwrap_or(Color::WHITE), even_odd);
            }
            OpSetType::FillSketch => {
                let mut w = o.fill_weight.unwrap_or_default();
                if w < 0.0 {
                    w = stroke_width / 2.0;
                }
                stroke(out, &d, fill_color.unwrap_or(Color::WHITE), w);
            }
        }
    }
}
