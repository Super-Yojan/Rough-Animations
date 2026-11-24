//! One short function per shape. All return an [`Object`], so colour, position and the rest
//! chain straight on: `cuboid(80, 110, 80).base_at((50, 20, 50)).fill("purple")`.
//!
//! Solids and flat shapes are centred on their position (use `base_at` to stand something on
//! a point). Flat shapes lie on the floor by default; change that with `.on(Plane::Front)`.

use crate::api::{Num, Pos, Result};
use crate::planar::Region;
use crate::primitives::{
    Arrow3D, Box3D, Cone, Cylinder, Line3D, Prism, Shape2D, Sphere, Tex, Text,
};
use crate::world::object::{Home, Object};

/// Box with the given width (x), height (y, up) and depth (z).
pub fn cuboid(width: impl Num, height: impl Num, depth: impl Num) -> Object {
    Object::new(Box3D::new(width.f32(), height.f32(), depth.f32()))
}

/// Cube with the given side length.
pub fn cube(size: impl Num) -> Object {
    let s = size.f32();
    Object::new(Box3D::new(s, s, s))
}

/// Upright cylinder.
pub fn cylinder(radius: impl Num, height: impl Num) -> Object {
    Object::new(Cylinder::new(radius.f32(), height.f32()))
}

/// Upright cone, base down.
pub fn cone(radius: impl Num, height: impl Num) -> Object {
    Object::new(Cone::new(radius.f32(), height.f32()))
}

pub fn sphere(radius: impl Num) -> Object {
    Object::new(Sphere::new(radius.f32()))
}

/// Flat circle.
pub fn circle(radius: impl Num) -> Object {
    Object::new(Shape2D::circle(radius.f32()))
}

pub fn ellipse(rx: impl Num, ry: impl Num) -> Object {
    Object::new(Shape2D::ellipse(rx.f32(), ry.f32()))
}

/// Flat rectangle.
pub fn rect(width: impl Num, height: impl Num) -> Object {
    Object::new(Shape2D::rect(width.f32(), height.f32()))
}

/// Flat regular polygon with `sides` sides (3 = triangle, 6 = hexagon, ...).
pub fn ngon(sides: usize, radius: impl Num) -> Object {
    Object::new(Shape2D::regular_polygon(sides, radius.f32()))
}

/// Flat star with `points` tips.
pub fn star(points: usize, outer_radius: impl Num, inner_radius: impl Num) -> Object {
    Object::new(Shape2D::star(points, outer_radius.f32(), inner_radius.f32()))
}

/// Flat polygon through the given `(x, y)` points.
pub fn polygon<A: Num, B: Num>(points: impl IntoIterator<Item = (A, B)>) -> Object {
    let region = Region::polygon(points.into_iter().map(|(x, y)| [x.f32(), y.f32()]));
    Object::new(Shape2D::new(region))
}

/// Extrude a flat shape upward into a solid, keeping its colour and position. Combine shapes
/// first to get holes and cut-outs: `prism(rect(80, 80).minus(circle(25)), 40)`.
///
/// # Panics
/// If `shape` is not a flat shape.
pub fn prism(shape: Object, height: impl Num) -> Object {
    let region = shape
        .geometry
        .region()
        .expect("prism() needs a flat shape (circle, rect, star, polygon, ...)")
        .clone();
    let mut solid = Object::new(Prism::new(region, height.f32()));
    solid.style = shape.style;
    solid.transform = shape.transform;
    solid
}

/// Straight line between two points.
pub fn line(from: impl Pos, to: impl Pos) -> Object {
    Object::new(Line3D::new(from.pos(), to.pos()))
}

/// Line with an arrowhead at `to`.
pub fn arrow(from: impl Pos, to: impl Pos) -> Object {
    Object::new(Arrow3D::new(from.pos(), to.pos()))
}

/// Text, upright on the screen and centred on its position. `\n` starts a new line. Set the
/// size with `.font_size(n)` and the colour with `.color(..)`.
pub fn text(content: &str) -> Object {
    Object::new(Text::new(content))
}

/// A caption or label: [`text`] with a readable default look (size 32, a soft gray). Change
/// either with the usual builders: `label("note").font_size(40).color("rose")`.
///
/// Added with no position, it goes just under the title spot (top centre of the canvas).
pub fn label(content: &str) -> Object {
    text(content).font_size(32).color("stone").homed(Home::Caption)
}

/// A big title: size 72, deep indigo, and `.animate()` for the lively hand-drawn wobble, which
/// suits large text. `title("Hello").font_size(110)` for a bigger one.
///
/// Added with no position, it goes at the top centre of the canvas: `s.add(title("Hello"))`.
pub fn title(content: &str) -> Object {
    text(content).font_size(72).color("indigo").animate().homed(Home::Title)
}

/// A LaTeX formula in math mode, e.g. `math(r"\int_0^1 x^2\,dx = \frac13")?`. Needs
/// `pdflatex` and `pdftocairo` or `mutool`.
pub fn math(source: &str) -> Result<Object> {
    Ok(Object::new(Tex::math(source)?))
}

/// LaTeX in text mode, e.g. `latex(r"\textbf{Bold} and $x^2$")?`.
pub fn latex(source: &str) -> Result<Object> {
    Ok(Object::new(Tex::text(source)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::IntoColor;
    use crate::primitives::Plane;

    #[test]
    fn solids_accept_ints_and_floats() {
        let b = cuboid(80, 110.0, 80u8).bounds();
        assert_eq!((b.max.x - b.min.x, b.max.y - b.min.y), (80.0, 110.0));
        let c = cube(10).bounds();
        assert_eq!(c.max.y - c.min.y, 10.0);
    }

    #[test]
    fn base_at_stands_a_solid_on_a_point() {
        for o in [cuboid(10, 40, 10), cylinder(5, 40), cone(5, 40), sphere(20)] {
            let b = o.base_at((7, 20, 3)).bounds();
            assert!((b.min.y - 20.0).abs() < 1e-3, "lowest point {}", b.min.y);
        }
        // Also with a scale applied first.
        let b = cuboid(10, 10, 10).scaled((1, 3, 1)).base_at((0, 5, 0)).bounds();
        assert!((b.min.y - 5.0).abs() < 1e-3 && (b.max.y - 35.0).abs() < 1e-3);
    }

    #[test]
    fn booleans_work_on_shapes_directly() {
        let ring = circle(20).minus(circle(8));
        assert_eq!(ring.geometry.region().unwrap().contours.len(), 2);
        let lens = circle(20).shift(-10, 0).intersect(circle(20).shift(10, 0));
        let area = lens.geometry.region().unwrap().area();
        assert!(area > 0.0 && area < circle(20).geometry.region().unwrap().area());
        let both = circle(10).union(circle(10).shift(40, 0));
        assert_eq!(both.geometry.region().unwrap().contours.len(), 2);
        let crescents = circle(20).xor(circle(20).shift(10, 0));
        assert!(!crescents.geometry.region().unwrap().is_empty());
    }

    #[test]
    #[should_panic(expected = "only flat shapes")]
    fn shifting_a_solid_is_an_error() {
        let _ = cube(10).shift(1, 1);
    }

    #[test]
    #[should_panic(expected = "need flat shapes")]
    fn boolean_on_a_solid_is_an_error() {
        let _ = cube(10).union(circle(5));
    }

    #[test]
    fn prism_extrudes_a_shape_and_keeps_its_look() {
        let p = prism(rect(40, 40).minus(circle(10)).fill("mint").at((5, 0, 5)), 30);
        assert_eq!(p.transform.position.x, 5.0);
        assert!(p.style.fill.is_some());
        let b = p.bounds();
        assert!((b.max.y - b.min.y - 30.0).abs() < 1e-3);
    }

    #[test]
    fn on_moves_flat_shapes_and_text_but_not_solids() {
        let wall = rect(10, 10).on(Plane::Front).bounds();
        assert!(wall.max.y - wall.min.y > 9.0, "a wall is tall");
        let floor = rect(10, 10).bounds();
        assert!(floor.max.y - floor.min.y < 1e-3, "the floor is flat");
        let _ = text("hi").on(Plane::Ground).font_size(30);
    }

    #[test]
    #[should_panic(expected = "only flat shapes")]
    fn on_a_solid_is_an_error() {
        let _ = cube(10).on(Plane::Front);
    }

    #[test]
    fn polygon_takes_any_number_type() {
        let p = polygon([(0, 0), (10, 0), (5, 8)]);
        assert!((p.geometry.region().unwrap().area() - 40.0).abs() < 1e-3);
    }

    #[test]
    fn label_and_title_have_readable_defaults() {
        let l = label("note");
        assert_eq!(l.style.fill, "stone".into_color().into());
        assert!(!l.geometry.wobbles(), "labels stay steady");
        let t = title("Big");
        assert_eq!(t.style.fill, "indigo".into_color().into());
        assert!(t.geometry.wobbles(), "titles are lively");
        // A title is bigger than a label, and both can be adjusted.
        assert!(t.dimensions().unwrap().1 > label("Big").dimensions().unwrap().1 * 1.5);
        let custom = label("note").font_size(64).color("rose");
        assert!(custom.dimensions().unwrap().1 > l.dimensions().unwrap().1 * 1.5);
        assert_eq!(custom.style.fill, "rose".into_color().into());
    }

    #[test]
    fn dimensions_exist_for_text_only() {
        let (w, h) = text("hello").font_size(30).dimensions().unwrap();
        assert!(w > 50.0 && h > 10.0);
        assert!(cube(10).dimensions().is_none());
        // Wider with more letters.
        assert!(text("hello hello").font_size(30).dimensions().unwrap().0 > w * 1.8);
    }

    #[test]
    fn lines_and_text_build() {
        let _ = line((0, 0, 0), (10, 0, 0));
        let _ = arrow([0, 0, 0], [0, 10, 0]).stroke("red");
        let t = text("two\nlines").font_size(20).color("rose");
        assert!(t.style.fill.is_some());
    }
}
