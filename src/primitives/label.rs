//! Text and LaTeX geometries: glyph outlines inked like any other rough shape.

use crate::planar::Region;
use crate::primitives::flat::Plane;
use crate::primitives::{DrawCtx, Geometry};
use crate::text::{Align, Font, GlyphRun, TexError, TexOptions, tex};
use crate::world::bounds::Aabb;
use crate::world::style::{Style, rgb};
use crate::world::transform::Transform;
use euclid::default::Point2D;
use roughr::core::FillStyle;

/// Where a label lives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// Upright on the screen, anchored at the object's projected position. Always readable.
    #[default]
    Billboard,
    /// Lying in a plane of the scene, so it's skewed like everything else on that plane.
    On(Plane),
}

/// Text and formulas share this: a run of glyph outlines, a size and a placement.
#[derive(Clone, Debug)]
struct Label {
    run: GlyphRun,
    /// World units (pixels, for a billboard) per em.
    size: f32,
    placement: Placement,
}

impl Label {
    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        let n = self.run.glyphs.len();
        if n == 0 {
            return;
        }
        let mean_scale = (t.scale.x + t.scale.y + t.scale.z) / 3.0;
        let center = ctx.project(t.position);

        // Revealing writes the glyphs one after another, each taking `window` of the total
        // time, so the last one finishes exactly when `reveal` reaches 1.
        let window = if n == 1 { 1.0 } else { (2.0 / (n as f32 + 1.0)).min(1.0) };
        for (i, glyph) in self.run.glyphs.iter().enumerate() {
            let start = if n == 1 { 0.0 } else { i as f32 / (n - 1) as f32 * (1.0 - window) };
            let progress = ((style.reveal - start) / window).clamp(0.0, 1.0);
            if progress <= 0.0 {
                continue;
            }
            let contours: Vec<Vec<Point2D<f32>>> = glyph
                .contours
                .iter()
                .map(|c| {
                    c.iter()
                        .map(|&[u, v]| match self.placement {
                            Placement::Billboard => {
                                let s = self.size * mean_scale;
                                Point2D::new(center.x + u * s, center.y - v * s)
                            }
                            Placement::On(plane) => ctx.project(
                                t.apply(plane.to_3d([u * self.size, v * self.size])),
                            ),
                        })
                        .collect()
                })
                .collect();
            let glyph_style = Style {
                reveal: progress,
                ..*style
            };
            ctx.rough_path(&contours, style.fill, &glyph_style);
        }
    }

    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        match self.placement {
            // Billboards are drawn in the front layer, so only their anchor matters.
            Placement::Billboard => None,
            Placement::On(plane) => {
                let (w, h) = self.run.size();
                let (hw, hh) = (w * self.size / 2.0, h * self.size / 2.0);
                Aabb::from_points(
                    [[-hw, -hh], [hw, -hh], [-hw, hh], [hw, hh]]
                        .map(|c| t.apply(plane.to_3d(c))),
                )
            }
        }
    }

    fn layer(&self) -> i32 {
        match self.placement {
            Placement::Billboard => 1,
            Placement::On(_) => 0,
        }
    }
}

/// Solid dark ink with a thin outline, so small letters stay legible.
fn ink_style() -> Style {
    let ink = rgb(35, 35, 40);
    Style {
        fill: Some(ink),
        stroke: ink,
        stroke_width: 0.7,
        fill_style: Some(FillStyle::Solid),
        ..Style::default()
    }
}

macro_rules! label_methods {
    () => {
        /// Size of one em, in world units (pixels for a billboard). Default 24.
        pub fn size(mut self, size: f32) -> Self {
            self.0.size = size;
            self
        }

        /// Draw the label lying in a plane of the scene instead of upright on screen.
        pub fn on(mut self, plane: Plane) -> Self {
            self.0.placement = Placement::On(plane);
            self
        }

        /// `(width, height)` in world units at the current size.
        pub fn dimensions(&self) -> (f32, f32) {
            let (w, h) = self.0.run.size();
            (w * self.0.size, h * self.0.size)
        }

        /// The glyph outlines, one region per letter / symbol / rule, in em units.
        pub fn glyphs(&self) -> &[Region] {
            &self.0.run.glyphs
        }
    };
}

macro_rules! label_geometry {
    ($ty:ty) => {
        impl Geometry for $ty {
            fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
                self.0.draw(ctx, t, style);
            }
            fn default_style(&self) -> Style {
                ink_style()
            }
            fn set_plane(&mut self, plane: Plane) -> bool {
                self.0.placement = Placement::On(plane);
                true
            }
            fn set_font_size(&mut self, size: f32) {
                self.0.size = size;
            }
            fn bounds(&self, t: &Transform) -> Option<Aabb> {
                self.0.bounds(t)
            }
            fn layer(&self) -> i32 {
                self.0.layer()
            }
        }
    };
}

/// Plain text in a font, drawn as hand-inked glyphs. Centred on the object's position;
/// `\n` starts a new line. Reveal it letter by letter with [`create`](crate::animation::create)
/// (or its alias [`write`](crate::animation::write)).
///
/// ```
/// use rough_animations::prelude::*;
/// // The everyday way: `text(..)` builds an Object from this geometry.
/// let label = text("Compute node").font_size(28).at((0, 130, 0)).color("purple");
/// ```
#[derive(Clone, Debug)]
pub struct Text(Label);

impl Text {
    pub fn new(text: &str) -> Self {
        Self::with_font(text, &Font::default_font())
    }

    pub fn with_font(text: &str, font: &Font) -> Self {
        Self(Label {
            run: font.layout(text, Align::Center, 1.0),
            size: 24.0,
            placement: Placement::default(),
        })
    }

    /// Re-lay out multi-line text with a different alignment and line spacing (1.0 = normal).
    pub fn laid_out(text: &str, font: &Font, align: Align, line_spacing: f32) -> Self {
        Self(Label {
            run: font.layout(text, align, line_spacing),
            size: 24.0,
            placement: Placement::default(),
        })
    }

    label_methods!();
}
label_geometry!(Text);

/// A LaTeX formula or text, typeset by your TeX installation and inked by hand.
///
/// Needs `pdflatex` plus `pdftocairo` (poppler) or `mutool` (MuPDF) on `PATH`. Results are
/// cached on disk. `Tex::math` takes math-mode source; `Tex::text` takes any LaTeX.
///
/// ```no_run
/// use rough_animations::prelude::*;
/// // The everyday way: `math(..)` builds an Object from this geometry.
/// let f = math(r"\int_0^1 x^2\,dx = \frac{1}{3}").unwrap().font_size(40);
/// ```
#[derive(Clone, Debug)]
pub struct Tex(Label);

impl Tex {
    /// Math mode (an `align*` body, so `&` and `\\` work for several lines).
    pub fn math(source: &str) -> Result<Self, TexError> {
        Self::math_with(source, &TexOptions::default())
    }

    pub fn math_with(source: &str, opts: &TexOptions) -> Result<Self, TexError> {
        let body = format!("\\begin{{align*}}\n{source}\n\\end{{align*}}");
        Self::from_body(&body, opts)
    }

    /// Any LaTeX in text mode, e.g. `\textbf{Bold} and $x^2$`.
    pub fn text(source: &str) -> Result<Self, TexError> {
        Self::from_body(source, &TexOptions::default())
    }

    pub fn text_with(source: &str, opts: &TexOptions) -> Result<Self, TexError> {
        Self::from_body(source, opts)
    }

    fn from_body(body: &str, opts: &TexOptions) -> Result<Self, TexError> {
        Ok(Self(Label {
            run: tex::typeset(body, opts)?,
            size: 24.0,
            placement: Placement::default(),
        }))
    }

    label_methods!();
}
label_geometry!(Tex);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{create, write};
    use crate::animation::easing::Easing;
    use crate::api::stage::Stage;
    use crate::primitives::Box3D;
    use crate::render::camera::IsoCamera;
    use crate::text::TexOptions;
    use crate::world::scene::Scene;
    use roughr::core::Options;
    use tiny_skia::Pixmap;

    fn scene() -> Scene {
        Scene::new(IsoCamera::new(150.0, 150.0), Options::default())
    }

    fn render(scene: &Scene) -> Pixmap {
        let mut pm = Pixmap::new(300, 300).unwrap();
        scene.render(&mut pm.as_mut());
        pm
    }

    fn inked(pm: &Pixmap) -> usize {
        pm.pixels().iter().filter(|p| p.alpha() > 0).count()
    }

    /// `(min_x, min_y, max_x, max_y)` of drawn pixels.
    fn bbox(pm: &Pixmap) -> (i32, i32, i32, i32) {
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for (i, p) in pm.pixels().iter().enumerate() {
            if p.alpha() > 0 {
                let (x, y) = ((i as u32 % pm.width()) as i32, (i as u32 / pm.width()) as i32);
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
        (x0, y0, x1, y1)
    }

    #[test]
    fn text_draws_and_scales_with_size() {
        let mut small = scene();
        small.add(Text::new("Hello").size(20.0).at([0., 0., 0.]));
        let mut big = scene();
        big.add(Text::new("Hello").size(40.0).at([0., 0., 0.]));
        let (a, b) = (bbox(&render(&small)), bbox(&render(&big)));
        let (wa, wb) = (a.2 - a.0, b.2 - b.0);
        assert!(wa > 20, "small text should be visible, width {wa}");
        assert!((wb as f32 / wa as f32 - 2.0).abs() < 0.2, "{wa} -> {wb}");
    }

    #[test]
    fn billboard_text_is_centred_on_the_projected_position() {
        let mut s = scene();
        s.add(Text::new("Centre").size(30.0).at([60., 40., 10.]));
        let (x0, y0, x1, y1) = bbox(&render(&s));
        let p = IsoCamera::new(150.0, 150.0).project(60., 40., 10.);
        let (cx, cy) = ((x0 + x1) as f32 / 2.0, (y0 + y1) as f32 / 2.0);
        assert!((cx - p.x).abs() < 4.0 && (cy - p.y).abs() < 4.0, "({cx},{cy}) vs {p:?}");
    }

    #[test]
    fn text_is_solid_ink_in_the_chosen_colour() {
        let mut s = scene();
        s.add(Text::new("I").size(200.0).at([0., 0., 0.]).color(rgb(220, 30, 30)));
        let pm = render(&s);
        // The stem of a big "I" passes through the anchor point.
        let px = pm.pixel(150, 150).unwrap();
        assert_eq!(px.alpha(), 255, "solid fill, not hatching");
        assert!(px.red() > 180 && px.green() < 80, "{px:?}");
    }

    #[test]
    fn multi_line_text_is_taller() {
        let one = Text::new("ab").size(30.0);
        let two = Text::new("ab\nab").size(30.0);
        assert!(two.dimensions().1 > 1.5 * one.dimensions().1);
        assert_eq!(two.glyphs().len(), 4);
    }

    #[test]
    fn writing_reveals_letters_progressively_and_ends_identical() {
        let make = |animated: bool| {
            let mut tl = Stage::with(scene(), 300, 300);
            let obj = Text::new("Write me").size(36.0).at([0., 0., 0.]);
            let id = tl.add(if animated { obj.hidden() } else { obj });
            if animated {
                tl.play(write(id).duration(1.0).ease(Easing::Linear));
            } else {
                tl.wait(1.0);
            }
            tl
        };
        let bg = make(true).background.to_color_u8();
        let drawn = |pm: Pixmap| {
            pm.pixels()
                .iter()
                .filter(|p| (p.red(), p.green(), p.blue()) != (bg.red(), bg.green(), bg.blue()))
                .count()
        };
        let mut tl = make(true);
        let counts: Vec<usize> = [0.0, 0.25, 0.5, 0.75, 1.0]
            .iter()
            .map(|&t| drawn(tl.render_frame(t)))
            .collect();
        assert_eq!(counts[0], 0);
        assert!(counts.windows(2).all(|w| w[0] < w[1]), "{counts:?}");
        assert_eq!(
            make(true).render_frame(1.0).data(),
            make(false).render_frame(1.0).data(),
            "a finished write looks exactly like plain text"
        );
    }

    #[test]
    fn write_is_create() {
        let mut s = scene();
        let id = s.add(Text::new("x"));
        let (mut a, mut b) = (write(id), create(id));
        a.begin(&mut s);
        b.begin(&mut s);
        use crate::animation::Animation;
        a.update(&mut s, 0.5);
        let ra = s.get(id).unwrap().style.reveal;
        b.update(&mut s, 0.5);
        assert_eq!(ra, s.get(id).unwrap().style.reveal);
    }

    #[test]
    fn placement_controls_layer_and_look() {
        let mut s = scene();
        let flat = s.add(Text::new("Flat").size(30.0).on(Plane::Ground).at([0., 0., 0.]));
        let label = s.add(Text::new("Label").size(30.0).at([0., 0., 0.]));
        let boxed = s.add(Box3D::new(10., 10., 10.).at([0., 5., 0.]));
        // Text on the floor sorts like any flat shape (here: under the box), and billboard
        // labels are drawn in front of everything.
        assert_eq!(s.draw_order(), vec![flat, boxed, label]);

        let mut a = scene();
        a.add(Text::new("Same").size(30.0).at([0., 0., 0.]));
        let mut b = scene();
        b.add(Text::new("Same").size(30.0).on(Plane::Ground).at([0., 0., 0.]));
        assert_ne!(render(&a).data(), render(&b).data(), "plane text is skewed");
    }

    #[test]
    fn a_caption_on_a_slab_is_visible_not_buried() {
        // The caption lies on the slab's top face (y = 20); the slab's opaque patch must not
        // cover it.
        let mut s = scene();
        s.add(Box3D::new(200., 20., 200.).at([0., 10., 0.]).fill(rgb(225, 225, 232)));
        let with_caption = {
            let mut s2 = scene();
            s2.add(Box3D::new(200., 20., 200.).at([0., 10., 0.]).fill(rgb(225, 225, 232)));
            s2.add(Text::new("platform").size(36.).on(Plane::Ground).at([0., 20., 80.]));
            render(&s2)
        };
        assert_ne!(render(&s).data(), with_caption.data(), "the caption never showed up");
    }

    #[test]
    fn custom_font_data_is_accepted() {
        let font = crate::text::Font::default_font();
        let t = Text::with_font("Hi", &font).size(10.0);
        assert_eq!(t.glyphs().len(), 2);
    }

    fn have_latex() -> bool {
        use std::process::Command;
        let ok = |t: &str, a: &str| Command::new(t).arg(a).output().is_ok();
        ok("pdflatex", "--version") && (ok("pdftocairo", "-v") || ok("mutool", "-v"))
    }

    #[test]
    fn tex_draws_when_latex_is_installed() {
        if !have_latex() {
            return;
        }
        let f = Tex::math(r"\frac{a}{b} + \sqrt{x}").unwrap().size(40.0);
        assert!(f.glyphs().len() >= 5);
        let mut s = scene();
        s.add(f.at([0., 0., 0.]));
        assert!(inked(&render(&s)) > 200);
    }

    #[test]
    fn tex_errors_surface_to_the_caller() {
        let opts = TexOptions {
            latex: "definitely-not-latex-xyz".into(),
            cache: false,
            ..TexOptions::default()
        };
        assert!(Tex::math_with("x", &opts).is_err());
        if have_latex() {
            assert!(Tex::math(r"\notamacro").is_err());
        }
    }
}
