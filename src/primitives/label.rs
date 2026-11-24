//! Text and LaTeX geometries: glyph outlines inked like any other rough shape.

use crate::planar::Region;
use crate::primitives::flat::Plane;
use crate::primitives::{DrawCtx, Geometry};
use crate::text::handwriting::{PEN_WIDTH, handwrite};
use crate::text::{Align, DEFAULT_TRACKING, Font, GlyphRun, TexError, TexOptions, tex};
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

/// What plain text needs to be laid out again when its tracking changes.
#[derive(Clone, Debug)]
struct TextSource {
    text: String,
    font: Font,
    align: Align,
    line_spacing: f32,
    tracking: f32,
}

/// Text and formulas share this: a run of glyph outlines, a size and a placement.
#[derive(Clone, Debug)]
struct Label {
    run: GlyphRun,
    /// World units (pixels, for a billboard) per em.
    size: f32,
    placement: Placement,
    /// Lively mode (`.animate()`): hand-drawn wobbling outline and fill, written with a pen
    /// stroke. Off by default, because it is hard to read at small sizes.
    lively: bool,
    /// Present for plain text, so it can be re-laid-out with a new tracking.
    source: Option<TextSource>,
    /// Handwriting strength (`.handwritten()`); 0 is off.
    hand: f32,
    /// The glyphs distorted to look handwritten, when `hand > 0`.
    warped: Option<GlyphRun>,
}

impl Label {
    fn from_text(text: &str, font: &Font, align: Align, line_spacing: f32) -> Self {
        let source = TextSource {
            text: text.to_string(),
            font: font.clone(),
            align,
            line_spacing,
            tracking: DEFAULT_TRACKING,
        };
        Self {
            run: source.font.layout(text, align, line_spacing, DEFAULT_TRACKING),
            size: 24.0,
            placement: Placement::default(),
            lively: false,
            source: Some(source),
            hand: 0.0,
            warped: None,
        }
    }

    /// The glyphs as drawn: handwritten if that is on, otherwise as typeset.
    fn run(&self) -> &GlyphRun {
        self.warped.as_ref().unwrap_or(&self.run)
    }

    fn set_handwritten(&mut self, strength: f32) {
        self.hand = strength.max(0.0);
        self.warped = (self.hand > 0.0).then(|| handwrite(&self.run, self.hand));
    }

    fn set_tracking(&mut self, tracking: f32) {
        if let Some(src) = &mut self.source {
            src.tracking = tracking;
            self.run = src.font.layout(&src.text, src.align, src.line_spacing, tracking);
            // New layout, so the handwriting has to be redone for it.
            self.set_handwritten(self.hand);
        }
    }

    /// Glyph outlines (em units) to screen contours.
    fn project(
        &self,
        ctx: &DrawCtx,
        t: &Transform,
        region: &Region,
    ) -> Vec<Vec<Point2D<f32>>> {
        let mean_scale = (t.scale.x + t.scale.y + t.scale.z) / 3.0;
        let center = ctx.project(t.position);
        region
            .contours
            .iter()
            .map(|c| {
                c.iter()
                    .map(|&[u, v]| match self.placement {
                        Placement::Billboard => {
                            let s = self.size * mean_scale;
                            Point2D::new(center.x + u * s, center.y - v * s)
                        }
                        Placement::On(plane) => {
                            ctx.project(t.apply(plane.to_3d([u * self.size, v * self.size])))
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        let n = self.run().glyphs.len();
        if n == 0 {
            return;
        }
        // Revealing writes the glyphs one after another, each taking `window` of the total
        // time, so the last one finishes exactly when `reveal` reaches 1.
        let window = if n == 1 { 1.0 } else { (2.0 / (n as f32 + 1.0)).min(1.0) };
        let solid = style.fill_style == Some(FillStyle::Solid);
        for (i, glyph) in self.run().glyphs.iter().enumerate() {
            let start = if n == 1 { 0.0 } else { i as f32 / (n - 1) as f32 * (1.0 - window) };
            let progress = ((style.reveal - start) / window).clamp(0.0, 1.0);
            if progress <= 0.0 {
                continue;
            }
            let mut glyph_style = Style {
                reveal: progress,
                ..*style
            };

            if self.lively || !solid {
                // The hand-drawn look: rough fill and outline, traced with a pen. In lively
                // mode a missing outline gets a thin default so the pen has something to trace.
                if self.lively && glyph_style.stroke_width <= 0.0 {
                    glyph_style.stroke_width = 0.7;
                }
                let contours = self.project(ctx, t, glyph);
                ctx.rough_path(&contours, style.fill, &glyph_style);
                continue;
            }

            // Crisp mode: an exact solid fill, with no randomness, so it cannot flicker. It is
            // revealed by a left-to-right wipe instead of a fade.
            if let Some(fill) = style.fill {
                let shown = if progress >= 1.0 { glyph.clone() } else { wipe(glyph, progress) };
                if !shown.is_empty() {
                    let contours = self.project(ctx, t, &shown);
                    let color = style.with_opacity(fill);
                    ctx.fill_exact(&contours, color);
                    if self.hand > 0.0 {
                        // Pen weight: thicken the letters evenly, like a marker would.
                        let mean_scale = (t.scale.x + t.scale.y + t.scale.z) / 3.0;
                        let width = PEN_WIDTH * self.hand * self.size * mean_scale;
                        ctx.stroke_exact(&contours, color, width.max(0.5));
                    }
                }
            }
            // An optional outline (set `stroke_width`) is still hand-drawn, but steady.
            if style.stroke_width > 0.0 {
                let contours = self.project(ctx, t, glyph);
                ctx.rough_path(&contours, None, &glyph_style);
            }
        }
    }

    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        match self.placement {
            // Billboards are drawn in the front layer, so only their anchor matters.
            Placement::Billboard => None,
            Placement::On(plane) => {
                let (w, h) = self.run().size();
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

/// Solid dark ink, no outline: crisp and readable at any size.
fn ink_style() -> Style {
    let ink = rgb(35, 35, 40);
    Style {
        fill: Some(ink),
        stroke: ink,
        stroke_width: 0.0,
        fill_style: Some(FillStyle::Solid),
        ..Style::default()
    }
}

/// The part of `glyph` left of a vertical cut `progress` of the way across it.
fn wipe(glyph: &Region, progress: f32) -> Region {
    let Some((lo, hi)) = glyph.bounds() else {
        return Region::empty();
    };
    let cut = lo[0] + progress * (hi[0] - lo[0]);
    let keep = Region::polygon([
        [lo[0] - 1.0, lo[1] - 1.0],
        [cut, lo[1] - 1.0],
        [cut, hi[1] + 1.0],
        [lo[0] - 1.0, hi[1] + 1.0],
    ]);
    glyph.intersection(&keep)
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
            let (w, h) = self.0.run().size();
            (w * self.0.size, h * self.0.size)
        }

        /// The glyph outlines, one region per letter / symbol / rule, in em units.
        pub fn glyphs(&self) -> &[Region] {
            &self.0.run().glyphs
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
            fn set_tracking(&mut self, tracking: f32) {
                self.0.set_tracking(tracking);
            }
            fn set_lively(&mut self, lively: bool) {
                self.0.lively = lively;
            }
            fn set_handwritten(&mut self, strength: f32) {
                self.0.set_handwritten(strength);
            }
            fn wobbles(&self) -> bool {
                self.0.lively
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
        Self(Label::from_text(text, font, Align::Center, 1.0))
    }

    /// Re-lay out multi-line text with a different alignment and line spacing (1.0 = normal).
    pub fn laid_out(text: &str, font: &Font, align: Align, line_spacing: f32) -> Self {
        Self(Label::from_text(text, font, align, line_spacing))
    }

    /// Extra space after each letter, in ems (default 0.06; 0 is the font's own spacing).
    pub fn tracking(mut self, tracking: f32) -> Self {
        self.0.set_tracking(tracking);
        self
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
            lively: false,
            source: None,
            hand: 0.0,
            warped: None,
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

    // ---- flicker, tracking and .animate() ---------------------------------------------

    use crate::api::shapes::{cube, text};
    use crate::world::object::Object;

    fn pixels_at(scene: &Scene, wobble_step: u64) -> Vec<u8> {
        let mut pm = Pixmap::new(300, 300).unwrap();
        scene.render_wobble(&mut pm.as_mut(), wobble_step);
        pm.data().to_vec()
    }

    fn scene_with(obj: Object) -> Scene {
        let mut s = scene();
        s.add(obj);
        s
    }

    #[test]
    fn default_text_does_not_boil_when_the_wobble_changes() {
        let s = scene_with(text("Steady").font_size(40).at((0, 0, 0)));
        let first = pixels_at(&s, 0);
        for step in 1..6 {
            assert_eq!(first, pixels_at(&s, step), "text changed at wobble step {step}");
        }
    }

    #[test]
    fn animated_text_does_boil() {
        let s = scene_with(text("Lively").font_size(40).animate().at((0, 0, 0)));
        assert_ne!(pixels_at(&s, 0), pixels_at(&s, 1), "lively text should re-draw each step");
    }

    #[test]
    fn default_text_has_no_random_jitter_at_all() {
        // The same text drawn as the first object and as the second (so with a different
        // stroke seed) must be pixel-identical, because nothing about it is random.
        let first = scene_with(text("Exact").font_size(40).at((0, 0, 0)));
        let mut second = scene();
        second.add(cube(1).at((500, 500, 500)).hidden());
        second.add(text("Exact").font_size(40).at((0, 0, 0)));
        assert_eq!(pixels_at(&first, 0), pixels_at(&second, 0));
    }

    #[test]
    fn crisp_text_is_revealed_by_a_wipe_that_never_fades() {
        let region_alpha = |reveal: f32, x: u32| {
            let mut s = scene();
            let id = s.add(text("I").font_size(220).at((0, 0, 0)));
            s.get_mut(id).unwrap().style.reveal = reveal;
            let mut pm = Pixmap::new(300, 300).unwrap();
            s.render(&mut pm.as_mut());
            pm.pixel(x, 150).unwrap().alpha()
        };
        // Find the stem of the "I": the opaque run through the anchor row when fully drawn.
        let stem: Vec<u32> = (0..300).filter(|&x| region_alpha(1.0, x) == 255).collect();
        assert!(stem.len() > 4, "the stem should be solid");
        let near_left = stem[1]; // just inside the left edge of the stem
        // Once the wipe has passed that point it is fully opaque, never partly faded...
        for reveal in [0.3, 0.5, 0.8, 1.0] {
            assert_eq!(region_alpha(reveal, near_left), 255, "faded at reveal {reveal}");
        }
        // ...and before the wipe gets there it is simply not there yet.
        assert_eq!(region_alpha(0.02, *stem.last().unwrap()), 0);
    }

    #[test]
    fn a_partial_write_is_a_clean_cut_of_the_finished_glyph() {
        // Everything drawn at half reveal must also be drawn when finished (a wipe only ever
        // shows part of the glyph; it never adds anything).
        let render_at = |reveal: f32| {
            let mut s = scene();
            let id = s.add(text("W").font_size(200).at((0, 0, 0)));
            s.get_mut(id).unwrap().style.reveal = reveal;
            let mut pm = Pixmap::new(300, 300).unwrap();
            s.render(&mut pm.as_mut());
            pm
        };
        let (half, full) = (render_at(0.5), render_at(1.0));
        let (mut half_ink, mut full_ink) = (0, 0);
        for (h, f) in half.pixels().iter().zip(full.pixels()) {
            if h.alpha() > 200 {
                half_ink += 1;
                assert!(f.alpha() > 200, "the half-written glyph has ink the finished one lacks");
            }
            if f.alpha() > 200 {
                full_ink += 1;
            }
        }
        assert!(half_ink > 0 && half_ink < full_ink, "{half_ink} of {full_ink}");
    }

    /// Width in pixels of the ink an object draws.
    fn ink_width(obj: Object) -> f32 {
        let (x0, _, x1, _) = bbox(&render(&scene_with(obj)));
        (x1 - x0 + 1) as f32
    }

    #[test]
    fn tracking_widens_text_and_is_on_by_default() {
        let at = (0, 0, 0);
        let tight = ink_width(text("abcdef").font_size(30).tracking(0).at(at));
        let default = ink_width(text("abcdef").font_size(30).at(at));
        let loose = ink_width(text("abcdef").font_size(30).tracking(0.3).at(at));
        assert!(default > tight + 5.0, "default tracking should add space: {default} vs {tight}");
        assert!(loose > default);
        // 6 letters = 5 inner gaps of 0.3 em at 30 px.
        assert!((loose - tight - 5.0 * 0.3 * 30.0).abs() < 3.0, "{loose} - {tight}");
        // Harmless on things it doesn't apply to.
        let _ = cube(10).tracking(1);
    }

    #[test]
    fn handwritten_text_looks_different_but_is_just_as_steady() {
        let typeset = scene_with(text("Hand").font_size(70).at((0, 0, 0)));
        let hand = scene_with(text("Hand").font_size(70).handwritten().at((0, 0, 0)));
        assert_ne!(pixels_at(&typeset, 0), pixels_at(&hand, 0), "it should look different");
        // The handwriting is baked in, so it cannot change from frame to frame...
        for step in 1..6 {
            assert_eq!(pixels_at(&hand, 0), pixels_at(&hand, step), "flickers at step {step}");
        }
        // ...nor depend on which object it is.
        let mut later = scene();
        later.add(cube(1).at((500, 500, 500)).hidden());
        later.add(text("Hand").font_size(70).handwritten().at((0, 0, 0)));
        assert_eq!(pixels_at(&hand, 0), pixels_at(&later, 0));
    }

    #[test]
    fn handwritten_text_has_pen_weight() {
        // The same letters, handwritten, put more ink on the page than the typeset ones.
        let ink = |obj: Object| {
            let pm = render(&scene_with(obj));
            pm.pixels().iter().map(|p| p.alpha() as u64).sum::<u64>()
        };
        let plain = ink(text("lllll").font_size(80).at((0, 0, 0)));
        let hand = ink(text("lllll").font_size(80).handwritten().at((0, 0, 0)));
        assert!(hand as f64 > plain as f64 * 1.08, "{hand} vs {plain}");
    }

    #[test]
    fn handwriting_survives_a_change_of_tracking_and_is_harmless_elsewhere() {
        let width = |obj: Object| {
            let (x0, _, x1, _) = bbox(&render(&scene_with(obj)));
            (x1 - x0) as f32
        };
        let tight = width(text("abcd").font_size(40).handwritten().tracking(0).at((0, 0, 0)));
        let loose = width(text("abcd").font_size(40).handwritten().tracking(0.4).at((0, 0, 0)));
        assert!(loose > tight + 30.0, "{loose} vs {tight}");
        // Order doesn't matter.
        let a = width(text("abcd").font_size(40).tracking(0.4).handwritten().at((0, 0, 0)));
        assert_eq!(loose, a);
        // No effect, and no panic, on things that aren't text.
        let _ = cube(10).handwritten();
    }

    #[test]
    fn handwritten_math_works_when_latex_is_installed() {
        if !have_latex() {
            return;
        }
        let typeset = scene_with(crate::api::shapes::math(r"\frac{a}{b}").unwrap().font_size(80).at((0, 0, 0)));
        let hand = scene_with(
            crate::api::shapes::math(r"\frac{a}{b}").unwrap().font_size(80).handwritten().at((0, 0, 0)),
        );
        assert_ne!(pixels_at(&typeset, 0), pixels_at(&hand, 0));
        assert_eq!(pixels_at(&hand, 0), pixels_at(&hand, 4), "steady");
    }

    #[test]
    fn solid_hatched_and_outlines_are_all_available_to_the_user() {
        let base = pixels_at(&scene_with(text("Sketchy").font_size(60).at((0, 0, 0))), 0);
        let hatched = pixels_at(&scene_with(text("Sketchy").font_size(60).hatched().at((0, 0, 0))), 0);
        let outlined = pixels_at(&scene_with(text("Sketchy").font_size(60).stroke_width(1.5).at((0, 0, 0))), 0);
        assert_ne!(base, hatched, ".hatched() should change the fill");
        assert_ne!(base, outlined, "a stroke width should add a hand-drawn outline");
        // An outlined crisp label is still steady under wobble.
        let s = scene_with(text("Sketchy").font_size(60).stroke_width(1.5).at((0, 0, 0)));
        assert_eq!(pixels_at(&s, 0), pixels_at(&s, 3));
        // And .solid() puts hatched text back.
        let back = pixels_at(&scene_with(text("Sketchy").font_size(60).hatched().solid().at((0, 0, 0))), 0);
        assert_eq!(base, back);
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
