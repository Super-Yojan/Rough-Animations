use crate::api::{IntoColor, Num, Pos};
use crate::planar::Region;
use crate::primitives::{Geometry, Plane};
use crate::world::bounds::Aabb;
use crate::world::style::Style;
use crate::world::transform::Transform;
use roughr::core::FillStyle;

/// A thing in the scene: a geometry plus the properties animations will drive.
pub struct Object {
    pub transform: Transform,
    pub style: Style,
    pub visible: bool,
    /// Draw layer; lower is drawn first, before depth is considered.
    pub layer: i32,
    pub(crate) geometry: Box<dyn Geometry>,
}

impl Object {
    pub fn new(geometry: impl Geometry + 'static) -> Self {
        Self {
            transform: Transform::IDENTITY,
            style: geometry.default_style(),
            visible: true,
            layer: geometry.layer(),
            geometry: Box::new(geometry),
        }
    }

    /// Place the object's centre at `position`.
    pub fn at(mut self, position: impl Pos) -> Self {
        self.transform.position = position.pos();
        self
    }

    /// Place the object so its lowest point is at `position`, e.g. to stand a solid on the
    /// floor: `cuboid(80, 110, 80).base_at((50, 20, 50))`. Call after `rotated`/`scaled`.
    pub fn base_at(mut self, position: impl Pos) -> Self {
        let target = position.pos();
        self.transform.position = target;
        let lowest = self.bounds().min.y;
        self.transform.position.y += target.y - lowest;
        self
    }

    /// Euler rotation in radians (X, Y, Z order); use [`deg`](crate::api::num::deg) for degrees.
    pub fn rotated(mut self, rotation: impl Pos) -> Self {
        self.transform.rotation = rotation.pos();
        self
    }

    pub fn scaled(mut self, scale: impl Pos) -> Self {
        self.transform.scale = scale.pos();
        self
    }

    /// Fill colour: a name (`"purple"`), hex (`"#a882ff"`) or `(r, g, b)`.
    pub fn fill(mut self, color: impl IntoColor) -> Self {
        self.style.fill = Some(color.into_color());
        self
    }

    /// Set fill and stroke to the same colour (handy for text and lines).
    pub fn color(mut self, color: impl IntoColor) -> Self {
        let c = color.into_color();
        self.style.fill = Some(c);
        self.style.stroke = c;
        self
    }

    pub fn stroke(mut self, color: impl IntoColor) -> Self {
        self.style.stroke = color.into_color();
        self
    }

    pub fn stroke_width(mut self, width: impl Num) -> Self {
        self.style.stroke_width = width.f32();
        self
    }

    pub fn opacity(mut self, opacity: impl Num) -> Self {
        self.style.opacity = opacity.f32().clamp(0.0, 1.0);
        self
    }

    /// Override the draw layer (lower is drawn first).
    pub fn layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Lay a flat shape or text in a plane of the scene instead of its default.
    ///
    /// # Panics
    /// If the object is not a flat shape or text.
    pub fn on(mut self, plane: Plane) -> Self {
        assert!(
            self.geometry.set_plane(plane),
            "on(): only flat shapes (circle, rect, ...) and text can be placed on a plane"
        );
        self
    }

    /// Text size (one em), in pixels for upright labels. No effect on other objects.
    pub fn font_size(mut self, size: impl Num) -> Self {
        self.geometry.set_font_size(size.f32());
        self
    }

    /// Letter spacing of plain text, in ems (default 0.06; 0 is the font's own spacing).
    /// No effect on other objects or on LaTeX.
    pub fn tracking(mut self, tracking: impl Num) -> Self {
        self.geometry.set_tracking(tracking.f32());
        self
    }

    /// Give text the lively hand-drawn look: a wobbling outline and fill that are written with a
    /// pen stroke, redrawn with the stage's `wobble`. It is lovely on a big title and harder to
    /// read at small sizes, so text is crisp and steady unless you ask. No effect on other
    /// objects.
    pub fn animate(mut self) -> Self {
        self.geometry.set_lively(true);
        self
    }

    /// Make text or a LaTeX formula look handwritten: letters are slightly wobbly, each turned
    /// and sized a little differently, the line drifts, and a pen's thickness is added. It is
    /// steady (nothing flickers) and stays readable. Works on `text(..)`, `math(..)` and
    /// `latex(..)`; no effect on other objects.
    pub fn handwritten(mut self) -> Self {
        self.geometry.set_handwritten(1.0);
        self
    }

    /// Fill with flat colour (the default for text).
    pub fn solid(mut self) -> Self {
        self.style.fill_style = Some(FillStyle::Solid);
        self
    }

    /// Fill with sketchy hatching, as shapes do by default.
    pub fn hatched(mut self) -> Self {
        self.style.fill_style = None;
        self
    }

    /// Slide a flat shape's outline by `(dx, dy)` within its plane. Use it to arrange shapes
    /// before combining them with `union`, `minus`, ...
    ///
    /// # Panics
    /// If the object is not a flat shape.
    pub fn shift(mut self, dx: impl Num, dy: impl Num) -> Self {
        let (dx, dy) = (dx.f32(), dy.f32());
        let region = self
            .geometry
            .region_mut()
            .expect("shift(): only flat shapes can be shifted");
        *region = std::mem::take(region).translated(dx, dy);
        self
    }

    /// Everything covered by either shape. Combine shapes before placing them with `at`; the
    /// other shape's outline is used as it is, its position is ignored.
    pub fn union(self, other: Object) -> Self {
        self.combine(&other, |a, b| a.union(b))
    }

    /// Only what both shapes cover.
    pub fn intersect(self, other: Object) -> Self {
        self.combine(&other, |a, b| a.intersection(b))
    }

    /// This shape with `other` cut out of it.
    pub fn minus(self, other: Object) -> Self {
        self.combine(&other, |a, b| a.difference(b))
    }

    /// What exactly one of the two shapes covers.
    pub fn xor(self, other: Object) -> Self {
        self.combine(&other, |a, b| a.xor(b))
    }

    fn combine(mut self, other: &Object, op: impl Fn(&Region, &Region) -> Region) -> Self {
        let b = other
            .geometry
            .region()
            .expect("boolean operations need flat shapes (circle, rect, text, ...)");
        let a = self
            .geometry
            .region_mut()
            .expect("boolean operations need flat shapes (circle, rect, text, ...)");
        *a = op(a, b);
        self
    }

    /// World-space bounds used for front-to-back ordering (a point at the position if the
    /// geometry doesn't report any).
    pub fn bounds(&self) -> Aabb {
        self.geometry
            .bounds(&self.transform)
            .unwrap_or_else(|| Aabb::point(self.transform.position))
    }

    pub(crate) fn depth(&self) -> f32 {
        self.geometry.depth(&self.transform)
    }
}

impl<G: Geometry + 'static> From<G> for Object {
    fn from(g: G) -> Self {
        Object::new(g)
    }
}
