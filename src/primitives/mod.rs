//! Drawable geometries.

pub mod box3d;
pub(crate) mod common;
pub mod flat;
pub mod frustum;
pub mod label;
pub mod line;
pub mod prism;
pub mod sphere;

pub use box3d::Box3D;
pub use flat::{Plane, Shape2D};
pub use frustum::{Cone, Cylinder};
pub use label::{Placement, Tex, Text};
pub use line::{Arrow3D, Line3D};
pub use prism::Prism;
pub use sphere::Sphere;

use crate::api::Pos;
use crate::planar::Region;
use crate::render::camera::IsoCamera;
use crate::render::{partial, underlay};
use crate::world::bounds::Aabb;
use crate::world::object::Object;
use crate::world::style::Style;
use crate::world::transform::{Transform, Vec3};
use euclid::default::Point2D;
use rough_tiny_skia::{SkiaDrawable, SkiaGenerator};
use roughr::core::{FillStyle, Options};
use tiny_skia::{Color, PixmapMut};

/// Objects are drawn in two passes over the same `draw` code: first only the opaque paper
/// patches under filled shapes, then only the ink. Doing all patches first keeps one face's
/// patch from erasing the outline of the face next to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pass {
    Underlay,
    Ink,
}

/// Everything a geometry needs to draw itself.
pub struct DrawCtx<'a, 'b> {
    pub pixmap: &'a mut PixmapMut<'b>,
    pub camera: &'a IsoCamera,
    pub base_opts: &'a Options,
    /// Colour of the paper. When set, filled shapes first get an opaque patch of it, so that
    /// shapes in front hide the lines behind them. `None` leaves fills see-through.
    pub paper: Option<Color>,
    /// Which pass is running; see [`Pass`].
    pub pass: Pass,
    /// Seed for the rough strokes of the object being drawn. Each polygon gets `seed + n`, so
    /// strokes are stable between frames instead of shimmering.
    pub seed: u64,
    pub(crate) draw_count: u64,
}

impl DrawCtx<'_, '_> {
    /// Project a world-space point to screen space.
    pub fn project(&self, p: Vec3) -> Point2D<f32> {
        self.camera.project(p.x, p.y, p.z)
    }

    fn draw_rough(
        &mut self,
        style: &Style,
        fill: Option<Color>,
        make: impl FnOnce(&SkiaGenerator) -> SkiaDrawable<f32>,
    ) {
        let mut opts = style.rough_options(self.base_opts, fill);
        opts.seed = Some(self.seed.wrapping_add(self.draw_count));
        self.draw_count += 1;
        let reveal = style.reveal.clamp(0.0, 1.0);
        if reveal >= 1.0 {
            make(&SkiaGenerator::new(opts)).draw(self.pixmap);
        } else if reveal > 0.0 {
            partial::draw(self.pixmap, opts, reveal, make);
        }
    }

    /// Paint the paper colour inside `contours`, hiding what is behind a filled shape. The
    /// patch fades in with the object's opacity and with the fill during `create`, and is
    /// skipped for outline-only shapes and for solid ink, which is already opaque.
    fn underlay(&mut self, contours: &[Vec<Point2D<f32>>], fill: Option<Color>, style: &Style) {
        let (Some(paper), Some(fill)) = (self.paper, fill) else {
            return;
        };
        if style.fill_style == Some(FillStyle::Solid) {
            return;
        }
        let reveal = style.reveal.clamp(0.0, 1.0);
        let fade = style.opacity * fill.alpha() * partial::fill_progress(reveal.min(1.0));
        let mut color = paper;
        color.set_alpha((paper.alpha() * fade).clamp(0.0, 1.0));
        if color.alpha() > 0.0 {
            underlay::fill_contours(self.pixmap, contours, color);
        }
    }

    /// Draw a rough closed polygon with the given style; `fill` overrides `style.fill`
    /// (for shading).
    pub fn rough_polygon(&mut self, pts: &[Point2D<f32>], fill: Option<Color>, style: &Style) {
        match self.pass {
            Pass::Underlay => self.underlay(&[pts.to_vec()], fill, style),
            Pass::Ink => self.draw_rough(style, fill, |g| g.polygon::<f32>(pts)),
        }
    }

    /// Fill contours exactly (no roughness, no randomness) with a flat colour. Used for crisp
    /// text. Does nothing in the underlay pass.
    pub fn fill_exact(&mut self, contours: &[Vec<Point2D<f32>>], color: Color) {
        if self.pass == Pass::Ink {
            underlay::fill_contours(self.pixmap, contours, color);
        }
    }

    /// Stroke contours exactly with a flat colour and round joins, `width` pixels thick. Used
    /// for the pen weight of handwritten text. Does nothing in the underlay pass.
    pub fn stroke_exact(&mut self, contours: &[Vec<Point2D<f32>>], color: Color, width: f32) {
        if self.pass == Pass::Ink {
            underlay::stroke_contours(self.pixmap, contours, color, width);
        }
    }

    /// Draw several closed contours as one even-odd shape, so inner contours become holes.
    pub fn rough_path(
        &mut self,
        contours: &[Vec<Point2D<f32>>],
        fill: Option<Color>,
        style: &Style,
    ) {
        if self.pass == Pass::Underlay {
            return self.underlay(contours, fill, style);
        }
        let mut d = String::new();
        for c in contours.iter().filter(|c| c.len() >= 3) {
            for (i, p) in c.iter().enumerate() {
                d.push_str(if i == 0 { "M " } else { "L " });
                d.push_str(&format!("{} {} ", p.x, p.y));
            }
            d.push_str("Z ");
        }
        if d.is_empty() {
            return;
        }
        self.draw_rough(style, fill, |g| g.path::<f32>(d));
    }

    /// Draw an open sketchy polyline (stroke only).
    pub fn rough_polyline(&mut self, pts: &[Point2D<f32>], style: &Style) {
        if pts.len() >= 2 && self.pass == Pass::Ink {
            self.draw_rough(style, None, |g| g.linear_path::<f32>(pts, false));
        }
    }
}

/// The shape of an object, in local space. Position, rotation, scale and
/// colours live on the [`Object`] that wraps it.
pub trait Geometry {
    /// Draw using the object's world transform and style.
    fn draw(&self, ctx: &mut DrawCtx, transform: &Transform, style: &Style);

    /// World-space bounding box, used to order objects front to back. `None` is treated as a
    /// single point at the object's position.
    fn bounds(&self, _transform: &Transform) -> Option<Aabb> {
        None
    }

    /// Fallback depth (larger = nearer the viewer) for objects whose boxes overlap. The viewer
    /// is on the +x, +y, +z side, so this is `x + y + z`. Default: the object's origin.
    fn depth(&self, transform: &Transform) -> f32 {
        let p = transform.position;
        p.x + p.y + p.z
    }

    /// The planar region this geometry is built from, if it has one (flat shapes and prisms).
    /// Morphing needs it.
    fn region(&self) -> Option<&Region> {
        None
    }

    fn region_mut(&mut self) -> Option<&mut Region> {
        None
    }

    /// The style a new object starts with. Text uses solid ink instead of hachure.
    fn default_style(&self) -> Style {
        Style::default()
    }

    /// Move the geometry into `plane`. Returns `false` if it has no notion of a plane.
    fn set_plane(&mut self, _plane: Plane) -> bool {
        false
    }

    /// The `(width, height)` text takes up at its current size, in world units (pixels for an
    /// upright label). `None` for anything that isn't text.
    fn dimensions(&self) -> Option<(f32, f32)> {
        None
    }

    /// Change the text size. Ignored by anything that isn't text.
    fn set_font_size(&mut self, _size: f32) {}

    /// Change the letter spacing of plain text, in ems. Ignored by anything else.
    fn set_tracking(&mut self, _tracking: f32) {}

    /// Make text (and LaTeX) look handwritten, `strength` times the normal amount (0 = off).
    /// Ignored by anything that isn't text.
    fn set_handwritten(&mut self, _strength: f32) {}

    /// Turn on the lively hand-drawn mode of text. Ignored by anything that isn't text.
    fn set_lively(&mut self, _lively: bool) {}

    /// Whether the scene's wobble ("boiling lines") redraws this geometry's strokes. Text is
    /// steady unless it is made lively.
    fn wobbles(&self) -> bool {
        true
    }

    /// Draw layer: lower layers are drawn first, whatever their position. Almost everything is
    /// layer 0 and is ordered by its 3D bounds; screen-space labels use 1 so they stay on top.
    fn layer(&self) -> i32 {
        0
    }

    /// Wrap this geometry in an [`Object`] placed at `position`.
    fn at(self, position: impl Pos) -> Object
    where
        Self: Sized + 'static,
    {
        Object::new(self).at(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{Animation, create};
    use crate::planar::Region;
    use crate::render::camera::IsoCamera;
    use crate::world::scene::Scene;
    use crate::world::style::rgb;
    use tiny_skia::Pixmap;

    fn render(objects: impl IntoIterator<Item = Object>) -> Pixmap {
        let mut scene = Scene::new(IsoCamera::new(150.0, 150.0), Options::default());
        for o in objects {
            scene.add(o);
        }
        let mut pm = Pixmap::new(300, 300).unwrap();
        scene.render(&mut pm.as_mut());
        pm
    }

    fn render_scene(scene: &Scene) -> Pixmap {
        let mut pm = Pixmap::new(300, 300).unwrap();
        scene.render(&mut pm.as_mut());
        pm
    }

    fn ink(pm: &Pixmap) -> usize {
        pm.pixels().iter().filter(|p| p.alpha() > 0).count()
    }

    /// Bounding box `(min_x, min_y, max_x, max_y)` of drawn pixels.
    fn bbox(pm: &Pixmap) -> (u32, u32, u32, u32) {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
        for (i, p) in pm.pixels().iter().enumerate() {
            if p.alpha() > 0 {
                let (x, y) = (i as u32 % pm.width(), i as u32 / pm.width());
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
        (x0, y0, x1, y1)
    }

    fn solid(g: impl Geometry + 'static) -> Object {
        g.at([0., 0., 0.]).fill(rgb(200, 90, 90))
    }

    #[test]
    fn every_primitive_draws_something() {
        let region = &Region::rect(40., 40.) - &Region::circle(10.);
        let all: Vec<(&str, Object)> = vec![
            ("cylinder", solid(Cylinder::new(20., 40.))),
            ("cone", solid(Cone::new(20., 40.))),
            ("sphere", solid(Sphere::new(20.))),
            ("prism", solid(Prism::new(region.clone(), 20.))),
            ("flat", solid(Shape2D::new(region))),
            ("line", solid(Line3D::new([0., 0., 0.], [40., 0., 0.]))),
            ("arrow", solid(Arrow3D::new([0., 0., 0.], [0., 40., 0.]))),
        ];
        for (name, o) in all {
            assert!(ink(&render([o])) > 50, "{name} drew nothing");
        }
    }

    #[test]
    fn sphere_outline_is_a_circle_of_the_isometric_radius() {
        let (x0, y0, x1, y1) = bbox(&render([Sphere::new(40.).at([0., 0., 0.]).into()]));
        let expect = 2.0 * 40.0 * 1.2247;
        for extent in [(x1 - x0) as f32, (y1 - y0) as f32] {
            assert!((extent - expect).abs() < 6.0, "extent {extent}, expected ~{expect}");
        }
    }

    #[test]
    fn zoom_enlarges_a_sphere_with_the_rest_of_the_scene() {
        let diameter = |zoom: f32| {
            let mut scene = Scene::new(IsoCamera::new(150.0, 150.0), Options::default());
            scene.camera_mut().scale = zoom;
            scene.add(Sphere::new(30.).at([0., 0., 0.]));
            let (x0, _, x1, _) = bbox(&render_scene(&scene));
            (x1 - x0) as f32
        };
        let (a, b) = (diameter(1.0), diameter(2.0));
        assert!((b / a - 2.0).abs() < 0.1, "{a} -> {b}");
    }

    #[test]
    fn cylinder_is_taller_than_it_is_wide_when_tall() {
        let (x0, y0, x1, y1) = bbox(&render([solid(Cylinder::new(10., 100.))]));
        assert!(y1 - y0 > 2 * (x1 - x0));
    }

    #[test]
    fn cone_is_narrower_at_top_than_base() {
        let pm = render([solid(Cone::new(30., 80.))]);
        let (_, y0, _, y1) = bbox(&pm);
        let width_at = |y: u32| {
            let xs: Vec<u32> = (0..pm.width())
                .filter(|&x| pm.pixel(x, y).unwrap().alpha() > 0)
                .collect();
            xs.last().unwrap_or(&0) - xs.first().unwrap_or(&0)
        };
        assert!(width_at(y0 + 10) < width_at(y1 - 15));
    }

    #[test]
    fn hole_in_prism_shows_the_background() {
        // A tall ring seen from above: the hole's centre pixel stays empty at the top cap.
        let ring = &Region::circle(40.) - &Region::circle(25.);
        let flat = render([Shape2D::new(ring).at([0., 0., 0.]).fill(rgb(10, 10, 10)).into()]);
        // Ground-plane origin projects to (150, 150): the hole centre.
        assert_eq!(flat.pixel(150, 150).unwrap().alpha(), 0);
    }

    #[test]
    fn flat_shapes_sort_by_height_like_everything_else() {
        let mut scene = Scene::new(IsoCamera::new(0., 0.), Options::default());
        // A slab, a decal lying on top of it, and a box standing on the decal.
        let boxed = scene.add(Box3D::new(20., 20., 20.).at([0., 30., 0.]));
        let decal = scene.add(Shape2D::circle(15.).at([0., 20., 0.]));
        let slab = scene.add(Box3D::new(200., 20., 200.).at([0., 10., 0.]));
        assert_eq!(scene.draw_order(), vec![slab, decal, boxed]);
    }

    #[test]
    fn an_explicit_layer_still_overrides() {
        let mut scene = Scene::new(IsoCamera::new(0., 0.), Options::default());
        let decal = scene.add(Shape2D::circle(5.).at([0., 0., 0.]));
        let boxed = scene.add(Box3D::new(10., 10., 10.).at([0., 5., 0.]));
        assert_eq!(scene.draw_order(), vec![decal, boxed]);
        scene.get_mut(decal).unwrap().layer = 5;
        assert_eq!(scene.draw_order(), vec![boxed, decal]);
    }

    #[test]
    fn shape_planes_map_axes() {
        assert_eq!(Plane::Ground.to_3d([2., 3.]), Vec3::new(2., 0., -3.));
        assert_eq!(Plane::Front.to_3d([2., 3.]), Vec3::new(2., 3., 0.));
        assert_eq!(Plane::Side.to_3d([2., 3.]), Vec3::new(0., 3., -2.));
    }

    #[test]
    fn boolean_shape_can_be_created_progressively() {
        use crate::api::stage::Stage;
        let region = &Region::circle(30.) - &Region::circle(12.);
        let mut tl = Stage::with(
            Scene::new(IsoCamera::new(100., 100.), Options::default()),
            200,
            200,
        );
        let id = tl.add(Shape2D::new(region).at([0., 0., 0.]).fill(rgb(50, 120, 220)).hidden());
        tl.play(create(id).duration(1.0).ease(crate::animation::easing::Easing::Linear));
        // Stage frames have an opaque background, so count pixels that differ from it.
        let bg = tl.background.to_color_u8();
        let drawn = |pm: Pixmap| {
            pm.pixels()
                .iter()
                .filter(|p| (p.red(), p.green(), p.blue()) != (bg.red(), bg.green(), bg.blue()))
                .count()
        };
        let a = drawn(tl.render_frame(0.3));
        let b = drawn(tl.render_frame(0.6));
        let c = drawn(tl.render_frame(1.0));
        assert!(0 < a && a < b && b < c, "{a} {b} {c}");
    }

    #[test]
    fn prism_walls_face_away_from_the_solid() {
        // A box-shaped prism has exactly the same two visible walls as Box3D, so the
        // silhouettes agree.
        let prism = render([solid(Prism::new(Region::rect(40., 40.), 40.))]);
        let boxed = render([solid(Box3D::new(40., 40., 40.))]);
        let (p, b) = (bbox(&prism), bbox(&boxed));
        for (pv, bv) in [(p.0, b.0), (p.1, b.1), (p.2, b.2), (p.3, b.3)] {
            assert!((pv as i32 - bv as i32).abs() <= 4, "{p:?} vs {b:?}");
        }
    }

    // Keeps `Animation` in scope for `create(..).duration()` above.
    #[allow(dead_code)]
    fn _uses_animation_trait(a: impl Animation) -> f32 {
        a.duration()
    }
}
