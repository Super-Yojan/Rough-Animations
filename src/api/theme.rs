//! [`Theme`]: the page colour and the default colours of lines and text, in one choice.

use crate::api::{IntoColor, Sketch};
use crate::world::style::Role;
use tiny_skia::Color;

/// Names of the built-in themes, for `Stage::theme("dark")` and [`Theme::by_name`].
pub const NAMES: &[&str] = &["light", "dark", "chalkboard", "blueprint", "sepia", "terminal"];

/// How a scene looks by default: the page colour plus the colours that lines, text, titles and
/// captions take when you haven't chosen one yourself. Pick one with
/// `Stage::new(..).theme("dark")` or `.theme(Theme::chalkboard())`; the built-in ones are
/// [`NAMES`]. `light` is the default look.
///
/// Anything you colour explicitly (`.fill("mint")`, `.stroke("red")`, `.color(..)`) is left alone;
/// a theme only changes the defaults. Make your own with [`Theme::custom`] and the builders:
///
/// ```
/// use rough_animations::prelude::*;
///
/// let ocean = Theme::custom("#06283d", "#e0f2ff").title("#7dd3fc");
/// let s = Stage::new(900, 600).theme(ocean);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    /// The page.
    pub background: Color,
    /// Lines, arrows and body text.
    pub ink: Color,
    /// Big titles.
    pub title: Color,
    /// Captions and labels.
    pub caption: Color,
    /// The hand-drawn look that goes with the theme, if it has one. `None` leaves it alone.
    pub sketch: Option<Sketch>,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

fn hex(s: &str) -> Color {
    s.into_color()
}

impl Theme {
    /// The default look: warm white paper, dark ink, indigo titles.
    pub fn light() -> Self {
        Self::build("light", "#faf9f6", "#232328", "#3c3278", "#6a6a80", None)
    }

    /// A near-black page with soft white ink.
    pub fn dark() -> Self {
        Self::build("dark", "#101018", "#e8e8f0", "#c9c3ff", "#9a9ab0", None)
    }

    /// A green classroom board with chalk-white lines, yellow titles and rougher strokes.
    pub fn chalkboard() -> Self {
        Self::build(
            "chalkboard",
            "#1f3a2e",
            "#f1f1e6",
            "#ffe08a",
            "#b8d4c2",
            Some(Sketch::default().roughness(1.8)),
        )
    }

    /// White technical lines on blueprint blue, with straight, tidy strokes.
    pub fn blueprint() -> Self {
        Self::build(
            "blueprint",
            "#14447f",
            "#e6f0ff",
            "#ffffff",
            "#a8c8f2",
            Some(Sketch::clean().hatch_gap(8.0)),
        )
    }

    /// Aged paper with brown ink.
    pub fn sepia() -> Self {
        Self::build("sepia", "#f4ecd8", "#3b2f20", "#7a3e12", "#7d6a4c", None)
    }

    /// Green phosphor on black, with straight lines.
    pub fn terminal() -> Self {
        Self::build(
            "terminal",
            "#0a0f0a",
            "#7cfc9a",
            "#c8ffd6",
            "#4fae6a",
            Some(Sketch::clean()),
        )
    }

    /// A built-in theme by name (see [`NAMES`]), or `None`.
    pub fn by_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "light" => Some(Self::light()),
            "dark" => Some(Self::dark()),
            "chalkboard" => Some(Self::chalkboard()),
            "blueprint" => Some(Self::blueprint()),
            "sepia" => Some(Self::sepia()),
            "terminal" => Some(Self::terminal()),
            _ => None,
        }
    }

    /// Every built-in theme.
    pub fn all() -> Vec<Self> {
        NAMES.iter().map(|n| Self::by_name(n).expect("listed name")).collect()
    }

    /// Your own theme from a page colour and an ink colour. Titles use the ink, and captions
    /// are the ink faded a third of the way into the page. Adjust them with [`title`](Self::title)
    /// and [`caption`](Self::caption).
    pub fn custom(background: impl IntoColor, ink: impl IntoColor) -> Self {
        let (background, ink) = (background.into_color(), ink.into_color());
        Self {
            name: "custom".to_string(),
            background,
            ink,
            title: ink,
            caption: mix(ink, background, 0.35),
            sketch: None,
        }
    }

    fn build(
        name: &str,
        background: &str,
        ink: &str,
        title: &str,
        caption: &str,
        sketch: Option<Sketch>,
    ) -> Self {
        Self {
            name: name.to_string(),
            background: hex(background),
            ink: hex(ink),
            title: hex(title),
            caption: hex(caption),
            sketch,
        }
    }

    pub fn background(mut self, color: impl IntoColor) -> Self {
        self.background = color.into_color();
        self
    }

    pub fn ink(mut self, color: impl IntoColor) -> Self {
        self.ink = color.into_color();
        self
    }

    pub fn title(mut self, color: impl IntoColor) -> Self {
        self.title = color.into_color();
        self
    }

    pub fn caption(mut self, color: impl IntoColor) -> Self {
        self.caption = color.into_color();
        self
    }

    /// The strokes that go with this theme.
    pub fn sketch(mut self, sketch: Sketch) -> Self {
        self.sketch = Some(sketch);
        self
    }

    /// The colour for a role.
    pub(crate) fn color(&self, role: Role) -> Color {
        match role {
            Role::Ink => self.ink,
            Role::Title => self.title,
            Role::Caption => self.caption,
        }
    }
}

/// `a` mixed `amount` (0 to 1) of the way towards `b`.
fn mix(a: Color, b: Color, amount: f32) -> Color {
    let f = |x: f32, y: f32| x + (y - x) * amount;
    Color::from_rgba(f(a.red(), b.red()), f(a.green(), b.green()), f(a.blue(), b.blue()), 1.0)
        .expect("channels stay in range")
}

/// Anything that can be a theme: a [`Theme`], or the name of a built-in one (`"dark"`).
///
/// An unknown name panics with the list of valid ones, like an unknown colour does.
pub trait IntoTheme {
    fn into_theme(self) -> Theme;
}

impl IntoTheme for Theme {
    fn into_theme(self) -> Theme {
        self
    }
}

impl IntoTheme for &str {
    fn into_theme(self) -> Theme {
        Theme::by_name(self).unwrap_or_else(|| {
            panic!("unknown theme {self:?}: use one of {}", NAMES.join(", "))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::style::{Style, ink};

    /// WCAG relative luminance.
    fn luminance(c: Color) -> f32 {
        let lin = |v: f32| if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        0.2126 * lin(c.red()) + 0.7152 * lin(c.green()) + 0.0722 * lin(c.blue())
    }

    /// WCAG contrast ratio, 1 (none) to 21 (black on white).
    fn contrast(a: Color, b: Color) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn there_are_six_built_in_themes_found_by_name() {
        assert_eq!(NAMES.len(), 6);
        for name in NAMES {
            let t = Theme::by_name(name).unwrap_or_else(|| panic!("{name} is listed but missing"));
            assert_eq!(&t.name, name);
            assert_eq!(Theme::by_name(&name.to_uppercase()), Some(t.clone()), "case-insensitive");
        }
        assert_eq!(Theme::all().len(), NAMES.len());
        assert!(Theme::by_name("neon-pink").is_none());
        assert_eq!(Theme::default(), Theme::light());
    }

    #[test]
    fn every_theme_is_readable() {
        // Body ink and titles are text people have to read; captions are secondary but still legible.
        for t in Theme::all() {
            let (ink, title, caption) = (
                contrast(t.ink, t.background),
                contrast(t.title, t.background),
                contrast(t.caption, t.background),
            );
            assert!(ink >= 7.0, "{}: ink contrast {ink:.1}", t.name);
            assert!(title >= 4.5, "{}: title contrast {title:.1}", t.name);
            assert!(caption >= 3.0, "{}: caption contrast {caption:.1}", t.name);
        }
    }

    #[test]
    fn the_themes_are_genuinely_different() {
        let all = Theme::all();
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.background, b.background, "{} and {} share a page colour", a.name, b.name);
            }
        }
        // At least one dark and one light.
        assert!(all.iter().any(|t| luminance(t.background) < 0.05));
        assert!(all.iter().any(|t| luminance(t.background) > 0.5));
    }

    #[test]
    fn light_is_exactly_the_look_that_existed_before_themes() {
        // This is what keeps every existing script pixel-identical.
        let t = Theme::light();
        assert_eq!(t.ink, ink());
        assert_eq!(t.ink, Style::default().stroke);
        assert_eq!(t.background, Color::from_rgba8(250, 249, 246, 255));
        assert_eq!(t.title, "indigo".into_color());
        assert_eq!(t.caption, "stone".into_color());
        assert!(t.sketch.is_none());
    }

    #[test]
    fn custom_themes_and_builders() {
        let t = Theme::custom("#000000", "#ffffff");
        assert_eq!(t.title, t.ink, "titles use the ink by default");
        // The caption sits between ink and page.
        assert!(luminance(t.caption) > luminance(t.background) && luminance(t.caption) < luminance(t.ink));
        let t = t.title("#ff0000").caption("gray").ink("amber").background("slate").sketch(Sketch::clean());
        assert_eq!(t.title, Color::from_rgba8(255, 0, 0, 255));
        assert_eq!(t.caption, "gray".into_color());
        assert_eq!(t.ink, "amber".into_color());
        assert_eq!(t.background, "slate".into_color());
        assert_eq!(t.sketch, Some(Sketch::clean()));
    }

    #[test]
    fn roles_pick_the_right_colour() {
        let t = Theme::dark();
        assert_eq!(t.color(Role::Ink), t.ink);
        assert_eq!(t.color(Role::Title), t.title);
        assert_eq!(t.color(Role::Caption), t.caption);
    }

    // ---- using a theme on a stage -----------------------------------------------------

    use crate::api::Stage;
    use crate::api::shapes::*;
    use crate::primitives::Plane;
    use tiny_skia::Pixmap;

    /// The same little scene for every theme: default-coloured outlines, an arrow, text, a title.
    fn scene(s: &mut Stage) {
        s.add(title("Theme"));
        s.add(label("a caption"));
        s.add(cube(70).base_at((0, 0, 0)).fill("mint"));
        s.add(arrow((0, 0, 0), (0, 110, 0)).at((90, 0, 0)));
        s.add(text("plain text").at(s.pixel(450, 500)));
        s.add(rect(60, 40).on(Plane::Ground).at((-120, 0, 40)));
    }

    fn end_frame(s: &mut Stage) -> Pixmap {
        let end = s.duration();
        s.render_frame(end)
    }

    #[test]
    fn the_light_theme_changes_nothing_pixel_for_pixel() {
        let (mut plain, mut themed) = (Stage::new(900, 600), Stage::new(900, 600).theme(Theme::light()));
        scene(&mut plain);
        scene(&mut themed);
        assert_eq!(end_frame(&mut plain).data(), end_frame(&mut themed).data());
        // Same through the name.
        let mut by_name = Stage::new(900, 600).theme("light");
        scene(&mut by_name);
        assert_eq!(end_frame(&mut plain).data(), end_frame(&mut by_name).data());
    }

    #[test]
    fn a_theme_sets_the_page_and_the_default_colours() {
        let t = Theme::chalkboard();
        let mut s = Stage::new(900, 600).theme(t.clone());
        assert_eq!(s.background, t.background);
        assert_eq!(s.theme, t);
        let (c, tx, ti, la) = (
            s.add(cube(10)),
            s.add(text("t")),
            s.add(title("T")),
            s.add(label("l")),
        );
        let col = |s: &Stage, id| {
            let o = s.scene.get(id).unwrap();
            (o.style.stroke, o.style.fill)
        };
        assert_eq!(col(&s, c).0, t.ink, "outlines use the ink");
        assert_eq!(col(&s, tx), (t.ink, Some(t.ink)), "text uses the ink");
        assert_eq!(col(&s, ti), (t.title, Some(t.title)));
        assert_eq!(col(&s, la), (t.caption, Some(t.caption)));
    }

    #[test]
    fn colours_you_choose_are_left_alone() {
        let mut s = Stage::new(900, 600).theme("dark");
        let red = Color::from_rgba8(255, 0, 0, 255);
        let a = s.add(cube(10).stroke(red).fill("mint"));
        let b = s.add(text("x").color("rose"));
        let c = s.add(text("y").fill("gold"));
        let d = s.add(title("z").color("#123456"));
        let get = |id| s.scene.get(id).unwrap().style;
        assert_eq!(get(a).stroke, red);
        assert_eq!(get(a).fill, Some("mint".into_color()));
        assert_eq!((get(b).fill, get(b).stroke), (Some("rose".into_color()), "rose".into_color()));
        assert_eq!(get(c).fill, Some("gold".into_color()));
        assert_eq!(get(d).fill, Some("#123456".into_color()), "an explicit title colour beats the theme");
    }

    #[test]
    fn the_theme_applies_to_objects_added_before_it_too() {
        // Order must not matter: the theme can come before or after the objects.
        let mut s = Stage::with(
            crate::world::scene::Scene::new(
                crate::render::camera::IsoCamera::new(450.0, 360.0),
                crate::api::Sketch::default().options(),
            ),
            900,
            600,
        );
        let (c, t) = (s.add(cube(10)), s.add(label("late")));
        let s = s.theme("terminal");
        assert_eq!(s.scene.get(c).unwrap().style.stroke, Theme::terminal().ink);
        assert_eq!(s.scene.get(t).unwrap().style.fill, Some(Theme::terminal().caption));
        // And a later theme replaces an earlier one.
        let s = s.theme("sepia");
        assert_eq!(s.scene.get(c).unwrap().style.stroke, Theme::sepia().ink);
        assert_eq!(s.background, Theme::sepia().background);
    }

    #[test]
    fn headings_and_tags_follow_the_theme() {
        let mut s = Stage::new(900, 600).theme("blueprint");
        let ids = s.heading("Title", "caption");
        let cube_id = s.add(cube(40).base_at((0, 0, 0)));
        let tag = s.tag(cube_id, "tag", crate::api::Side::Above);
        let fill = |id| s.scene.get(id).unwrap().style.fill;
        assert_eq!(fill(ids[0]), Some(Theme::blueprint().title));
        assert_eq!(fill(ids[1]), Some(Theme::blueprint().caption));
        assert_eq!(fill(tag), Some(Theme::blueprint().caption));
    }

    #[test]
    fn a_theme_can_bring_its_own_strokes_and_a_later_sketch_wins() {
        let roughness = |s: &Stage| s.scene.options().roughness;
        assert_eq!(roughness(&Stage::new(100, 100)), Some(1.3));
        assert_eq!(roughness(&Stage::new(100, 100).theme("dark")), Some(1.3), "dark leaves the strokes alone");
        assert_eq!(roughness(&Stage::new(100, 100).theme("blueprint")), Some(0.0), "blueprint is drawn straight");
        assert_eq!(roughness(&Stage::new(100, 100).theme("chalkboard")), Some(1.8), "chalk is rougher");
        // Whichever comes last wins.
        let s = Stage::new(100, 100).theme("blueprint").sketch(crate::api::Sketch::default());
        assert_eq!(roughness(&s), Some(1.3));
        let s = Stage::new(100, 100).sketch(crate::api::Sketch::default().roughness(3.0)).theme("dark");
        assert_eq!(roughness(&s), Some(3.0), "a theme without strokes keeps the chosen ones");
    }

    #[test]
    #[should_panic(expected = "unknown theme \"neon\"")]
    fn an_unknown_theme_name_panics_on_the_stage_too() {
        let _ = Stage::new(100, 100).theme("neon");
    }

    #[test]
    fn a_custom_theme_works_on_a_stage() {
        let ocean = Theme::custom("#06283d", "#e0f2ff").title("#7dd3fc");
        let mut s = Stage::new(900, 600).theme(ocean.clone());
        let t = s.add(title("x"));
        let c = s.add(cube(10));
        assert_eq!(s.scene.get(t).unwrap().style.fill, Some("#7dd3fc".into_color()));
        assert_eq!(s.scene.get(c).unwrap().style.stroke, ocean.ink);
        assert_eq!(s.background, ocean.background);
    }

    /// Fraction of pixels that stand out from the page by at least `min` WCAG contrast.
    fn visible_ink(px: &Pixmap, page: Color, min: f32) -> usize {
        px.pixels()
            .iter()
            .filter(|p| {
                let c = Color::from_rgba8(p.red(), p.green(), p.blue(), 255);
                contrast(c, page) >= min
            })
            .count()
    }

    #[test]
    fn every_theme_draws_legible_default_ink_on_its_page() {
        // The real point of a theme: with no colours chosen, lines and text can be read.
        for t in Theme::all() {
            let mut s = Stage::new(900, 600).theme(t.clone());
            scene(&mut s);
            let frame = end_frame(&mut s);
            let ink = visible_ink(&frame, t.background, 4.5);
            assert!(ink > 1500, "{}: only {ink} legible pixels", t.name);
        }
    }

    #[test]
    fn without_a_theme_a_dark_page_loses_its_ink_which_is_what_themes_fix() {
        // A dark page alone leaves the default dark ink almost invisible (the problem themes solve).
        let dark_page = Color::from_rgba8(16, 16, 24, 255);
        let mut bare = Stage::new(900, 600).background(dark_page);
        scene(&mut bare);
        let bare_ink = visible_ink(&end_frame(&mut bare), dark_page, 4.5);

        let mut themed = Stage::new(900, 600).theme("dark");
        scene(&mut themed);
        let themed_ink = visible_ink(&end_frame(&mut themed), Theme::dark().background, 4.5);
        assert!(themed_ink > 5 * bare_ink.max(1), "themed {themed_ink} vs bare {bare_ink}");
    }

    #[test]
    fn hidden_line_patches_follow_the_themed_page() {
        // Filled shapes cover what is behind them with a patch of page colour. On a dark theme
        // that patch must be dark: not one pixel of the light page colour may leak through.
        let light_page = Color::from_rgba8(250, 249, 246, 255);
        for name in ["dark", "chalkboard", "blueprint", "terminal"] {
            let mut s = Stage::new(900, 600).theme(name);
            s.add(cuboid(120, 120, 120).base_at((0, 0, 0)).fill("slate"));
            s.add(cuboid(60, 60, 60).base_at((-20, 0, 90)).fill("amber")); // in front, overlapping
            let frame = end_frame(&mut s);
            let leaked = frame
                .pixels()
                .iter()
                .filter(|p| (p.red(), p.green(), p.blue()) == (250, 249, 246))
                .count();
            assert_eq!(leaked, 0, "{name}: {leaked} pixels of the light page colour");
            let _ = light_page;
        }
    }

    #[test]
    fn the_page_is_the_themed_background_in_the_pixels() {
        for t in Theme::all() {
            let mut s = Stage::new(100, 80).theme(t.clone());
            let frame = end_frame(&mut s);
            let corner = frame.pixel(1, 1).unwrap();
            let want = t.background.to_color_u8();
            assert_eq!((corner.red(), corner.green(), corner.blue()), (want.red(), want.green(), want.blue()), "{}", t.name);
        }
    }

    #[test]
    #[should_panic(expected = "unknown theme \"neon\": use one of light, dark")]
    fn an_unknown_name_panics_with_the_valid_ones() {
        let _ = "neon".into_theme();
    }
}
