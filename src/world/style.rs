use palette::Srgba;
use roughr::core::{FillStyle, Options};
use tiny_skia::Color;

/// Convenience: opaque colour from 8-bit RGB.
pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba8(r, g, b, 255)
}

/// The default colour of lines and text. Themes replace it.
pub fn ink() -> Color {
    rgb(35, 35, 40)
}

/// Which theme colour an object's lines or text use until a colour is chosen for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Lines and body text.
    Ink,
    /// Big titles.
    Title,
    /// Captions and labels.
    Caption,
}

/// Visual properties of an object. All of these are meant to be animatable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub stroke: Color,
    pub stroke_width: f32,
    /// `None` means no fill.
    pub fill: Option<Color>,
    /// 0.0 (invisible) to 1.0 (opaque); multiplies stroke and fill alpha.
    pub opacity: f32,
    /// How much of the shape has been drawn: 0.0 nothing, 1.0 everything. Below 1.0 the
    /// outline is traced pen-style and the fill fades in at the end (see `create`).
    pub reveal: f32,
    /// Fill pattern; `None` uses the scene's (e.g. hachure). Text uses `Solid`.
    pub fill_style: Option<FillStyle>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            stroke: ink(),
            stroke_width: 1.6,
            fill: None,
            opacity: 1.0,
            reveal: 1.0,
            fill_style: None,
        }
    }
}

impl Style {
    /// Colour with this style's opacity folded into its alpha.
    pub fn with_opacity(&self, c: Color) -> Color {
        let mut c = c;
        c.set_alpha((c.alpha() * self.opacity).clamp(0.0, 1.0));
        c
    }

    /// Build roughr options: scene-wide `base` (roughness, hachure, ...) overridden by this style.
    /// `fill` is passed separately so geometry can shade faces.
    pub fn rough_options(&self, base: &Options, fill: Option<Color>) -> Options {
        let mut opts = base.clone();
        opts.stroke = Some(to_srgba(self.with_opacity(self.stroke)));
        opts.stroke_width = Some(self.stroke_width);
        if self.fill_style.is_some() {
            opts.fill_style = self.fill_style;
        }
        opts.fill = fill.map(|c| to_srgba(self.with_opacity(c)));
        opts
    }
}

fn to_srgba(c: Color) -> Srgba {
    Srgba::new(c.red(), c.green(), c.blue(), c.alpha())
}

/// Multiply the RGB channels of a colour (alpha untouched), clamped to `[0, 1]`.
pub fn shade(c: Color, factor: f32) -> Color {
    Color::from_rgba(
        (c.red() * factor).clamp(0.0, 1.0),
        (c.green() * factor).clamp(0.0, 1.0),
        (c.blue() * factor).clamp(0.0, 1.0),
        c.alpha(),
    )
    .unwrap_or(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_scales_alpha() {
        let s = Style {
            opacity: 0.5,
            ..Style::default()
        };
        assert!((s.with_opacity(rgb(1, 2, 3)).alpha() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn shade_clamps() {
        let c = shade(rgb(200, 200, 200), 2.0);
        assert_eq!(c.red(), 1.0);
    }
}
